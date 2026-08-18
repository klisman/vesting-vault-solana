use anchor_lang::prelude::*;

/// PDA seed prefix. Full seeds: `[VESTING_SEED, creator, id]`.
///
/// `id` is a 32-byte grant identifier chosen by the creator so the same wallet
/// can issue many grants (e.g. one per teammate or round).
#[constant]
pub const VESTING_SEED: &[u8] = b"vesting";

/// Seeds used to sign token CPIs as the vesting PDA.
pub fn vesting_signer_seeds<'a>(
    creator: &'a Pubkey,
    id: &'a [u8; 32],
    bump: &'a [u8],
) -> [&'a [u8]; 4] {
    [VESTING_SEED, creator.as_ref(), id.as_ref(), bump]
}
