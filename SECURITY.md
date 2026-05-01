# Security Policy

## Reporting a vulnerability

If you believe you have found a vulnerability in `flipdash-router`, please **report it privately first** rather than opening a public GitHub issue.

- **Primary contact:** [@HuntlerX on X](https://x.com/HuntlerX) — DM with a brief summary; we will respond and provide a secure channel for technical details.
- **Acknowledgment:** within 48 hours of report.
- **First triage update:** within 5 business days.
- **Coordinated disclosure:** we ask for ~30 days from acknowledgment before public disclosure of the report, longer if a fix requires an on-chain upgrade.

Please **do not**:
- File a public GitHub issue with exploit details
- Disclose findings on social media or chat platforms before we have responded
- Test exploits against mainnet beyond the minimum needed to demonstrate impact (use devnet or a forked validator where possible)

## Scope

### In scope

| Component | Address |
|---|---|
| `flipdash-router` mainnet program | `Dash3ZZKehWHGNvbCpkde6gvJTR2io7YZCt5DyU73PuJ` |
| Source code in this repository | the commit/tag whose build matches the deployed bytecode (see "Verifying the deployed binary" below) |

Bugs of interest:
- Anything that lets a third party gain value they shouldn't (theft from another user, drain of router-owned funds, manipulation of fee accounting)
- Anything that lets a malicious caller make the program behave differently from the documented instruction semantics (account-substitution, fee-sink redirection, slippage bypass, snapshot-delta manipulation)
- Vulnerabilities in the program's CPI integration with `flipcash-program` or the USDF↔USDC bridge that originate in this router's logic
- Off-by-one or boundary errors in the fixed-point arithmetic, fee math, or pinned account checks
- Any way to brick the program (DoS) at the protocol level

### Out of scope

- The frontend at `flipdash.cash` and the off-chain indexer (separate project, separate scope)
- Vulnerabilities in upstream programs we CPI into (`flipcash-program` at `ccJYP5gjZqcEHaphcxAZvkxCrnTVfYMjyhSYkpQtf8Z`, `usdf-swap-program` at `usdfcP2V1bh1Lz7Y87pxR4zJd3wnVtssJ6GeSHFeZeu`) — please report those to their respective maintainers
- Generic Solana ecosystem issues (RPC node behavior, validator economics, MEV) that aren't router-specific
- Wallet UX / Phantom decoder issues that don't trace back to a defect in this program's instruction layout
- Operational concerns (key rotation, backup procedures, infra) unless they reveal a code defect

## Severity classification

| Severity | Definition | Examples |
|---|---|---|
| **Critical** | Direct theft of user or treasury funds; full bypass of slippage; ability to drain a flipcash currency pool through this router | Account substitution that lets attacker keep proceeds, hidden authority on fee sink that lets attacker close-and-claim |
| **High** | User-funds loss requiring narrow conditions; cross-asset confusion; sustained DoS of a router instruction | A specific input pattern that lets attacker take more than 0.85% fee from victim |
| **Medium** | User-fairness issues; griefing vectors that cost attacker meaningfully but harm victims at scale; off-by-one in slippage/fee math | Edge case that lets attacker overcharge fees in narrow conditions; weakened slippage tolerance |
| **Low** | Code-quality / hardening issues without a current exploit path | Pinning improvements, documentation gaps, residual authority seams |
| **Informational** | Doesn't affect security but worth noting | Naming, comments, build determinism |

## What you can expect from us

- Acknowledgment of receipt
- Triage and severity classification
- A patch plan with timing if the report is valid
- Public credit (with your permission) once disclosed

## Verifying the deployed binary

The on-chain program at `Dash3ZZKehWHGNvbCpkde6gvJTR2io7YZCt5DyU73PuJ` is reproducible from this repository. Anyone can verify by building from the tagged release commit and comparing against the on-chain program data:

```bash
git clone https://github.com/HuntlerX/flipdash-router
cd flipdash-router
git checkout v0.3.0     # tag of the deployed build
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

The two SHA-256 digests must be identical. The build is deterministic across clean rebuilds with the same toolchain.

## Upgrade policy

The program is deployed via the BPF Upgradeable Loader. The upgrade authority retains the ability to patch bugs and ship improvements. The intent is to:

- Remain upgradeable for the foreseeable future to allow bug fixes
- Document any upgrade in the repository history alongside the source change
- Communicate planned upgrades publicly before they ship

There is no current commitment to freeze the upgrade authority.

## Audit history

The program has gone through multiple internal adversarial review rounds prior to mainnet deployment. Internal audit logs are kept privately. Independent verification is welcome — the source is reproducible, instruction semantics are documented in `program/src/instruction/*.rs`, and CPI helpers in `program/src/cpi.rs` show every external call this program makes.

If you would like to perform a formal audit and publish the results, please reach out via the contact above.

## License

The project is released under the [MIT License](./LICENSE). Security reports are not subject to the license; you may reproduce relevant code excerpts in your report.
