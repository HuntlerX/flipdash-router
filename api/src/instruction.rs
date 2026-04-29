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
}

instruction!(InstructionType, BuyTokensIx);
instruction!(InstructionType, BuyTokensViaBridgeIx);
instruction!(InstructionType, SellTokensIx);
instruction!(InstructionType, SellTokensViaBridgeIx);

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
