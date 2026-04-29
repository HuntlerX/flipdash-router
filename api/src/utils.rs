use steel::*;
use solana_program::msg;

/// Errors-out if the given condition is false. Mirrors the helper used in
/// `flipcash-program` so on-chain logs remain consistent across the two
/// programs.
pub fn check_condition(condition: bool, message: &str) -> ProgramResult {
    if !condition {
        msg!("Failed condition: {}", message);
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    account.is_signer()?.is_writable()?;
    Ok(())
}

pub fn check_mut(account: &AccountInfo) -> ProgramResult {
    account.is_writable()?;
    Ok(())
}

pub fn check_program(account: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
    account.is_program(program_id)?;
    Ok(())
}
