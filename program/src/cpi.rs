//! CPI helpers for the two external programs we wrap: the Flipcash bonding
//! curve (`flipcash-program`) and the USDF↔USDC bridge (`usdf-swap-program`).
//!
//! Neither program is published as a crate we can depend on, so we encode
//! their instructions by hand against their public ABIs:
//!   - flipcash buy:  `[u8 disc=4][u64 in_amount LE][u64 min_amount_out LE]`
//!   - flipcash sell: `[u8 disc=5][u64 in_amount LE][u64 min_amount_out LE]`
//!   - bridge swap:   `[u8 disc=2][u64 amount LE][u8 usdf_to_other]`
//!
//! The program-account checks here are belt-and-suspenders: the SDK builders
//! always pass the right program IDs from the on-chain consts, but we
//! re-verify here so a buggy or malicious caller can't redirect a CPI.

use solana_program::{instruction::Instruction, program::invoke};
use steel::*;

use flipdash_router_api::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn invoke_flipcash_buy<'info>(
    flipcash_program: &AccountInfo<'info>,
    buyer: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    target_mint: &AccountInfo<'info>,
    base_mint: &AccountInfo<'info>,
    target_vault: &AccountInfo<'info>,
    base_vault: &AccountInfo<'info>,
    buyer_target: &AccountInfo<'info>,
    buyer_base: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    in_amount: u64,
    min_amount_out: u64,
) -> ProgramResult {
    if *flipcash_program.key != FLIPCASH_PROGRAM {
        return Err(FlipdashRouterError::InvalidProgram.into());
    }

    let mut data = Vec::with_capacity(17);
    data.push(FLIPCASH_IX_BUY);
    data.extend_from_slice(&in_amount.to_le_bytes());
    data.extend_from_slice(&min_amount_out.to_le_bytes());

    let ix = Instruction {
        program_id: FLIPCASH_PROGRAM,
        accounts: vec![
            AccountMeta::new(*buyer.key, true),
            AccountMeta::new_readonly(*pool.key, false),
            AccountMeta::new_readonly(*target_mint.key, false),
            AccountMeta::new_readonly(*base_mint.key, false),
            AccountMeta::new(*target_vault.key, false),
            AccountMeta::new(*base_vault.key, false),
            AccountMeta::new(*buyer_target.key, false),
            AccountMeta::new(*buyer_base.key, false),
            AccountMeta::new_readonly(*token_program.key, false),
        ],
        data,
    };
    invoke(
        &ix,
        &[
            buyer.clone(),
            pool.clone(),
            target_mint.clone(),
            base_mint.clone(),
            target_vault.clone(),
            base_vault.clone(),
            buyer_target.clone(),
            buyer_base.clone(),
            token_program.clone(),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn invoke_flipcash_sell<'info>(
    flipcash_program: &AccountInfo<'info>,
    seller: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    target_mint: &AccountInfo<'info>,
    base_mint: &AccountInfo<'info>,
    target_vault: &AccountInfo<'info>,
    base_vault: &AccountInfo<'info>,
    seller_target: &AccountInfo<'info>,
    seller_base: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    in_amount: u64,
    min_amount_out: u64,
) -> ProgramResult {
    if *flipcash_program.key != FLIPCASH_PROGRAM {
        return Err(FlipdashRouterError::InvalidProgram.into());
    }

    let mut data = Vec::with_capacity(17);
    data.push(FLIPCASH_IX_SELL);
    data.extend_from_slice(&in_amount.to_le_bytes());
    data.extend_from_slice(&min_amount_out.to_le_bytes());

    let ix = Instruction {
        program_id: FLIPCASH_PROGRAM,
        accounts: vec![
            AccountMeta::new(*seller.key, true),
            AccountMeta::new(*pool.key, false), // mut: sell mutates fees_accumulated
            AccountMeta::new_readonly(*target_mint.key, false),
            AccountMeta::new_readonly(*base_mint.key, false),
            AccountMeta::new(*target_vault.key, false),
            AccountMeta::new(*base_vault.key, false),
            AccountMeta::new(*seller_target.key, false),
            AccountMeta::new(*seller_base.key, false),
            AccountMeta::new_readonly(*token_program.key, false),
        ],
        data,
    };
    invoke(
        &ix,
        &[
            seller.clone(),
            pool.clone(),
            target_mint.clone(),
            base_mint.clone(),
            target_vault.clone(),
            base_vault.clone(),
            seller_target.clone(),
            seller_base.clone(),
            token_program.clone(),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn invoke_bridge_swap<'info>(
    bridge_program: &AccountInfo<'info>,
    user: &AccountInfo<'info>,
    pool: &AccountInfo<'info>,
    usdf_vault: &AccountInfo<'info>,
    other_vault: &AccountInfo<'info>,
    user_usdf_token: &AccountInfo<'info>,
    user_other_token: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount: u64,
    usdf_to_other: bool,
) -> ProgramResult {
    if *bridge_program.key != USDF_SWAP_PROGRAM {
        return Err(FlipdashRouterError::InvalidProgram.into());
    }

    let mut data = Vec::with_capacity(10);
    data.push(USDF_SWAP_IX_SWAP);
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(if usdf_to_other { 1 } else { 0 });

    let ix = Instruction {
        program_id: USDF_SWAP_PROGRAM,
        accounts: vec![
            AccountMeta::new(*user.key, true),
            AccountMeta::new_readonly(*pool.key, false),
            AccountMeta::new(*usdf_vault.key, false),
            AccountMeta::new(*other_vault.key, false),
            AccountMeta::new(*user_usdf_token.key, false),
            AccountMeta::new(*user_other_token.key, false),
            AccountMeta::new_readonly(*token_program.key, false),
        ],
        data,
    };
    invoke(
        &ix,
        &[
            user.clone(),
            pool.clone(),
            usdf_vault.clone(),
            other_vault.clone(),
            user_usdf_token.clone(),
            user_other_token.clone(),
            token_program.clone(),
        ],
    )
}

/// Plain SPL Token `Transfer` (legacy non-checked variant). The router uses
/// this only for taking the FlipDash fee from the user's USDF ATA into the
/// FEE_OWNER's USDF ATA. The mint is fixed (USDF) and the destination is
/// validated by the caller before this is reached.
pub fn spl_transfer<'info>(
    source: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount: u64,
) -> ProgramResult {
    let mut data = Vec::with_capacity(9);
    data.push(3); // SPL Token instruction 3 = Transfer
    data.extend_from_slice(&amount.to_le_bytes());

    let ix = Instruction {
        program_id: spl_token::id(),
        accounts: vec![
            AccountMeta::new(*source.key, false),
            AccountMeta::new(*destination.key, false),
            AccountMeta::new_readonly(*authority.key, true),
        ],
        data,
    };
    invoke(
        &ix,
        &[source.clone(), destination.clone(), authority.clone(), token_program.clone()],
    )
}
