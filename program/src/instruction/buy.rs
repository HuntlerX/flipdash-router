//! `BuyTokensIx` — user pays USDF, gets a flipcash currency token.
//!
//! Flow: take fee in USDF → CPI flipcash buy with `(in_amount - fee)`.
//! Flipcash enforces the user's `min_amount_out` in target tokens; we don't
//! re-check on top.

use steel::*;

use flipdash_router_api::prelude::*;

use crate::cpi::*;
use crate::instruction::{check_token_account, compute_fee};

pub fn process_buy_tokens(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let args = BuyTokensIx::try_from_bytes(data)?.to_struct();

    // Reject zero up-front: flipcash-program reads in_amount==0 as a
    // sentinel meaning "use the caller's full base ATA balance", which
    // would silently turn this into a full-balance trade with zero fee.
    if args.in_amount == 0 {
        return Err(FlipdashRouterError::ZeroAmount.into());
    }

    let [
        user_info,
        fee_usdf_ata_info,
        user_usdf_ata_info,
        user_target_ata_info,
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

    // ─── Account validation ────────────────────────────────────────────
    check_signer(user_info)?;
    check_program(token_program_info, &spl_token::id())?;

    if *flipcash_program_info.key != FLIPCASH_PROGRAM {
        return Err(FlipdashRouterError::InvalidProgram.into());
    }

    if *usdf_mint_info.key != USDF_MINT {
        return Err(FlipdashRouterError::InvalidMint.into());
    }
    if *target_mint_info.key == USDF_MINT {
        // The "target" mint must be a flipcash currency, not USDF itself.
        return Err(FlipdashRouterError::InvalidMint.into());
    }

    check_token_account(user_usdf_ata_info, user_info.key, &USDF_MINT)?;
    check_token_account(user_target_ata_info, user_info.key, target_mint_info.key)?;
    if *fee_usdf_ata_info.key != FEE_USDF_ATA {
        return Err(FlipdashRouterError::InvalidFeeAccount.into());
    }

    // ─── Fee transfer ──────────────────────────────────────────────────
    let fee = compute_fee(args.in_amount)?;
    let net_in = args
        .in_amount
        .checked_sub(fee)
        .ok_or(FlipdashRouterError::Overflow)?;

    if fee > 0 {
        spl_transfer(
            user_usdf_ata_info,
            fee_usdf_ata_info,
            user_info,
            token_program_info,
            fee,
        )?;
    }

    // ─── Flipcash buy CPI ──────────────────────────────────────────────
    invoke_flipcash_buy(
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
        net_in,
        args.min_amount_out,
    )?;

    Ok(())
}
