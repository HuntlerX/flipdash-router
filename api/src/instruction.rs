use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum InstructionType {
    Unknown = 0,
    /// USDF in → flipcash currency out. Fee taken in USDF before curve.
    BuyTokensIx = 1,
    /// USDC in → bridge → USDF in → flipcash currency out. Fee in USDF.
    BuyTokensViaBridgeIx = 2,
    /// flipcash currency in → USDF out. Fee taken in USDF after curve sell.
    SellTokensIx = 3,
    /// flipcash currency in → USDF → bridge → USDC out. Fee in USDF.
    SellTokensViaBridgeIx = 4,
    /// flipcash currency A in → USDF (curve sell) → flipcash currency B out
    /// (curve buy). Single 0.85% fee taken in USDF on the intermediate gross
    /// (vs the 1.7% the user would pay doing the two trades manually).
    CurrencyToCurrencyIx = 5,
    /// Same shape as `BuyTokensViaBridgeIx`, but routes the USDC→USDF leg
    /// through the Coinbase ocp-server stable swapper instead of the
    /// usdf-swap-program. Off-chain selection (indexer picks the bridge
    /// with sufficient USDC liquidity) lets us fall back transparently.
    BuyTokensViaCoinbaseIx = 6,
    /// Same shape as `SellTokensViaBridgeIx`, but routes the USDF→USDC leg
    /// through the Coinbase ocp-server stable swapper.
    SellTokensViaCoinbaseIx = 7,
}

instruction!(InstructionType, BuyTokensIx);
instruction!(InstructionType, BuyTokensViaBridgeIx);
instruction!(InstructionType, SellTokensIx);
instruction!(InstructionType, SellTokensViaBridgeIx);
instruction!(InstructionType, CurrencyToCurrencyIx);
instruction!(InstructionType, BuyTokensViaCoinbaseIx);
instruction!(InstructionType, SellTokensViaCoinbaseIx);

// All four ixs share the same wire layout: total in_amount + slippage min.
//
// Buy ixs:
//   in_amount      = the user's USDF (Buy) or USDC (BuyViaBridge) input
//   min_amount_out = worst-case currency tokens the user must receive
//
// Sell ixs:
//   in_amount      = the currency tokens the user is selling
//   min_amount_out = worst-case USDF (Sell) or USDC (SellViaBridge) the user
//                    must receive *after* the FlipDash fee.
//
// The slippage check on min_amount_out is done by the router on the user's
// destination ATA (snapshot delta), so it covers fee + curve + bridge in
// one number — what the user sees in the UI.

#[derive(Debug)]
pub struct ParsedSwapArgs {
    pub in_amount: u64,
    pub min_amount_out: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BuyTokensIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BuyTokensViaBridgeIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SellTokensIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SellTokensViaBridgeIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

// Currency↔currency uses the same wire layout. `in_amount` is currency-A
// quarks the user is selling. `min_amount_out` is the worst-case
// currency-B quarks the user must end up with (snapshot delta on the
// dst ATA, i.e. covers the curve-sell-fee + router-fee + curve-buy
// slippage in one number — what the UI shows).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CurrencyToCurrencyIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

// Same wire layout as the bridge variants — the only difference is the
// program / pool / vaults the router CPIs into for the USDC↔USDF leg.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BuyTokensViaCoinbaseIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SellTokensViaCoinbaseIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

macro_rules! impl_swap_args {
    ($t:ty) => {
        impl $t {
            pub fn from_struct(parsed: ParsedSwapArgs) -> Self {
                Self {
                    in_amount: parsed.in_amount.to_le_bytes(),
                    min_amount_out: parsed.min_amount_out.to_le_bytes(),
                }
            }
            pub fn to_struct(&self) -> ParsedSwapArgs {
                ParsedSwapArgs {
                    in_amount: u64::from_le_bytes(self.in_amount),
                    min_amount_out: u64::from_le_bytes(self.min_amount_out),
                }
            }
        }
    };
}

impl_swap_args!(BuyTokensIx);
impl_swap_args!(BuyTokensViaBridgeIx);
impl_swap_args!(SellTokensIx);
impl_swap_args!(SellTokensViaBridgeIx);
impl_swap_args!(CurrencyToCurrencyIx);
impl_swap_args!(BuyTokensViaCoinbaseIx);
impl_swap_args!(SellTokensViaCoinbaseIx);
