use anchor_lang::prelude::*;

/// PDA seed prefix. Full seeds: `[VESTING_SEED, creator, id]`.
///
/// `id` is a 32-byte grant identifier chosen by the creator so the same wallet
/// can issue many grants (e.g. one per teammate or round).
#[constant]
pub const VESTING_SEED: &[u8] = b"vesting";
