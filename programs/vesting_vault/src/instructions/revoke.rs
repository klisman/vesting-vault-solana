use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    constants::{vesting_signer_seeds, VESTING_SEED},
    error::VestingError,
    math,
    state::Vesting,
};

#[derive(Accounts)]
pub struct Revoke<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        has_one = creator @ VestingError::UnauthorizedCreator,
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

pub fn handle_revoke(ctx: Context<Revoke>) -> Result<()> {
    let vesting = &ctx.accounts.vesting;
    require!(vesting.revocable, VestingError::NotRevocable);
    require!(!vesting.revoked, VestingError::AlreadyRevoked);

    let now = Clock::get()?.unix_timestamp;
    let vested = math::vested_amount(
        now,
        vesting.start_ts,
        vesting.cliff_ts,
        vesting.end_ts,
        vesting.total_amount,
    );
    let unvested = vesting.total_amount.saturating_sub(vested);

    if unvested > 0 {
        let bump = [vesting.bump];
        let seeds = vesting_signer_seeds(&vesting.creator, &vesting.id, &bump);
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                TransferChecked {
                    from: ctx.accounts.vault.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.creator_ata.to_account_info(),
                    authority: ctx.accounts.vesting.to_account_info(),
                },
                &[&seeds],
            ),
            unvested,
            ctx.accounts.mint.decimals,
        )?;
    }

    let vesting = &mut ctx.accounts.vesting;
    vesting.revoked = true;
    vesting.vested_at_revoke = vested;
    Ok(())
}
