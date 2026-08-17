use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::VESTING_SEED, error::VestingError, state::Vesting};

#[derive(Accounts)]
pub struct Revoke<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        has_one = creator @ VestingError::UnauthorizedCreator,
        has_one = mint,
        seeds = [VESTING_SEED, creator.key().as_ref(), vesting.id.as_ref()],
        bump = vesting.bump
    )]
    pub vesting: Account<'info, Vesting>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// Vault ATA owned by the vesting PDA.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vesting,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = creator,
        associated_token::token_program = token_program
    )]
    pub creator_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_revoke(_ctx: Context<Revoke>) -> Result<()> {
    err!(VestingError::NotYetImplemented)
}
