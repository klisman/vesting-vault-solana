use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::VESTING_SEED, error::VestingError, state::Vesting};

#[derive(Accounts)]
pub struct Close<'info> {
    /// Receives rent from the closed vesting account. Must be the creator.
    #[account(mut, address = vesting.creator @ VestingError::UnauthorizedCreator)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        close = creator,
        has_one = creator,
        has_one = mint,
        seeds = [VESTING_SEED, creator.key().as_ref(), vesting.id.as_ref()],
        bump = vesting.bump
    )]
    pub vesting: Account<'info, Vesting>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// Vault ATA owned by the vesting PDA. Must be empty before close (Day 2).
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vesting,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_close(_ctx: Context<Close>) -> Result<()> {
    err!(VestingError::NotYetImplemented)
}
