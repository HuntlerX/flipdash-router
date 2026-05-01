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
    /// Coinbase pool accounts did not match the pinned constants
    /// (pool / vault / vault-token / fee-recipient / fee-recipient-ATA /
    /// whitelist). Same defense-in-depth as `InvalidBridgePool`.
    InvalidCoinbasePool = 8,
    /// The Coinbase CPI (a closed-source upgradable program with
    /// signer + writable on the user's source ATA) debited *more* than
    /// the user authorized via `in_amount` — or didn't debit exactly
    /// that amount. Defense against a malicious pool-program upgrade
    /// that could otherwise drain extra source tokens while crediting
    /// just enough on the destination side to pass the slippage check.
    SourceOverDebit = 9,
}

error!(FlipdashRouterError);
