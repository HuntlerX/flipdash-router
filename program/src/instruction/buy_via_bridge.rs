//! `BuyTokensViaBridgeIx` — user pays USDC, gets a flipcash currency.
//!
//! Flow: bridge USDC→USDF (1:1) → take fee in USDF → CPI flipcash buy with
//! the post-fee USDF. Flipcash enforces `min_amount_out` in target tokens.

use steel::*;

use flipdash_router_api::prelude::*;

use crate::cpi::*;
use crate::instruction::{check_token_account, compute_fee, token_amount};

pub fn process_buy_tokens_via_bridge(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let args = BuyTokensViaBridgeIx::try_from_bytes(data)?.to_struct();

    if args.in_amount == 0 {
        return Err(FlipdashRouterError::ZeroAmount.into());
    }

    let [
        user_info,
        fee_usdf_ata_info,
        user_usdc_ata_info,
        user_usdf_ata_info,
        user_target_ata_info,
        usdc_mint_info,
        usdf_mint_info,
        target_mint_info,
        bridge_pool_info,
        bridge_usdf_vault_info,
        bridge_usdc_vault_info,
        flipcash_pool_info,
        flipcash_target_vault_info,
        flipcash_usdf_vault_info,
        usdf_swap_program_info,
        flipcash_program_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // ─── Account validation ────────────────────────────────────────────
    check_signer(user_info)?;
    check_program(token_program_info, &spl_token::id())?;

    if *usdc_mint_info.key != USDC_MINT
        || *usdf_mint_info.key != USDF_MINT
        || *target_mint_info.key == USDF_MINT
        || *target_mint_info.key == USDC_MINT
    {
        return Err(FlipdashRouterError::InvalidMint.into());
    }

    // Pin the canonical bridge accounts. Otherwise an off-chain caller
    // could plug in a non-1:1 pool and the assumed `args.in_amount` USDF
    // we charge fee on would diverge from what actually arrived.
    if *bridge_pool_info.key != BRIDGE_POOL
        || *bridge_usdf_vault_info.key != BRIDGE_USDF_VAULT
        || *bridge_usdc_vault_info.key != BRIDGE_USDC_VAULT
    {
        return Err(FlipdashRouterError::InvalidBridgePool.into());
    }

    check_token_account(user_usdc_ata_info, user_info.key, &USDC_MINT)?;
    check_token_account(user_usdf_ata_info, user_info.key, &USDF_MINT)?;
    check_token_account(user_target_ata_info, user_info.key, target_mint_info.key)?;
    if *fee_usdf_ata_info.key != FEE_USDF_ATA {
        return Err(FlipdashRouterError::InvalidFeeAccount.into());
    }

    // ─── 1. Bridge USDC → USDF — observe actual delta ──────────────────
    // Defense-in-depth on top of the pinned pool above: derive fee + buy
    // input from the *observed* USDF delta, not from the assumed
    // `args.in_amount`. If the bridge ever stops being 1:1 (or the pinned
    // pool's decimals change), fee accounting still tracks reality.
    let usdf_before = token_amount(user_usdf_ata_info)?;

    invoke_bridge_swap(
        usdf_swap_program_info,
        user_info,
        bridge_pool_info,
        bridge_usdf_vault_info,
        bridge_usdc_vault_info,
        user_usdf_ata_info,
        user_usdc_ata_info,
        token_program_info,
        args.in_amount,
        /* usdf_to_other */ false,
    )?;

    let usdf_after = token_amount(user_usdf_ata_info)?;
    let bridged = usdf_after
        .checked_sub(usdf_before)
        .ok_or(FlipdashRouterError::Overflow)?;
    if bridged == 0 {
        return Err(FlipdashRouterError::NoCurveOutput.into());
    }

    // ─── 2. Fee in USDF (off the actual bridged amount) ────────────────
    let fee = compute_fee(bridged)?;
    let net_in = bridged
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

    // ─── 3. Flipcash buy ───────────────────────────────────────────────
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
