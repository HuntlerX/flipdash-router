//! `BuyTokensViaCoinbaseIx` — same shape as `BuyTokensViaBridgeIx` but
//! routes the USDC→USDF leg through the Coinbase ocp-server stable
//! swapper (`pqgqKa…`) instead of the usdf-swap-program.
//!
//! Flow: bridge USDC→USDF (Coinbase) → take fee in USDF on the actual
//! delta → CPI flipcash buy with the post-fee USDF.
//!
//! Why a separate ix vs reusing `BuyTokensViaBridgeIx`: the Coinbase pool
//! is a different program with a different account layout (16 accounts
//! including a Whitelist + ATA + System program; the usdf-swap path needs
//! 7 accounts). Pinning each program/pool/vault as a `==` constant is the
//! defense-in-depth posture the router has held since v0.1.0; we keep
//! the constants narrow per ix instead of allowing either bridge.

use steel::*;

use flipdash_router_api::prelude::*;

use crate::cpi::*;
use crate::instruction::{check_token_account, compute_fee, token_amount};

pub fn process_buy_tokens_via_coinbase(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let args = BuyTokensViaCoinbaseIx::try_from_bytes(data)?.to_struct();

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
        // Coinbase bridge accounts
        coinbase_pool_info,
        coinbase_usdc_vault_info,
        coinbase_usdc_vault_acct_info,
        coinbase_usdf_vault_info,
        coinbase_usdf_vault_acct_info,
        coinbase_fee_recipient_info,
        coinbase_fee_recip_ata_info,
        coinbase_whitelist_info,
        // Flipcash curve accounts
        flipcash_pool_info,
        flipcash_target_vault_info,
        flipcash_usdf_vault_info,
        // Programs
        coinbase_program_info,
        flipcash_program_info,
        token_program_info,
        ata_program_info,
        system_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // ─── Account validation ────────────────────────────────────────────
    check_signer(user_info)?;
    check_program(token_program_info, &spl_token::id())?;

    // Fail-fast lift: program-id checks before any spl_transfer or CPI.
    if *flipcash_program_info.key != FLIPCASH_PROGRAM
        || *coinbase_program_info.key != COINBASE_PROGRAM
        || *ata_program_info.key != ATA_PROGRAM
        || *system_program_info.key != solana_program::system_program::ID
    {
        return Err(FlipdashRouterError::InvalidProgram.into());
    }

    if *usdc_mint_info.key != USDC_MINT
        || *usdf_mint_info.key != USDF_MINT
        || *target_mint_info.key == USDF_MINT
        || *target_mint_info.key == USDC_MINT
    {
        return Err(FlipdashRouterError::InvalidMint.into());
    }

    // Pin every Coinbase-side account so a bogus or upgraded pool can't
    // be supplied by the off-chain caller. fee_recipient_ATA is the
    // USDC-direction one because we're sending USDC into the pool.
    if *coinbase_pool_info.key != COINBASE_POOL
        || *coinbase_usdc_vault_info.key != COINBASE_USDC_VAULT
        || *coinbase_usdc_vault_acct_info.key != COINBASE_USDC_VAULT_ACCT
        || *coinbase_usdf_vault_info.key != COINBASE_USDF_VAULT
        || *coinbase_usdf_vault_acct_info.key != COINBASE_USDF_VAULT_ACCT
        || *coinbase_fee_recipient_info.key != COINBASE_FEE_RECIPIENT
        || *coinbase_fee_recip_ata_info.key != COINBASE_FEE_RECIP_USDC_ATA
        || *coinbase_whitelist_info.key != COINBASE_WHITELIST
    {
        return Err(FlipdashRouterError::InvalidCoinbasePool.into());
    }

    check_token_account(user_usdc_ata_info, user_info.key, &USDC_MINT)?;
    check_token_account(user_usdf_ata_info, user_info.key, &USDF_MINT)?;
    check_token_account(user_target_ata_info, user_info.key, target_mint_info.key)?;
    if *fee_usdf_ata_info.key != FEE_USDF_ATA {
        return Err(FlipdashRouterError::InvalidFeeAccount.into());
    }

    // ─── 1. Bridge USDC → USDF — observe actual deltas on BOTH sides ──
    // Same defense-in-depth as the usdf-swap path on the destination
    // side (derive fee + buy input from the *observed* USDF delta, not
    // from the assumed `args.in_amount`). On top of that, the Coinbase
    // program is closed-source and upgradable — a malicious upgrade
    // could over-debit `user_usdc_ata` while still crediting enough
    // USDF for the slippage check to pass. Snapshot the source side
    // too and require it decreased by *exactly* `in_amount`.
    let usdc_before = token_amount(user_usdc_ata_info)?;
    let usdf_before = token_amount(user_usdf_ata_info)?;

    invoke_coinbase_swap(
        coinbase_program_info,
        coinbase_pool_info,
        coinbase_usdc_vault_info,
        coinbase_usdf_vault_info,
        coinbase_usdc_vault_acct_info,
        coinbase_usdf_vault_acct_info,
        user_usdc_ata_info,
        user_usdf_ata_info,
        coinbase_fee_recip_ata_info,
        coinbase_fee_recipient_info,
        usdc_mint_info,
        usdf_mint_info,
        user_info,
        coinbase_whitelist_info,
        token_program_info,
        ata_program_info,
        system_program_info,
        args.in_amount,
        /* min_amount_out */ 0,
    )?;

    let usdf_after = token_amount(user_usdf_ata_info)?;
    let usdc_after = token_amount(user_usdc_ata_info)?;
    let bridged = usdf_after
        .checked_sub(usdf_before)
        .ok_or(FlipdashRouterError::Overflow)?;
    if bridged == 0 {
        return Err(FlipdashRouterError::NoCurveOutput.into());
    }
    // Strict source-debit cap. If the Coinbase program took more than
    // `in_amount` (or somehow less, which we also treat as a malformed
    // execution), revert the whole tx. checked_sub returns None on
    // underflow (i.e. usdc_after > usdc_before, which would be free
    // tokens for the user — also unexpected; reject).
    let debited = usdc_before
        .checked_sub(usdc_after)
        .ok_or(FlipdashRouterError::SourceOverDebit)?;
    if debited != args.in_amount {
        return Err(FlipdashRouterError::SourceOverDebit.into());
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
