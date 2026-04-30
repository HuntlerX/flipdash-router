//! `SellTokensViaBridgeIx` — user sells flipcash currency, receives USDC.
//!
//! Flow: snapshot user's USDF → CPI flipcash sell (no internal slippage) →
//! reload, take fee on gross USDF → bridge net USDF → USDC → snapshot user
//! USDC delta → enforce `delta >= min_amount_out`.

use steel::*;

use flipdash_router_api::prelude::*;

use crate::cpi::*;
use crate::instruction::{check_token_account, compute_fee, token_amount};

pub fn process_sell_tokens_via_bridge(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let args = SellTokensViaBridgeIx::try_from_bytes(data)?.to_struct();

    // See note in sell.rs::process_sell_tokens — Flipcash's zero sentinel.
    if args.in_amount == 0 {
        return Err(FlipdashRouterError::ZeroAmount.into());
    }

    let [
        user_info,
        fee_usdf_ata_info,
        user_target_ata_info,
        user_usdf_ata_info,
        user_usdc_ata_info,
        usdf_mint_info,
        usdc_mint_info,
        target_mint_info,
        flipcash_pool_info,
        flipcash_target_vault_info,
        flipcash_usdf_vault_info,
        bridge_pool_info,
        bridge_usdf_vault_info,
        bridge_usdc_vault_info,
        flipcash_program_info,
        usdf_swap_program_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // ─── Validation ────────────────────────────────────────────────────
    check_signer(user_info)?;
    check_program(token_program_info, &spl_token::id())?;

    if *flipcash_program_info.key != FLIPCASH_PROGRAM
        || *usdf_swap_program_info.key != USDF_SWAP_PROGRAM
    {
        return Err(FlipdashRouterError::InvalidProgram.into());
    }

    if *usdf_mint_info.key != USDF_MINT
        || *usdc_mint_info.key != USDC_MINT
        || *target_mint_info.key == USDF_MINT
        || *target_mint_info.key == USDC_MINT
    {
        return Err(FlipdashRouterError::InvalidMint.into());
    }

    if *bridge_pool_info.key != BRIDGE_POOL
        || *bridge_usdf_vault_info.key != BRIDGE_USDF_VAULT
        || *bridge_usdc_vault_info.key != BRIDGE_USDC_VAULT
    {
        return Err(FlipdashRouterError::InvalidBridgePool.into());
    }

    check_token_account(user_target_ata_info, user_info.key, target_mint_info.key)?;
    check_token_account(user_usdf_ata_info, user_info.key, &USDF_MINT)?;
    check_token_account(user_usdc_ata_info, user_info.key, &USDC_MINT)?;
    if *fee_usdf_ata_info.key != FEE_USDF_ATA {
        return Err(FlipdashRouterError::InvalidFeeAccount.into());
    }

    // ─── Curve sell → USDF ─────────────────────────────────────────────
    let usdf_before = token_amount(user_usdf_ata_info)?;

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

    // ─── Fee in USDF ───────────────────────────────────────────────────
    let fee = compute_fee(gross)?;
    let net_usdf = gross
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

    // ─── Bridge USDF → USDC (1:1) ──────────────────────────────────────
    let usdc_before = token_amount(user_usdc_ata_info)?;

    invoke_bridge_swap(
        usdf_swap_program_info,
        user_info,
        bridge_pool_info,
        bridge_usdf_vault_info,
        bridge_usdc_vault_info,
        user_usdf_ata_info,
        user_usdc_ata_info,
        token_program_info,
        net_usdf,
        /* usdf_to_other */ true,
    )?;

    let usdc_after = token_amount(user_usdc_ata_info)?;
    let received_usdc = usdc_after
        .checked_sub(usdc_before)
        .ok_or(FlipdashRouterError::Overflow)?;

    if received_usdc < args.min_amount_out {
        return Err(FlipdashRouterError::SlippageExceeded.into());
    }

    Ok(())
}
