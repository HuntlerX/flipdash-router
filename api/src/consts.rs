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

// ─── Coinbase ocp-server stable swapper (alternate USDF↔USDC bridge) ────
// Open-access (no real whitelist enforcement on swap), 1:1, fee_rate=0
// today. Pinned here so we still pin the program/pool/vaults as a
// defense-in-depth posture even if their admin upgrades the program
// behind the same ID.
pub const COINBASE_PROGRAM: Pubkey = pubkey!("pqgqKahpG1y2wsgxFhzaAnkV1cL9vk8MSg9qm4q646F");
pub const COINBASE_POOL: Pubkey = pubkey!("CrDL9SoCyW1tBgn8k7rgGSpWhnszneWDbvKvqPAU4PL9");
pub const COINBASE_USDC_VAULT: Pubkey = pubkey!("2bQv8iFVXm9Z6wJk7KMFhhtLegNFZPtcDeJc5qrwJNqZ");
pub const COINBASE_USDC_VAULT_ACCT: Pubkey = pubkey!("YioohQk1msG36osqTZ9bUG9GwaygVpq9ACQ7gUrtUHr");
pub const COINBASE_USDF_VAULT: Pubkey = pubkey!("3vxe5BnJUWNz3kgSLXKaGuibTnjofxgGuAjhpMeEq95s");
pub const COINBASE_USDF_VAULT_ACCT: Pubkey = pubkey!("ZR8euZnAt7duoF7PfEqkq6ZqFJmaLQzKqEWAmozH4uq");
pub const COINBASE_FEE_RECIPIENT: Pubkey = pubkey!("4ZnFXk7KyB5khDqjWSHqHBQH1nQCnmvkr1pRFivWcP7e");
// fee_rate is 0 today; we pin both ATAs (one per direction) so a future
// fee_rate>0 + admin migration can't redirect fees to a non-canonical
// account from our caller's perspective. Derived via
// `find_program_address([fee_recipient, spl_token::id(), MINT], ATA_PROGRAM)`.
pub const COINBASE_FEE_RECIP_USDC_ATA: Pubkey = pubkey!("J3cK8ie2msk96idNt2d2wDofWSM8CrLarSZTHnGMPXsQ");
pub const COINBASE_FEE_RECIP_USDF_ATA: Pubkey = pubkey!("oupWgJfwWu8Nbut4n3trqp8zqUQqTkeLBTiDZEKoobp");
pub const COINBASE_WHITELIST: Pubkey = pubkey!("24UrnpmHQWUgTYjovHWySFg1JT4AXUCUp4Ly25C25GNj");
// Associated Token Account program — required by Coinbase's swap ix at
// the end of its account list. The existing usdf-swap path doesn't need
// this account.
pub const ATA_PROGRAM: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
// Anchor instruction discriminator (sha256("global:swap")[:8]).
pub const COINBASE_IX_SWAP_DISC: [u8; 8] = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8];
