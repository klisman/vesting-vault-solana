use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::VESTING_SEED, error::VestingError, state::Vesting};

#[derive(AnchorSerialize, AnchorDeserialize)]
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
    _ctx: Context<CreateVesting>,
    _id: [u8; 32],
    _params: CreateVestingParams,
) -> Result<()> {
    err!(VestingError::NotYetImplemented)
}
