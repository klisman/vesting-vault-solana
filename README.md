# Vesting Vault

Time-locked SPL token grants on Solana: linear vesting, an optional cliff, and creator revoke.

This repository is a work in progress. The on-chain program implements create, claim, revoke, and close. A web app and a live devnet deploy land in follow-up PRs.

## Stack

| Tool | Version |
| --- | --- |
| Anchor | 1.1.2 |
| Solana CLI | 3.1.10 |
| Rust | 1.89.0 (see `rust-toolchain.toml`) |
| Node.js | >= 20.18 (web app, later) |

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

## Program ID (localnet / planned devnet)

```
2p4En7X5pMCAwuX16MjN9tqHLbh8H6DGYpwmdNupg9Y8
```

Not deployed yet.

## License

MIT
