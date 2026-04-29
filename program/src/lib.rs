//! On-chain entrypoint for the FlipDash Router.
//!
//! The router exposes four user-signed instructions that wrap the Flipcash
//! bonding curve and the USDF↔USDC bridge into atomic CPIs, with a fixed
//! 0.85% fee in USDF on every swap.
//!
//! - [`instruction::buy`] — USDF → flipcash currency
//! - [`instruction::buy_via_bridge`] — USDC → bridge → USDF → flipcash currency
//! - [`instruction::sell`] — flipcash currency → USDF
//! - [`instruction::sell_via_bridge`] — flipcash currency → USDF → bridge → USDC
//!
//! All CPIs go through helpers in [`cpi`], which re-verify program IDs as a
//! defense-in-depth check. There are no admin functions, no PDA-owned funds,
//! no settable parameters; the fee receiver and rate are baked into the
//! binary.

#![allow(unexpected_cfgs)]

use steel::*;
use flipdash_router_api::prelude::*;

pub mod cpi;
pub mod instruction;
mod security;

use instruction::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&flipdash_router_api::ID, program_id, data)?;

    match ix {
        InstructionType::Unknown => return Err(ProgramError::InvalidInstructionData),
        InstructionType::BuyTokensIx => process_buy_tokens(accounts, data)?,
        InstructionType::BuyTokensViaBridgeIx => process_buy_tokens_via_bridge(accounts, data)?,
        InstructionType::SellTokensIx => process_sell_tokens(accounts, data)?,
        InstructionType::SellTokensViaBridgeIx => process_sell_tokens_via_bridge(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
