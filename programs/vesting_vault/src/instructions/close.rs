use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, CloseAccount, Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{vesting_signer_seeds, VESTING_SEED},
    error::VestingError,
    state::Vesting,
};

#[derive(Accounts)]
pub struct Close<'info> {
    /// Receives rent from the closed vesting account and vault ATA.
    #[account(mut, address = vesting.creator @ VestingError::UnauthorizedCreator)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        close = creator,
        has_one = creator,
        has_one = mint,
        seeds = [VESTING_SEED, vesting.creator.as_ref(), vesting.id.as_ref()],
        bump = vesting.bump
    )]
    pub vesting: Account<'info, Vesting>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// Vault ATA owned by the vesting PDA. Closed after it is confirmed empty.
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

pub fn handle_close(ctx: Context<Close>) -> Result<()> {
    require!(ctx.accounts.vault.amount == 0, VestingError::VaultNotEmpty);
    require!(
        ctx.accounts.vesting.is_settled(),
        VestingError::GrantNotSettled
    );

    let bump = [ctx.accounts.vesting.bump];
    let seeds = vesting_signer_seeds(
        &ctx.accounts.vesting.creator,
        &ctx.accounts.vesting.id,
        &bump,
    );

    token_interface::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.creator.to_account_info(),
            authority: ctx.accounts.vesting.to_account_info(),
        },
        &[&seeds],
    ))?;

    Ok(())
}
