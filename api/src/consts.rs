use solana_program::{pubkey, pubkey::Pubkey};

/// Treasury wallet that owns the FEE_USDF_ATA. Hardcoded so the fee
/// destination is auditable and immutable per build. Re-deploy to change.
pub const FEE_OWNER: Pubkey = pubkey!("fLipd8evZdFfCiti4DJpjLguSzrdaZfWMRQEBiAVbL5");

/// Fee in basis points. 85 = 0.85%.
pub const FEE_BPS: u16 = 85;
pub const FEE_BPS_DIVISOR: u128 = 10_000;

// ─── External programs we CPI into ──────────────────────────────────────
pub const FLIPCASH_PROGRAM: Pubkey = pubkey!("ccJYP5gjZqcEHaphcxAZvkxCrnTVfYMjyhSYkpQtf8Z");
pub const USDF_SWAP_PROGRAM: Pubkey = pubkey!("usdfcP2V1bh1Lz7Y87pxR4zJd3wnVtssJ6GeSHFeZeu");

// ─── Mints we hardcode in account validation ────────────────────────────
pub const USDF_MINT: Pubkey = pubkey!("5AMAA9JV9H97YYVxx8F6FsCMmTwXSuTTQneiup4RYAUQ");
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

// ─── Canonical USDF↔USDC bridge accounts ────────────────────────────────
// Pinned so a malicious or buggy off-chain caller can't wire in a non-1:1
// pool that would let the router's "fee = compute_fee(args.in_amount)"
// assumption silently consume the user's preexisting USDF.
pub const BRIDGE_POOL: Pubkey = pubkey!("8q2Kv6wMKDhkg92itiYGxr6jvSHvUhuCay6zrhUncyvK");
pub const BRIDGE_USDF_VAULT: Pubkey = pubkey!("FmpZMBbtM2vu7vwmRAAQZa7a6jvQntmmoSYCYWXv4EeX");
pub const BRIDGE_USDC_VAULT: Pubkey = pubkey!("3W6Czwv4iWtvv1heeb7MNK97NqW3PmxNvvYW2vipBdsS");

// ─── Canonical FEE_OWNER USDF ATA ───────────────────────────────────────
// `find_program_address([FEE_OWNER, spl_token::id(), USDF_MINT], ATA_PROGRAM)`.
//
// Pinned (rather than just checking owner+mint) to close a residual-authority
// seam: SPL Token's `SetAuthority::AccountOwner` does *not* clear
// `close_authority` on non-native mints. Without the pin, an attacker could
// create a USDF account, retain close-authority on it, transfer ownership to
// FEE_OWNER, and grief any router caller that happens to be configured for
// that account (e.g. by closing it after a treasury sweep). Pinning the
// canonical ATA via `==` makes that whole class of issues impossible.
pub const FEE_USDF_ATA: Pubkey = pubkey!("GxFJoSyoKA71iuAFNP3fbmATJXYZMdUB2W9rhobPUD3L");

// ─── Instruction-data discriminators of the programs we CPI into ────────
// Single-byte enum discriminators per Steel framework convention.
pub const FLIPCASH_IX_BUY: u8 = 4;
pub const FLIPCASH_IX_SELL: u8 = 5;
pub const USDF_SWAP_IX_SWAP: u8 = 2;
