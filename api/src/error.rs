use steel::*;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum FlipdashRouterError {
    /// Arithmetic overflow.
    Overflow = 0,
    /// User passed in an unexpected mint for one of the input/output ATAs.
    InvalidMint = 1,
    /// `fee_usdf_ata` did not match the canonical `FEE_USDF_ATA` pinned
    /// in `consts`. Refused so a residual-authority account masquerading
    /// as the treasury can never be picked.
    InvalidFeeAccount = 2,
    /// CPI-target program account did not match the expected program ID.
    InvalidProgram = 3,
    /// User received less USDF/USDC/target than `min_amount_out`.
    SlippageExceeded = 4,
    /// A snapshot-delta CPI (Flipcash sell, USDF→USDC bridge, or USDC→USDF
    /// bridge) did not deliver any output. Returned when the post-CPI
    /// balance equals the pre-CPI balance.
    NoCurveOutput = 5,
    /// `in_amount == 0` rejected. Flipcash interprets zero as "use the
    /// caller's full balance"; we reject it at entry so a malformed router
    /// call can never silently expand into a full-balance trade.
    ZeroAmount = 6,
    /// Bridge pool / vault accounts did not match the canonical USDF↔USDC
    /// bridge pinned in consts. Refused so a non-1:1 pool can't be supplied
    /// by the off-chain caller.
    InvalidBridgePool = 7,
}

error!(FlipdashRouterError);
