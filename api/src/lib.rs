//! Shared types and SDK builders for the FlipDash Router program.
//!
//! This crate is consumed both on-chain (by the program crate) and off-chain
//! (by clients that need to build router instructions). The split is:
//!
//! - [`consts`] — pinned addresses, fee constants, ABI discriminators of the
//!   external programs we CPI into.
//! - [`error`] — `FlipdashRouterError` enum mapped to Solana custom error codes.
//! - [`instruction`] — wire-format `repr(C)` ix structs + the `InstructionType`
//!   discriminator enum.
//! - [`utils`] — small validation helpers used on-chain.
//! - [`sdk`] — `build_*_ix` instruction builders. Off-chain only.
//!
//! The `prelude` re-exports the pieces a typical caller needs.

#![allow(unexpected_cfgs)]

pub mod consts;
pub mod error;
pub mod instruction;
pub mod utils;

#[cfg(not(target_os = "solana"))]
pub mod sdk;

pub mod prelude {
    pub use crate::consts::*;
    pub use crate::error::*;
    pub use crate::instruction::*;
    pub use crate::utils::*;

    #[cfg(not(target_os = "solana"))]
    pub use crate::sdk::*;
}

use steel::*;

declare_id!("Dash3ZZKehWHGNvbCpkde6gvJTR2io7YZCt5DyU73PuJ");
