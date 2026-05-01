pub mod buy;
pub mod buy_via_bridge;
pub mod buy_via_coinbase;
pub mod currency_to_currency;
pub mod sell;
pub mod sell_via_bridge;
pub mod sell_via_coinbase;

pub use buy::*;
pub use buy_via_bridge::*;
pub use buy_via_coinbase::*;
pub use currency_to_currency::*;
pub use sell::*;
pub use sell_via_bridge::*;
pub use sell_via_coinbase::*;

use solana_program::program_error::ProgramError;
use steel::*;

use flipdash_router_api::prelude::*;

/// Read the current SPL Token amount on `info`. Re-parses the account data
/// so callers can use this both before and after a CPI to compute deltas.
pub(crate) fn token_amount(info: &AccountInfo<'_>) -> Result<u64, ProgramError> {
    Ok(info.as_token_account()?.amount())
}

/// Compute fee = floor(`gross_usdf` × FEE_BPS / 10_000). Returns
/// `FlipdashRouterError::Overflow` on either the u128 multiplication
/// overflow or the final u64 cast (neither can fire for `u64` inputs +
/// `FEE_BPS = 85`, but we keep the checks as defense-in-depth).
pub(crate) fn compute_fee(gross_usdf: u64) -> Result<u64, ProgramError> {
    let f = (gross_usdf as u128)
        .checked_mul(FEE_BPS as u128)
        .ok_or(FlipdashRouterError::Overflow)?
        / FEE_BPS_DIVISOR;
    u64::try_from(f).map_err(|_| FlipdashRouterError::Overflow.into())
}

/// Validate that `info` is an SPL Token account owned by `expected_owner`
/// for `expected_mint`. Used for the user's input/output ATAs and for the
/// FEE_OWNER's USDF ATA.
pub(crate) fn check_token_account(
    info: &AccountInfo<'_>,
    expected_owner: &Pubkey,
    expected_mint: &Pubkey,
) -> ProgramResult {
    let acct = info.as_token_account()?;
    if acct.owner() != *expected_owner {
        return Err(FlipdashRouterError::InvalidFeeAccount.into());
    }
    if acct.mint() != *expected_mint {
        return Err(FlipdashRouterError::InvalidMint.into());
    }
    Ok(())
}
