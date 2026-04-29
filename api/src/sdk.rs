use steel::*;

use crate::consts::*;
use crate::instruction::*;

/// Build a `BuyTokensIx` (USDF → flipcash currency).
///
/// The caller (off-chain backend) is responsible for:
///   - knowing the flipcash `pool` address and the two vault addresses for
///     the target currency,
///   - having pre-created the user's USDF/target ATAs and the FEE_OWNER's
///     USDF ATA (the router never creates ATAs — wallets do).
#[allow(clippy::too_many_arguments)]
pub fn build_buy_tokens_ix(
    user: Pubkey,
    fee_usdf_ata: Pubkey,
    user_usdf_ata: Pubkey,
    user_target_ata: Pubkey,
    target_mint: Pubkey,
    flipcash_pool: Pubkey,
    flipcash_target_vault: Pubkey,
    flipcash_usdf_vault: Pubkey,
    in_amount: u64,
    min_amount_out: u64,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(fee_usdf_ata, false),
            AccountMeta::new(user_usdf_ata, false),
            AccountMeta::new(user_target_ata, false),
            AccountMeta::new_readonly(USDF_MINT, false),
            AccountMeta::new_readonly(target_mint, false),
            AccountMeta::new_readonly(flipcash_pool, false),
            AccountMeta::new(flipcash_target_vault, false),
            AccountMeta::new(flipcash_usdf_vault, false),
            AccountMeta::new_readonly(FLIPCASH_PROGRAM, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: BuyTokensIx::from_struct(ParsedSwapArgs {
            in_amount,
            min_amount_out,
        })
        .to_bytes(),
    }
}

/// Build a `BuyTokensViaBridgeIx` (USDC → bridge → USDF → flipcash currency).
#[allow(clippy::too_many_arguments)]
pub fn build_buy_tokens_via_bridge_ix(
    user: Pubkey,
    fee_usdf_ata: Pubkey,
    user_usdc_ata: Pubkey,
    user_usdf_ata: Pubkey,
    user_target_ata: Pubkey,
    target_mint: Pubkey,
    bridge_pool: Pubkey,
    bridge_usdf_vault: Pubkey,
    bridge_usdc_vault: Pubkey,
    flipcash_pool: Pubkey,
    flipcash_target_vault: Pubkey,
    flipcash_usdf_vault: Pubkey,
    in_amount: u64,
    min_amount_out: u64,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(fee_usdf_ata, false),
            AccountMeta::new(user_usdc_ata, false),
            AccountMeta::new(user_usdf_ata, false),
            AccountMeta::new(user_target_ata, false),
            AccountMeta::new_readonly(USDC_MINT, false),
            AccountMeta::new_readonly(USDF_MINT, false),
            AccountMeta::new_readonly(target_mint, false),
            AccountMeta::new_readonly(bridge_pool, false),
            AccountMeta::new(bridge_usdf_vault, false),
            AccountMeta::new(bridge_usdc_vault, false),
            AccountMeta::new_readonly(flipcash_pool, false),
            AccountMeta::new(flipcash_target_vault, false),
            AccountMeta::new(flipcash_usdf_vault, false),
            AccountMeta::new_readonly(USDF_SWAP_PROGRAM, false),
            AccountMeta::new_readonly(FLIPCASH_PROGRAM, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: BuyTokensViaBridgeIx::from_struct(ParsedSwapArgs {
            in_amount,
            min_amount_out,
        })
        .to_bytes(),
    }
}

/// Build a `SellTokensIx` (flipcash currency → USDF).
#[allow(clippy::too_many_arguments)]
pub fn build_sell_tokens_ix(
    user: Pubkey,
    fee_usdf_ata: Pubkey,
    user_target_ata: Pubkey,
    user_usdf_ata: Pubkey,
    target_mint: Pubkey,
    flipcash_pool: Pubkey,
    flipcash_target_vault: Pubkey,
    flipcash_usdf_vault: Pubkey,
    in_amount: u64,
    min_amount_out: u64,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(fee_usdf_ata, false),
            AccountMeta::new(user_target_ata, false),
            AccountMeta::new(user_usdf_ata, false),
            AccountMeta::new_readonly(USDF_MINT, false),
            AccountMeta::new_readonly(target_mint, false),
            AccountMeta::new(flipcash_pool, false), // mut on sell path
            AccountMeta::new(flipcash_target_vault, false),
            AccountMeta::new(flipcash_usdf_vault, false),
            AccountMeta::new_readonly(FLIPCASH_PROGRAM, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: SellTokensIx::from_struct(ParsedSwapArgs {
            in_amount,
            min_amount_out,
        })
        .to_bytes(),
    }
}

/// Build a `SellTokensViaBridgeIx` (flipcash currency → USDF → bridge → USDC).
#[allow(clippy::too_many_arguments)]
pub fn build_sell_tokens_via_bridge_ix(
    user: Pubkey,
    fee_usdf_ata: Pubkey,
    user_target_ata: Pubkey,
    user_usdf_ata: Pubkey,
    user_usdc_ata: Pubkey,
    target_mint: Pubkey,
    flipcash_pool: Pubkey,
    flipcash_target_vault: Pubkey,
    flipcash_usdf_vault: Pubkey,
    bridge_pool: Pubkey,
    bridge_usdf_vault: Pubkey,
    bridge_usdc_vault: Pubkey,
    in_amount: u64,
    min_amount_out: u64,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(fee_usdf_ata, false),
            AccountMeta::new(user_target_ata, false),
            AccountMeta::new(user_usdf_ata, false),
            AccountMeta::new(user_usdc_ata, false),
            AccountMeta::new_readonly(USDF_MINT, false),
            AccountMeta::new_readonly(USDC_MINT, false),
            AccountMeta::new_readonly(target_mint, false),
            AccountMeta::new(flipcash_pool, false), // mut on sell path
            AccountMeta::new(flipcash_target_vault, false),
            AccountMeta::new(flipcash_usdf_vault, false),
            AccountMeta::new_readonly(bridge_pool, false),
            AccountMeta::new(bridge_usdf_vault, false),
            AccountMeta::new(bridge_usdc_vault, false),
            AccountMeta::new_readonly(FLIPCASH_PROGRAM, false),
            AccountMeta::new_readonly(USDF_SWAP_PROGRAM, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: SellTokensViaBridgeIx::from_struct(ParsedSwapArgs {
            in_amount,
            min_amount_out,
        })
        .to_bytes(),
    }
}
