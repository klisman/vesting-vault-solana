#[path = "common/harness.rs"]
mod harness;

use harness::{
    assert_anchor_error, claim_ix, funded_grant, load_vesting, send_ix, set_clock, setup_svm,
    token_amount, CLIFF, END, START, TOTAL,
};
use solana_keypair::Keypair;
use solana_signer::Signer;
use vesting_vault::error::VestingError;

#[test]
fn claim_before_cliff_is_zero() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    set_clock(&mut svm, CLIFF - 1);
    let res = send_ix(
        &mut svm,
        &grant.beneficiary,
        claim_ix(
            grant.beneficiary.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
            grant.beneficiary_ata,
        ),
    );
    assert_anchor_error(res, VestingError::NothingToClaim);
}

#[test]
fn claim_at_midpoint_after_cliff() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    let now = START + 500;
    set_clock(&mut svm, now);
    let res = send_ix(
        &mut svm,
        &grant.beneficiary,
        claim_ix(
            grant.beneficiary.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
            grant.beneficiary_ata,
        ),
    );
    assert!(res.is_ok(), "{res:?}");

    let expected = TOTAL / 2;
    assert_eq!(token_amount(&svm, &grant.beneficiary_ata), expected);
    assert_eq!(token_amount(&svm, &grant.vault), TOTAL - expected);
    let state = load_vesting(&svm, &grant.vesting);
    assert_eq!(state.claimed_amount, expected);
}

#[test]
fn claim_after_end_unlocks_all() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    set_clock(&mut svm, END);
    let res = send_ix(
        &mut svm,
        &grant.beneficiary,
        claim_ix(
            grant.beneficiary.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
            grant.beneficiary_ata,
        ),
    );
    assert!(res.is_ok(), "{res:?}");
    assert_eq!(token_amount(&svm, &grant.beneficiary_ata), TOTAL);
    assert_eq!(token_amount(&svm, &grant.vault), 0);
}

#[test]
fn double_claim_without_time_passing_fails() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    set_clock(&mut svm, END);
    let ix = claim_ix(
        grant.beneficiary.pubkey(),
        grant.vesting,
        grant.mint,
        grant.vault,
        grant.beneficiary_ata,
    );
    assert!(send_ix(&mut svm, &grant.beneficiary, ix.clone()).is_ok());
    let res = send_ix(&mut svm, &grant.beneficiary, ix);
    assert_anchor_error(res, VestingError::NothingToClaim);
}

#[test]
fn unauthorized_claim_is_rejected() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 10_000_000_000).unwrap();
    set_clock(&mut svm, END);
    let attacker_ata = harness::ata(&attacker.pubkey(), &grant.mint);
    let res = send_ix(
        &mut svm,
        &attacker,
        claim_ix(
            attacker.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
            attacker_ata,
        ),
    );
    assert_anchor_error(res, VestingError::UnauthorizedBeneficiary);
}
