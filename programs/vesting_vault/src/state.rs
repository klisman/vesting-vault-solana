use anchor_lang::prelude::*;

use crate::math::{self, claimable};

/// One token grant: linear vesting, optional cliff, optional creator revoke.
///
/// PDA seeds: `[b"vesting", creator, id]`.
/// The associated vault token account is owned by this PDA (no extra keypair).
#[account]
#[derive(InitSpace)]
pub struct Vesting {
    /// Canonical bump for the vesting PDA.
    pub bump: u8,
    /// Wallet that created and funded the grant. May revoke if `revocable`.
    pub creator: Pubkey,
    /// Wallet that may claim vested tokens.
    pub beneficiary: Pubkey,
    /// SPL Token or Token-2022 mint locked in the vault.
    pub mint: Pubkey,
    /// Tokens deposited at creation. Never increases.
    pub total_amount: u64,
    /// Tokens already transferred to the beneficiary.
    pub claimed_amount: u64,
    /// Unix timestamp when the linear schedule starts.
    pub start_ts: i64,
    /// Unix timestamp before which vested amount is zero. May equal `start_ts`.
    pub cliff_ts: i64,
    /// Unix timestamp when the grant is fully vested.
    pub end_ts: i64,
    /// If true, `creator` may call `revoke` and recover unvested tokens.
    pub revocable: bool,
    /// Set once on a successful `revoke`.
    pub revoked: bool,
    /// Vested amount snapshotted at revoke time. Zero until revoked.
    /// After revoke, claimable is capped at this value minus `claimed_amount`.
    pub vested_at_revoke: u64,
    /// Creator-chosen grant id. Part of the PDA seeds.
    pub id: [u8; 32],
}

impl Vesting {
    /// Unlocked tokens at `now`. After revoke this is frozen at the snapshot.
    pub fn currently_vested(&self, now: i64) -> u64 {
        if self.revoked {
            self.vested_at_revoke
        } else {
            math::vested_amount(
                now,
                self.start_ts,
                self.cliff_ts,
                self.end_ts,
                self.total_amount,
            )
        }
    }

    /// Tokens the beneficiary may withdraw at `now`.
    pub fn claimable_amount(&self, now: i64) -> u64 {
        claimable(self.currently_vested(now), self.claimed_amount)
    }

    /// True when the beneficiary has taken every token they are still owed.
    pub fn is_settled(&self) -> bool {
        if self.revoked {
            self.claimed_amount == self.vested_at_revoke
        } else {
            self.claimed_amount == self.total_amount
        }
    }
}
