//! `SellTokensViaCoinbaseIx` — same shape as `SellTokensViaBridgeIx` but
//! routes the USDF→USDC leg through the Coinbase ocp-server stable
//! swapper (`pqgqKa…`).
//!
//! Flow: snapshot user USDF → CPI flipcash sell (no internal slippage) →
//! reload, take 0.85% router fee on gross USDF → bridge net USDF → USDC
//! via Coinbase pool → enforce `dst delta >= min_amount_out` against the
//! user's USDC ATA.

use steel::*;

use flipdash_router_api::prelude::*;

use crate::cpi::*;
use crate::instruction::{check_token_account, compute_fee, token_amount};

pub fn process_sell_tokens_via_coinbase(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let args = SellTokensViaCoinbaseIx::try_from_bytes(data)?.to_struct();

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
        // Flipcash curve accounts
        flipcash_pool_info,
        flipcash_target_vault_info,
        flipcash_usdf_vault_info,
        // Coinbase bridge accounts
        coinbase_pool_info,
        coinbase_usdf_vault_info,
        coinbase_usdf_vault_acct_info,
        coinbase_usdc_vault_info,
        coinbase_usdc_vault_acct_info,
        coinbase_fee_recipient_info,
        coinbase_fee_recip_ata_info,
        coinbase_whitelist_info,
        // Programs
        flipcash_program_info,
        coinbase_program_info,
        token_program_info,
        ata_program_info,
        system_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // ─── Validation ────────────────────────────────────────────────────
    check_signer(user_info)?;
    check_program(token_program_info, &spl_token::id())?;

    if *flipcash_program_info.key != FLIPCASH_PROGRAM
        || *coinbase_program_info.key != COINBASE_PROGRAM
        || *ata_program_info.key != ATA_PROGRAM
        || *system_program_info.key != solana_program::system_program::ID
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

    // Pin Coinbase pool/vault/fee accounts. fee_recipient_ATA is the
    // USDF-direction one because we're sending USDF into the pool.
    if *coinbase_pool_info.key != COINBASE_POOL
        || *coinbase_usdf_vault_info.key != COINBASE_USDF_VAULT
        || *coinbase_usdf_vault_acct_info.key != COINBASE_USDF_VAULT_ACCT
        || *coinbase_usdc_vault_info.key != COINBASE_USDC_VAULT
        || *coinbase_usdc_vault_acct_info.key != COINBASE_USDC_VAULT_ACCT
        || *coinbase_fee_recipient_info.key != COINBASE_FEE_RECIPIENT
        || *coinbase_fee_recip_ata_info.key != COINBASE_FEE_RECIP_USDF_ATA
        || *coinbase_whitelist_info.key != COINBASE_WHITELIST
    {
        return Err(FlipdashRouterError::InvalidCoinbasePool.into());
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

    // ─── Bridge USDF → USDC via Coinbase pool (1:1) ────────────────────
    // Snapshot BOTH sides. The destination delta is what we enforce
    // `min_amount_out` against. The source delta MUST equal exactly
    // `net_usdf` — anything else means the closed-source Coinbase
    // program over-debited the user (defended against malicious
    // upgrade of `pqgqKa…`).
    let usdf_before_bridge = token_amount(user_usdf_ata_info)?;
    let usdc_before = token_amount(user_usdc_ata_info)?;

    invoke_coinbase_swap(
        coinbase_program_info,
        coinbase_pool_info,
        coinbase_usdf_vault_info,
        coinbase_usdc_vault_info,
        coinbase_usdf_vault_acct_info,
        coinbase_usdc_vault_acct_info,
        user_usdf_ata_info,
        user_usdc_ata_info,
        coinbase_fee_recip_ata_info,
        coinbase_fee_recipient_info,
        usdf_mint_info,
        usdc_mint_info,
        user_info,
        coinbase_whitelist_info,
        token_program_info,
        ata_program_info,
        system_program_info,
        net_usdf,
        /* min_amount_out */ 0,
    )?;

    let usdc_after = token_amount(user_usdc_ata_info)?;
    let usdf_after_bridge = token_amount(user_usdf_ata_info)?;
    let received_usdc = usdc_after
        .checked_sub(usdc_before)
        .ok_or(FlipdashRouterError::Overflow)?;

    if received_usdc < args.min_amount_out {
        return Err(FlipdashRouterError::SlippageExceeded.into());
    }

    // Source-debit cap (see buy_via_coinbase.rs for rationale). The
    // Coinbase CPI must have taken EXACTLY `net_usdf` from the user's
    // USDF ATA — no more, no less.
    let debited = usdf_before_bridge
        .checked_sub(usdf_after_bridge)
        .ok_or(FlipdashRouterError::SourceOverDebit)?;
    if debited != net_usdf {
        return Err(FlipdashRouterError::SourceOverDebit.into());
    }

    Ok(())
}
