# Vesting Vault

Time-locked SPL token grants on Solana: linear vesting, an optional cliff, and creator revoke.

The on-chain program implements create, claim, revoke, and close. Day 3 adds
an IDL-generated Kit client and Surfpool integration tests for time-dependent
flows. A web app and live devnet deploy land in follow-up PRs.

## Stack

| Tool | Version |
| --- | --- |
| Anchor | 1.1.2 |
| Solana CLI | 3.1.10 |
| Rust | 1.89.0 (see `rust-toolchain.toml`) |
| Node.js | >= 20.18 |

## Build

```bash
anchor build --ignore-keys
```

## Test

```bash
cargo fmt --all -- --check
anchor build --ignore-keys
cargo clippy --workspace --all-targets -- -D warnings
cargo test
```

`cargo test` needs a prior `anchor build` so LiteSVM can load `target/deploy/vesting_vault.so`.

## Generate the Kit client

```bash
anchor build --ignore-keys
npm install
npm run generate:client
```

Codama reads `target/idl/vesting_vault.json` and writes the typed client to
`clients/ts/src/generated/`. Do not hand-write Borsh layouts.

## Surfpool integration tests

```bash
npm run test:integration
```

The suite starts an embedded Surfnet, deploys the local program, advances
Clock through the cliff/midpoint/end, and tests the revoke snapshot flow.

## Program ID (localnet / planned devnet)

```
2p4En7X5pMCAwuX16MjN9tqHLbh8H6DGYpwmdNupg9Y8
```

Not deployed yet.

## License

MIT
