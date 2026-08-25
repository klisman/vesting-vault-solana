//! Linear vesting math. Pure functions so they can be unit-tested without the SVM.
//!
//! Schedule (enforced at create): `start_ts <= cliff_ts < end_ts`.
//! The cliff is a gate: nothing unlocks before it, then the amount jumps onto
//! the linear-from-start curve (classic employee / investor grant).

/// Tokens unlocked at `now` for a grant that has not been revoked.
pub fn vested_amount(now: i64, start_ts: i64, cliff_ts: i64, end_ts: i64, total: u64) -> u64 {
    if now < cliff_ts {
        return 0;
    }
    if now >= end_ts {
        return total;
    }

    let elapsed = u128::try_from(i128::from(now) - i128::from(start_ts)).unwrap_or(0);
    let duration = match u128::try_from(i128::from(end_ts) - i128::from(start_ts)) {
        Ok(d) if d > 0 => d,
        _ => return total,
    };

    let vested = u128::from(total)
        .saturating_mul(elapsed)
        .checked_div(duration)
        .unwrap_or(u128::from(total));
    vested.min(u128::from(total)) as u64
}

/// Remaining tokens the beneficiary may withdraw.
pub fn claimable(vested: u64, claimed: u64) -> u64 {
    vested.saturating_sub(claimed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: i64 = 1_000;
    const END: i64 = 2_000;
    const TOTAL: u64 = 1_000_000;

    #[test]
    fn nothing_before_cliff() {
        let cliff = START + 100;
        assert_eq!(vested_amount(START, START, cliff, END, TOTAL), 0);
        assert_eq!(vested_amount(cliff - 1, START, cliff, END, TOTAL), 0);
    }

    #[test]
    fn at_cliff_catches_up_to_linear_from_start() {
        let cliff = START + 100;
        // 100 / 1000 of the grant unlocks the instant the cliff passes.
        assert_eq!(vested_amount(cliff, START, cliff, END, TOTAL), 100_000);
    }

    #[test]
    fn midpoint_is_half_when_cliff_equals_start() {
        assert_eq!(
            vested_amount(START + 500, START, START, END, TOTAL),
            500_000
        );
    }

    #[test]
    fn fully_vested_at_and_after_end() {
        assert_eq!(vested_amount(END, START, START, END, TOTAL), TOTAL);
        assert_eq!(vested_amount(END + 10_000, START, START, END, TOTAL), TOTAL);
    }

    #[test]
    fn claimable_subtracts_already_claimed() {
        assert_eq!(claimable(500, 200), 300);
        assert_eq!(claimable(500, 500), 0);
        assert_eq!(claimable(200, 500), 0);
    }

    #[test]
    fn overflow_scale_total_does_not_wrap() {
        let total = u64::MAX;
        let now = START + 500;
        let vested = vested_amount(now, START, START, END, total);
        // Exactly half of u64::MAX, floored: (u64::MAX * 500) / 1000
        let expected = ((u128::from(total) * 500) / 1000) as u64;
        assert_eq!(vested, expected);
        assert!(vested > 0);
        assert!(vested < total);
    }

    #[test]
    fn extreme_i64_schedule_does_not_saturate_duration() {
        let start = i64::MIN;
        let end = i64::MAX;
        let now = 0;
        let vested = vested_amount(now, start, start, end, TOTAL);
        let elapsed = u128::try_from(i128::from(now) - i128::from(start)).unwrap();
        let duration = u128::try_from(i128::from(end) - i128::from(start)).unwrap();
        let expected = (u128::from(TOTAL) * elapsed / duration) as u64;
        assert_eq!(vested, expected);
        // i64 saturating subtraction would collapse duration to i64::MAX and
        // report the grant as fully vested.
        assert_ne!(vested, TOTAL);
    }
}
