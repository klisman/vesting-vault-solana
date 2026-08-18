#[path = "common/harness.rs"]
mod harness;

use harness::{
    assert_anchor_error, claim_ix, close_ix, funded_grant, send_ix, set_clock, setup_svm, END,
    START,
};
use solana_signer::Signer;
use vesting_vault::error::VestingError;

#[test]
fn close_after_full_claim_reclaims_rent() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    set_clock(&mut svm, END);
    assert!(send_ix(
        &mut svm,
        &grant.beneficiary,
        claim_ix(
            grant.beneficiary.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
            grant.beneficiary_ata,
        ),
    )
    .is_ok());

    let res = send_ix(
        &mut svm,
        &grant.creator,
        close_ix(
            grant.creator.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
        ),
    );
    assert!(res.is_ok(), "{res:?}");
    assert!(svm.get_account(&grant.vesting).is_none());
    assert!(svm.get_account(&grant.vault).is_none());
}

#[test]
fn close_rejected_while_tokens_remain() {
    let (mut svm, program_id) = setup_svm();
    let grant = funded_grant(&mut svm, &program_id, true);
    set_clock(&mut svm, START + 500);
    let res = send_ix(
        &mut svm,
        &grant.creator,
        close_ix(
            grant.creator.pubkey(),
            grant.vesting,
            grant.mint,
            grant.vault,
        ),
    );
    assert_anchor_error(res, VestingError::VaultNotEmpty);
}
