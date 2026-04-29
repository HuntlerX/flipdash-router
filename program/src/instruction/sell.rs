//! `SellTokensIx` — user sells flipcash currency, receives USDF.
//!
//! Flow: snapshot user's USDF balance → CPI flipcash sell (no internal
//! slippage, router enforces it) → reload USDF balance → take fee on the
//! gross USDF received → enforce `net >= min_amount_out`.

use steel::*;

use flipdash_router_api::prelude::*;

use crate::cpi::*;
use crate::instruction::{check_token_account, compute_fee, token_amount};

pub fn process_sell_tokens(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let args = SellTokensIx::try_from_bytes(data)?.to_struct();

    // Reject zero: flipcash sell treats in_amount==0 as "sell the entire
    // target ATA". The router must never expand a malformed call into a
    // full-balance liquidation.
    if args.in_amount == 0 {
        return Err(FlipdashRouterError::ZeroAmount.into());
    }

    let [
        user_info,
        fee_usdf_ata_info,
        user_target_ata_info,
        user_usdf_ata_info,
        usdf_mint_info,
        target_mint_info,
        flipcash_pool_info,
        flipcash_target_vault_info,
        flipcash_usdf_vault_info,
        flipcash_program_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // ─── Validation ────────────────────────────────────────────────────
    check_signer(user_info)?;
    check_program(token_program_info, &spl_token::id())?;

    if *usdf_mint_info.key != USDF_MINT {
        return Err(FlipdashRouterError::InvalidMint.into());
    }
    if *target_mint_info.key == USDF_MINT {
        return Err(FlipdashRouterError::InvalidMint.into());
    }

    check_token_account(user_target_ata_info, user_info.key, target_mint_info.key)?;
    check_token_account(user_usdf_ata_info, user_info.key, &USDF_MINT)?;
    if *fee_usdf_ata_info.key != FEE_USDF_ATA {
        return Err(FlipdashRouterError::InvalidFeeAccount.into());
    }

    // ─── Curve sell ────────────────────────────────────────────────────
    let usdf_before = token_amount(user_usdf_ata_info)?;

    // Pass 0 as the curve's slippage min — the router does its own check
    // below on the post-fee net. Tx atomicity means the user can't lose
    // funds if the router check fails (whole tx reverts).
    invoke_flipcash_sell(
        flipcash_program_info,
        user_info,
        flipcash_pool_info,
        target_mint_info,
        usdf_mint_info,
        flipcash_target_vault_info,
        flipcash_usdf_vault_info,
        user_target_ata_info,
        user_usdf_ata_info,
        token_program_info,
        args.in_amount,
        /* min_amount_out */ 0,
    )?;

    let usdf_after = token_amount(user_usdf_ata_info)?;
    let gross = usdf_after
        .checked_sub(usdf_before)
        .ok_or(FlipdashRouterError::Overflow)?;
    if gross == 0 {
        return Err(FlipdashRouterError::NoCurveOutput.into());
    }

    // ─── Fee + slippage ────────────────────────────────────────────────
    let fee = compute_fee(gross)?;
    let net = gross
        .checked_sub(fee)
        .ok_or(FlipdashRouterError::Overflow)?;

    if net < args.min_amount_out {
        return Err(FlipdashRouterError::SlippageExceeded.into());
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

    Ok(())
}
