#[path = "common/harness.rs"]
mod harness;

use anchor_lang::prelude::Pubkey;
use harness::{
    assert_anchor_error, ata, create_vesting_ix, default_params, insert_mint, insert_token_account,
    load_vesting, send_ix, set_clock, setup_svm, token_amount, vesting_pda, GRANT_ID, START, TOTAL,
};
use solana_keypair::Keypair;
use solana_signer::Signer;
use vesting_vault::{error::VestingError, CreateVestingParams};

#[test]
fn create_locks_tokens_in_vault() {
    let (mut svm, program_id) = setup_svm();
    let creator = Keypair::new();
    let beneficiary = Keypair::new();
    svm.airdrop(&creator.pubkey(), 10_000_000_000).unwrap();

    let mint = Pubkey::new_unique();
    insert_mint(&mut svm, mint, TOTAL);
    let (vesting, bump) = vesting_pda(&program_id, &creator.pubkey(), &GRANT_ID);
    let vault = ata(&vesting, &mint);
    let creator_ata = ata(&creator.pubkey(), &mint);
    insert_token_account(&mut svm, creator_ata, mint, creator.pubkey(), TOTAL);

    set_clock(&mut svm, START);
    let res = send_ix(
        &mut svm,
        &creator,
        create_vesting_ix(
            creator.pubkey(),
            beneficiary.pubkey(),
            mint,
            vesting,
            vault,
            creator_ata,
            GRANT_ID,
            default_params(true),
        ),
    );
    assert!(res.is_ok(), "{res:?}");

    let state = load_vesting(&svm, &vesting);
    assert_eq!(state.creator, creator.pubkey());
    assert_eq!(state.beneficiary, beneficiary.pubkey());
    assert_eq!(state.mint, mint);
    assert_eq!(state.total_amount, TOTAL);
    assert_eq!(state.claimed_amount, 0);
    assert_eq!(state.bump, bump);
    assert_eq!(state.id, GRANT_ID);
    assert!(state.revocable);
    assert!(!state.revoked);
    assert_eq!(token_amount(&svm, &vault), TOTAL);
    assert_eq!(token_amount(&svm, &creator_ata), 0);
}

#[test]
fn create_rejects_zero_amount() {
    let (mut svm, program_id) = setup_svm();
    let creator = Keypair::new();
    let beneficiary = Keypair::new();
    svm.airdrop(&creator.pubkey(), 10_000_000_000).unwrap();
    let mint = Pubkey::new_unique();
    insert_mint(&mut svm, mint, TOTAL);
    let (vesting, _) = vesting_pda(&program_id, &creator.pubkey(), &GRANT_ID);
    let vault = ata(&vesting, &mint);
    let creator_ata = ata(&creator.pubkey(), &mint);
    insert_token_account(&mut svm, creator_ata, mint, creator.pubkey(), TOTAL);

    let mut params = default_params(true);
    params.total_amount = 0;
    set_clock(&mut svm, START);
    let res = send_ix(
        &mut svm,
        &creator,
        create_vesting_ix(
            creator.pubkey(),
            beneficiary.pubkey(),
            mint,
            vesting,
            vault,
            creator_ata,
            GRANT_ID,
            params,
        ),
    );
    assert_anchor_error(res, VestingError::InvalidAmount);
}

#[test]
fn create_rejects_invalid_schedule() {
    let (mut svm, program_id) = setup_svm();
    let creator = Keypair::new();
    let beneficiary = Keypair::new();
    svm.airdrop(&creator.pubkey(), 10_000_000_000).unwrap();
    let mint = Pubkey::new_unique();
    insert_mint(&mut svm, mint, TOTAL);
    let (vesting, _) = vesting_pda(&program_id, &creator.pubkey(), &GRANT_ID);
    let vault = ata(&vesting, &mint);
    let creator_ata = ata(&creator.pubkey(), &mint);
    insert_token_account(&mut svm, creator_ata, mint, creator.pubkey(), TOTAL);

    let params = CreateVestingParams {
        start_ts: START,
        cliff_ts: START - 1,
        end_ts: START + 10,
        total_amount: TOTAL,
        revocable: true,
    };
    set_clock(&mut svm, START);
    let res = send_ix(
        &mut svm,
        &creator,
        create_vesting_ix(
            creator.pubkey(),
            beneficiary.pubkey(),
            mint,
            vesting,
            vault,
            creator_ata,
            GRANT_ID,
            params,
        ),
    );
    assert_anchor_error(res, VestingError::InvalidSchedule);
}
