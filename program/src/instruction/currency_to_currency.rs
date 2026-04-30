//! `CurrencyToCurrencyIx` — user swaps one flipcash currency for another in
//! a single signed tx.
//!
//! Flow:
//!   1. Snapshot user USDF balance.
//!   2. CPI flipcash sell src → USDF (curve enforces its own 1% pool fee).
//!   3. Reload USDF, take a single 0.85% router fee on the gross delta.
//!   4. Snapshot user dst balance.
//!   5. CPI flipcash buy USDF (post-fee) → dst.
//!   6. Reload dst, enforce `delta >= min_amount_out`.
//!
//! Why a dedicated ix instead of the client emitting Sell+Buy back-to-back:
//!   - The buy needs to know the *exact* USDF delivered by the sell, which
//!     isn't known off-chain (curve pool state can drift between quote and
//!     execution). Doing it in one program loop reads the snapshot atomically.
//!   - One fee transfer instead of two — user pays 0.85% (this ix) vs ~1.7%
//!     (manual two-tx). Same revenue per "round trip" as the underlying
//!     legs would normally produce — both legs would charge if separate.
//!   - Net-zero USDF flow on the user's side under a normal buy: the curve
//!     consumes the full `net_usdf` we hand it. If a future flipcash change
//!     ever caps the actual consumed input below `net_usdf` (e.g. supply
//!     ceiling), any unused USDF would be left in the user's USDF ATA —
//!     not lost, just stranded. The slippage check is on dst delta so the
//!     user is still protected against bad fills.

use steel::*;

use flipdash_router_api::prelude::*;

use crate::cpi::*;
use crate::instruction::{check_token_account, compute_fee, token_amount};

pub fn process_currency_to_currency(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let args = CurrencyToCurrencyIx::try_from_bytes(data)?.to_struct();

    // Reject zero — same reason as sell.rs: flipcash sell treats `in_amount==0`
    // as "use the entire ATA".
    if args.in_amount == 0 {
        return Err(FlipdashRouterError::ZeroAmount.into());
    }

    let [
        user_info,
        fee_usdf_ata_info,
        user_src_ata_info,
        user_usdf_ata_info,
        user_dst_ata_info,
        usdf_mint_info,
        src_mint_info,
        dst_mint_info,
        flipcash_pool_src_info,
        flipcash_src_vault_info,
        flipcash_usdf_vault_src_info,
        flipcash_pool_dst_info,
        flipcash_dst_vault_info,
        flipcash_usdf_vault_dst_info,
        flipcash_program_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // ─── Validation ────────────────────────────────────────────────────
    check_signer(user_info)?;
    check_program(token_program_info, &spl_token::id())?;

    if *flipcash_program_info.key != FLIPCASH_PROGRAM {
        return Err(FlipdashRouterError::InvalidProgram.into());
    }

    if *usdf_mint_info.key != USDF_MINT {
        return Err(FlipdashRouterError::InvalidMint.into());
    }
    // Both src and dst must be flipcash currencies, not USDF.
    if *src_mint_info.key == USDF_MINT || *dst_mint_info.key == USDF_MINT {
        return Err(FlipdashRouterError::InvalidMint.into());
    }
    // Block same-mint swaps. Without this guard a user could pay a 0.85% fee
    // for a no-op (curve sell + curve buy on the same pool nets to ~0 minus
    // pool sell fee + slippage). Refusing at entry keeps the only legitimate
    // call shape unambiguous.
    if *src_mint_info.key == *dst_mint_info.key {
        return Err(FlipdashRouterError::InvalidMint.into());
    }

    check_token_account(user_src_ata_info, user_info.key, src_mint_info.key)?;
    check_token_account(user_usdf_ata_info, user_info.key, &USDF_MINT)?;
    check_token_account(user_dst_ata_info, user_info.key, dst_mint_info.key)?;
    if *fee_usdf_ata_info.key != FEE_USDF_ATA {
        return Err(FlipdashRouterError::InvalidFeeAccount.into());
    }

    // ─── 1. Curve sell src → USDF ──────────────────────────────────────
    let usdf_before = token_amount(user_usdf_ata_info)?;

    invoke_flipcash_sell(
        flipcash_program_info,
        user_info,
        flipcash_pool_src_info,
        src_mint_info,
        usdf_mint_info,
        flipcash_src_vault_info,
        flipcash_usdf_vault_src_info,
        user_src_ata_info,
        user_usdf_ata_info,
        token_program_info,
        args.in_amount,
        /* min_amount_out */ 0,
    )?;

    let usdf_after_sell = token_amount(user_usdf_ata_info)?;
    let gross = usdf_after_sell
        .checked_sub(usdf_before)
        .ok_or(FlipdashRouterError::Overflow)?;
    if gross == 0 {
        return Err(FlipdashRouterError::NoCurveOutput.into());
    }

    // ─── 2. Single router fee on the intermediate USDF ─────────────────
    let fee = compute_fee(gross)?;
    let net_usdf = gross
        .checked_sub(fee)
        .ok_or(FlipdashRouterError::Overflow)?;
    if net_usdf == 0 {
        // gross was so small the fee ate it all. Buy with 0 would re-trigger
        // the flipcash zero-sentinel, so refuse here.
        return Err(FlipdashRouterError::NoCurveOutput.into());
    }

    if fee > 0 {
        spl_transfer(
            user_usdf_ata_info,
            fee_usdf_ata_info,
            user_info,
            token_program_info,
            fee,
        )?;
    }

    // ─── 3. Curve buy USDF → dst ───────────────────────────────────────
    // Snapshot dst balance pre-CPI so we can compute the user's *actual*
    // dst delta (independent of any preexisting balance) and apply
    // `min_amount_out` against it.
    let dst_before = token_amount(user_dst_ata_info)?;

    invoke_flipcash_buy(
        flipcash_program_info,
        user_info,
        flipcash_pool_dst_info,
        dst_mint_info,
        usdf_mint_info,
        flipcash_dst_vault_info,
        flipcash_usdf_vault_dst_info,
        user_dst_ata_info,
        user_usdf_ata_info,
        token_program_info,
        net_usdf,
        /* min_amount_out */ 0,
    )?;

    let dst_after = token_amount(user_dst_ata_info)?;
    let dst_delta = dst_after
        .checked_sub(dst_before)
        .ok_or(FlipdashRouterError::Overflow)?;

    if dst_delta < args.min_amount_out {
        return Err(FlipdashRouterError::SlippageExceeded.into());
    }

    Ok(())
}
