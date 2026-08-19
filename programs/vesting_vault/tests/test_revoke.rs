#[path = "common/harness.rs"]
mod harness;

use harness::{
    assert_anchor_error, claim_ix, funded_grant, insert_token_account, load_vesting, revoke_ix,
    send_ix, set_clock, setup_svm, token_amount, START, TOTAL,
};
use solana_keypair::Keypair;
use solana_signer::Signer;
use vesting_vault::error::VestingError;

#[test]
fn revoke_returns_unvested_and_leaves_vested_claimable() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    let now = START + 500;
    set_clock(&mut svm, now);

    let res = send_ix(
        &mut svm,
        &grant.creator,
        revoke_ix(
            grant.creator.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
            grant.creator_ata,
        ),
    );
    assert!(res.is_ok(), "{res:?}");

    let vested = TOTAL / 2;
    assert_eq!(token_amount(&svm, &grant.creator_ata), TOTAL - vested);
    assert_eq!(token_amount(&svm, &grant.vault), vested);

    let state = load_vesting(&svm, &grant.vesting);
    assert!(state.revoked);
    assert_eq!(state.vested_at_revoke, vested);

    // Time passing after revoke must not unlock more tokens.
    set_clock(&mut svm, now + 10_000);
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
    assert_eq!(token_amount(&svm, &grant.beneficiary_ata), vested);
    assert_eq!(token_amount(&svm, &grant.vault), 0);
}

#[test]
fn revoke_rejected_when_not_revocable() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, false);
    set_clock(&mut svm, START + 500);
    let res = send_ix(
        &mut svm,
        &grant.creator,
        revoke_ix(
            grant.creator.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
            grant.creator_ata,
        ),
    );
    assert_anchor_error(res, VestingError::NotRevocable);
}

#[test]
fn unauthorized_revoke_is_rejected() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 10_000_000_000).unwrap();
    set_clock(&mut svm, START + 500);
    let attacker_ata = harness::ata(&attacker.pubkey(), &grant.mint);
    insert_token_account(&mut svm, attacker_ata, grant.mint, attacker.pubkey(), 0);
    let res = send_ix(
        &mut svm,
        &attacker,
        revoke_ix(
            attacker.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
            attacker_ata,
        ),
    );
    assert_anchor_error(res, VestingError::UnauthorizedCreator);
}

#[test]
fn revoke_twice_is_rejected() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    set_clock(&mut svm, START + 500);
    let ix = revoke_ix(
        grant.creator.pubkey(),
        grant.vesting,
        grant.mint,
        grant.vault,
        grant.creator_ata,
    );
    assert!(send_ix(&mut svm, &grant.creator, ix.clone()).is_ok());
    let res = send_ix(&mut svm, &grant.creator, ix);
    assert_anchor_error(res, VestingError::AlreadyRevoked);
}
