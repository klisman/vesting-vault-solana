# Security Notes

This project is an educational vesting vault and is not a mainnet audit.

## Program invariants

- `creator` and `beneficiary` are signers for privileged actions through Anchor
  `Signer` accounts and `has_one` constraints.
- Each grant is a PDA derived from `["vesting", creator, id]`.
- The vault ATA is owned by the vesting PDA. Token transfers out of the vault
  require the PDA's canonical signer seeds; there is no admin key or backdoor.
- Creation requires `start_ts <= cliff_ts < end_ts`, a positive amount, and
  sufficient creator balance.
- Vesting uses checked arithmetic with `u128` intermediates for
  `total_amount * (now - start_ts) / (end_ts - start_ts)`.
- A revoke snapshots the vested amount. It returns only unvested tokens, so it
  cannot claw back tokens that were already vested or claimed.
- Token-2022 mints with `TransferFeeConfig` are rejected because this version
  requires transfers to be accounted for 1:1.

## Scope limitations

This version does not support streaming fees, clawback of already-claimed
tokens, arbitrary Token-2022 transfer hooks, or a production mainnet audit.
Keep test keys and local fixtures out of committed files. Client transactions
should be simulated before a wallet signs them.
