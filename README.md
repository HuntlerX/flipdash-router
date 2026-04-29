# FlipDash Router

A Solana program that wraps the [Flipcash bonding curve](https://github.com/code-payments/flipcash-program) and the [USDF↔USDC bridge](https://github.com/code-payments/usdf-swap-program) into single user-signed instructions, taking a fixed 0.85% fee in USDF on every swap.

The router is intentionally minimal: no admin functions, no PDA-owned funds, no settable parameters. The fee receiver, fee rate, bridge accounts, and CPI program IDs are all hardcoded in the binary; changing any of them requires redeploying through the upgrade authority.

## Instructions

| Discriminator | Name                       | Flow                                                |
|---:|---|---|
| 1  | `BuyTokensIx`              | USDF → flipcash currency                            |
| 2  | `BuyTokensViaBridgeIx`     | USDC → bridge → USDF → flipcash currency            |
| 3  | `SellTokensIx`             | flipcash currency → USDF                            |
| 4  | `SellTokensViaBridgeIx`    | flipcash currency → USDF → bridge → USDC            |

Each instruction takes the same 16-byte argument: `(in_amount: u64, min_amount_out: u64)`, both little-endian.

## Safety properties

- **No admin paths.** Once deployed, only the on-chain processing logic can move funds, and that logic only moves funds *with the user's signature*.
- **No PDA-owned funds.** The router never holds tokens or lamports; it strictly orchestrates user-signed CPIs.
- **Pinned external accounts.** Bridge program, bridge pool, bridge vaults, fee-receiver USDF ATA, USDF mint, USDC mint, and the Flipcash program ID are all `==`-checked against constants in `api/src/consts.rs`.
- **Snapshot-delta accounting.** All sell paths (and the bridge-buy path) compute fees from the *observed* on-chain delta after each CPI, not from caller-supplied amounts, so the fee accurately reflects what actually moved.
- **Zero-amount sentinel rejected.** Flipcash interprets `in_amount == 0` as "use full balance"; the router rejects it at entry.
- **Slippage check enforced post-CPI.** `min_amount_out` is checked against the user's actual destination balance change, so it covers fee + curve + bridge in one number.

## Build

```bash
make build-mainnet
```

Produces `target/deploy/flipdash_router.so` (~76 KB). The build is **deterministic** — the SHA-256 will match across rebuilds with the same toolchain.

## Verifying the deployed binary

The on-chain bytecode for the mainnet program is reproducible from this repository.

```bash
git clone https://github.com/HuntlerX/flipdash-router
cd flipdash-router
git checkout v0.1.0     # tag of the deployed build
make build-mainnet

solana program dump -u mainnet-beta \
  Dash3ZZKehWHGNvbCpkde6gvJTR2io7YZCt5DyU73PuJ \
  /tmp/flipdash_router_onchain.so

# The program-data account has trailing zero-padding from its rent-exempt
# allocation; truncate to the local ELF length before hashing.
truncate -s $(stat -c%s target/deploy/flipdash_router.so) \
  /tmp/flipdash_router_onchain.so

sha256sum target/deploy/flipdash_router.so /tmp/flipdash_router_onchain.so
```

The two SHA-256 digests must be identical.

## Deploy

```bash
make deploy-mainnet
```

Deploy reads keypairs from `$HOME/.config/flipdash/keys/` (configurable in the `Makefile`). Keypairs are intentionally **not** stored in this repository — see `.gitignore`.

## Cluster

| Cluster      | Program ID                                                |
|---|---|
| **mainnet**  | `Dash3ZZKehWHGNvbCpkde6gvJTR2io7YZCt5DyU73PuJ`            |

The mainnet program is deployed via the Solana BPF Upgradeable Loader. The upgrade authority is retained so bugs can be patched.

## Layout

```
api/         off-chain SDK + on-chain shared types
program/     on-chain program (cdylib)
```

The `api` crate publishes `build_*_ix` builder functions for off-chain callers; the `program` crate is the SBF binary.

## Security disclosures

See [`SECURITY.md`](./SECURITY.md).

## Toolchain

The build uses the Solana SBF toolchain (`cargo build-sbf`) which ships with the [Anza Solana CLI](https://github.com/anza-xyz/agave). Dependencies are pinned in `Cargo.toml` and `Cargo.lock`; both are committed.

## License

[MIT](./LICENSE)
