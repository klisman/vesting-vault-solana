use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{constants::VESTING_SEED, error::VestingError, state::Vesting};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateVestingParams {
    pub start_ts: i64,
    pub cliff_ts: i64,
    pub end_ts: i64,
    pub total_amount: u64,
    pub revocable: bool,
}

#[derive(Accounts)]
#[instruction(id: [u8; 32])]
pub struct CreateVesting<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: Only the pubkey is stored on `Vesting`; the beneficiary does not
    /// need to sign or exist as a system account at creation time.
    pub beneficiary: UncheckedAccount<'info>,

    /// Rejected if the mint has Token-2022 `TransferFeeConfig`. Fee-on-transfer
    /// would make `total_amount` diverge from the vault balance.
    #[account(
        constraint = !crate::token_ext::has_transfer_fee(&mint.to_account_info())
            @ VestingError::TransferFeeNotSupported
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// Vesting grant PDA. Seeds: `["vesting", creator, id]`.
    #[account(
        init,
        payer = creator,
        space = 8 + Vesting::INIT_SPACE,
        seeds = [VESTING_SEED, creator.key().as_ref(), id.as_ref()],
        bump
    )]
    pub vesting: Account<'info, Vesting>,

    /// Vault ATA. Token authority is the vesting PDA — no extra signer key.
    #[account(
        init,
        payer = creator,
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
    pub system_program: Program<'info, System>,
}

pub fn handle_create_vesting(
    ctx: Context<CreateVesting>,
    id: [u8; 32],
    params: CreateVestingParams,
) -> Result<()> {
    require!(params.total_amount > 0, VestingError::InvalidAmount);
    require!(
        params.start_ts <= params.cliff_ts && params.cliff_ts < params.end_ts,
        VestingError::InvalidSchedule
    );
    require!(
        ctx.accounts.creator_ata.amount >= params.total_amount,
        VestingError::InsufficientFunds
    );

    let vesting = &mut ctx.accounts.vesting;
    vesting.bump = ctx.bumps.vesting;
    vesting.creator = ctx.accounts.creator.key();
    vesting.beneficiary = ctx.accounts.beneficiary.key();
    vesting.mint = ctx.accounts.mint.key();
    vesting.total_amount = params.total_amount;
    vesting.claimed_amount = 0;
    vesting.start_ts = params.start_ts;
    vesting.cliff_ts = params.cliff_ts;
    vesting.end_ts = params.end_ts;
    vesting.revocable = params.revocable;
    vesting.revoked = false;
    vesting.vested_at_revoke = 0;
    vesting.id = id;

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.creator_ata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
            },
        ),
        params.total_amount,
        ctx.accounts.mint.decimals,
    )?;

    Ok(())
}
