pub(super) const CAPACITY: i32 = i32::MAX;
const LEVEL_LIMIT: i32 = 32_768;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Action {
    WithdrawLevel,
    WithdrawAll,
    DepositLevel,
    DepositAll,
}

impl TryFrom<i32> for Action {
    type Error = ();

    fn try_from(id: i32) -> Result<Self, Self::Error> {
        match id {
            10 => Ok(Self::WithdrawLevel),
            11 => Ok(Self::WithdrawAll),
            20 => Ok(Self::DepositLevel),
            21 => Ok(Self::DepositAll),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Transfer {
    pub(super) balance: i32,
    pub(super) level: i32,
    pub(super) progress: f32,
    pub(super) player_delta: i32,
}

fn level_start(level: i32) -> i64 {
    let n = i64::from(level);
    match level {
        0..15 => n * n + 6 * n,
        15..30 => (5 * n * n - 81 * n + 720) / 2,
        _ => (9 * n * n - 325 * n + 4_440) / 2,
    }
}

fn level_of(points: i64) -> i32 {
    let mut lower = 0;
    let mut upper = LEVEL_LIMIT;
    while lower + 1 < upper {
        let middle = (lower + upper) / 2;
        if level_start(middle) <= points {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    lower
}

pub(super) fn transfer(
    balance: i32,
    level: i32,
    progress: f32,
    action: Action,
) -> Option<Transfer> {
    if balance < 0 || !(0..LEVEL_LIMIT).contains(&level) || !(0.0..1.0).contains(&progress) {
        return None;
    }
    let start = level_start(level);
    let next = level_start(level + 1);
    let needed = next - start;
    let current = start + ((f64::from(progress) * needed as f64).round() as i64).min(needed - 1);
    let capacity = i64::from(CAPACITY);
    if current > capacity {
        return None;
    }
    let balance = i64::from(balance);
    let withdraw_limit = balance.min(capacity - current);
    let deposit_limit = current.min(capacity - balance);
    let previous = level_start(level.saturating_sub(1).max(0));
    let delta = match action {
        Action::WithdrawLevel => withdraw_limit.min(next - current),
        Action::WithdrawAll => withdraw_limit,
        Action::DepositLevel => {
            let level_cost = if current == start {
                start - previous
            } else {
                current - start
            };
            -deposit_limit.min(level_cost)
        }
        Action::DepositAll => -deposit_limit,
    };
    if delta == 0 {
        return None;
    }
    let updated = current + delta;
    let level = level_of(updated);
    let start = level_start(level);
    let needed = level_start(level + 1) - start;
    Some(Transfer {
        balance: (balance - delta) as i32,
        level,
        progress: (updated - start) as f32 / needed as f32,
        player_delta: delta as i32,
    })
}

pub(super) fn absorb(balance: i32, value: u32, count: u32) -> (i32, u32) {
    if balance < 0 || value == 0 {
        return (balance, 0);
    }
    let room = (CAPACITY - balance) as u32;
    let taken = count.min(room / value);
    (balance + (taken * value) as i32, taken)
}

pub(super) fn encode_balance(balance: i32) -> [i32; 2] {
    [i32::from(balance as i16), balance >> 16]
}

pub(super) fn decode_balance(data: &[i32]) -> i32 {
    match data {
        [lower, upper, ..] => ((upper & 0x7fff) << 16) | (lower & 0xffff),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_vanilla_level_boundaries() {
        for (level, points) in [
            (0, 0),
            (1, 7),
            (15, 315),
            (16, 352),
            (30, 1_395),
            (31, 1_507),
        ] {
            assert_eq!(level_start(level), points);
            assert_eq!(level_of(points), level);
        }
    }

    #[test]
    fn deposits_points_held_and_round_trips() -> Result<(), &'static str> {
        let deposit =
            transfer(20, 16, 4.0 / 42.0, Action::DepositAll).ok_or("expected an XP transfer")?;
        assert_eq!(
            (deposit.level, deposit.progress, deposit.balance),
            (0, 0.0, 376)
        );
        assert_eq!(deposit.player_delta, -356);
        let withdraw =
            transfer(356, 0, 0.0, Action::WithdrawAll).ok_or("expected an XP transfer")?;
        assert_eq!((withdraw.level, withdraw.balance), (16, 0));
        assert_eq!(withdraw.progress, 4.0 / 42.0);
        assert_eq!(withdraw.player_delta, 356);
        Ok(())
    }

    #[test]
    fn single_level_uses_progress_or_the_previous_bar() -> Result<(), &'static str> {
        let partial =
            transfer(0, 2, 2.0 / 11.0, Action::DepositLevel).ok_or("expected an XP transfer")?;
        assert_eq!(
            (partial.level, partial.progress, partial.balance),
            (2, 0.0, 2)
        );
        let full = transfer(0, 2, 0.0, Action::DepositLevel).ok_or("expected an XP transfer")?;
        assert_eq!((full.level, full.progress, full.balance), (1, 0.0, 9));
        let enough =
            transfer(15, 2, 2.0 / 11.0, Action::WithdrawLevel).ok_or("expected an XP transfer")?;
        assert_eq!((enough.level, enough.progress, enough.balance), (3, 0.0, 6));
        let insufficient =
            transfer(2, 2, 2.0 / 11.0, Action::WithdrawLevel).ok_or("expected an XP transfer")?;
        assert_eq!(
            (
                insufficient.level,
                insufficient.progress,
                insufficient.balance
            ),
            (2, 4.0 / 11.0, 0)
        );
        Ok(())
    }

    #[test]
    fn bounds_each_transfer_and_rejects_invalid_input() -> Result<(), &'static str> {
        let deposit =
            transfer(CAPACITY - 1, 1, 0.0, Action::DepositAll).ok_or("expected an XP transfer")?;
        assert_eq!((deposit.balance, deposit.player_delta), (CAPACITY, -1));
        assert!(transfer(CAPACITY, 1, 0.0, Action::DepositAll).is_none());
        assert!(transfer(0, 0, 0.0, Action::DepositLevel).is_none());
        assert!(transfer(0, 1, f32::NAN, Action::DepositAll).is_none());
        assert!(transfer(2, i32::MAX, 0.0, Action::WithdrawAll).is_none());
        assert!(Action::try_from(-1).is_err());
        let large =
            transfer(CAPACITY, 0, 0.0, Action::WithdrawAll).ok_or("expected an XP transfer")?;
        assert_eq!(large.player_delta, CAPACITY);
        assert!(transfer(1, large.level, large.progress, Action::WithdrawAll).is_none());
        Ok(())
    }

    #[test]
    fn merged_orbs_only_consume_counts_that_fit() {
        assert_eq!(absorb(21, 7, 5), (56, 5));
        assert_eq!(absorb(CAPACITY - 10, 3, 5), (CAPACITY - 1, 3));
        assert_eq!(absorb(CAPACITY - 1, 3, 5), (CAPACITY - 1, 0));
        assert_eq!(absorb(10, 0, 2), (10, 0));
        assert_eq!(absorb(10, u32::MAX, u32::MAX), (10, 0));
    }

    #[test]
    fn menu_balance_survives_signed_16_bit_transport() {
        for n in [0, 1, 32_768, 65_535, 65_536, 123_456_789, CAPACITY] {
            let data = encode_balance(n);
            assert!(data.into_iter().all(|v| i16::try_from(v).is_ok()));
            assert_eq!(decode_balance(&data), n);
        }
        assert_eq!(decode_balance(&[12]), 0);
    }
}
