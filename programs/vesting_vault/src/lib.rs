pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2p4En7X5pMCAwuX16MjN9tqHLbh8H6DGYpwmdNupg9Y8");

#[program]
pub mod vesting_vault {
    use super::*;

    /// Create a vesting grant, initialize the vault ATA, and lock `total_amount`.
    pub fn create_vesting(
        ctx: Context<CreateVesting>,
        id: [u8; 32],
        params: CreateVestingParams,
    ) -> Result<()> {
        crate::instructions::create_vesting::handle_create_vesting(ctx, id, params)
    }

    /// Beneficiary withdraws tokens that have vested since the last claim.
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        crate::instructions::claim::handle_claim(ctx)
    }

    /// Creator recovers unvested tokens. Already-vested amounts stay claimable.
    pub fn revoke(ctx: Context<Revoke>) -> Result<()> {
        crate::instructions::revoke::handle_revoke(ctx)
    }

    /// Close an empty grant and reclaim rent.
    pub fn close(ctx: Context<Close>) -> Result<()> {
        crate::instructions::close::handle_close(ctx)
    }
}
