use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::VESTING_SEED, error::VestingError, state::Vesting};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(
        mut,
        has_one = beneficiary @ VestingError::UnauthorizedBeneficiary,
        has_one = mint,
        seeds = [VESTING_SEED, vesting.creator.as_ref(), vesting.id.as_ref()],
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

    /// Beneficiary ATA. `init_if_needed` is safe here: the address is the
    /// canonical ATA for (beneficiary, mint, token_program), so it cannot be
    /// redirected to an attacker-owned account.
    #[account(
        init_if_needed,
        payer = beneficiary,
        associated_token::mint = mint,
        associated_token::authority = beneficiary,
        associated_token::token_program = token_program
    )]
    pub beneficiary_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_claim(_ctx: Context<Claim>) -> Result<()> {
    err!(VestingError::NotYetImplemented)
}
