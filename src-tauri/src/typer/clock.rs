use std::time::{Duration, Instant};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTiming {
    pub elapsed_ms: u64,
    pub remaining_ms: Option<u64>,
}

pub(super) fn estimate_remaining_ms(elapsed_ms: u64, current: u32, total: u32) -> Option<u64> {
    if total > 0 && current >= total {
        return Some(0);
    }
    // Wait for a useful pace sample; a single keystroke after a long initial
    // thinking pause would imply a wildly misleading estimate.
    if current < 10 || elapsed_ms < 5000 {
        return None;
    }
    Some(
        ((elapsed_ms as u128 * total.saturating_sub(current) as u128) / current as u128)
            .min(u64::MAX as u128) as u64,
    )
}

/// Active session time includes planning/revisions, but excludes countdowns and
/// manual or automatic pauses. The engine owns it so reopening the UI is safe.
#[derive(Default)]
pub(super) struct SessionClock {
    elapsed: Duration,
    running_since: Option<Instant>,
}

impl SessionClock {
    pub fn set_running(&mut self, running: bool, now: Instant) {
        if running {
            self.running_since.get_or_insert(now);
        } else if let Some(started) = self.running_since.take() {
            self.elapsed += now.saturating_duration_since(started);
        }
    }

    pub fn elapsed_ms(&self, now: Instant) -> u64 {
        (self.elapsed
            + self
                .running_since
                .map(|start| now.saturating_duration_since(start))
                .unwrap_or_default())
        .as_millis()
        .min(u64::MAX as u128) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_excludes_countdown_and_pauses_and_freezes_at_completion() {
        let mut clock = SessionClock::default();
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        assert_eq!(clock.elapsed_ms(at(3)), 0);
        clock.set_running(true, at(3));
        clock.set_running(true, at(4)); // repeated events must not reset time
        assert_eq!(clock.elapsed_ms(at(10)), 7000);
        clock.set_running(false, at(10));
        assert_eq!(clock.elapsed_ms(at(20)), 7000);
        clock.set_running(true, at(23));
        clock.set_running(false, at(28));
        assert_eq!(clock.elapsed_ms(at(100)), 12000);
        clock = SessionClock::default();
        assert_eq!(clock.elapsed_ms(at(101)), 0);
    }

    #[test]
    fn remaining_time_waits_for_a_sample_and_tracks_observed_pace() {
        assert_eq!(estimate_remaining_ms(0, 0, 100), None);
        assert_eq!(estimate_remaining_ms(9000, 1, 100), None);
        assert_eq!(estimate_remaining_ms(1000, 20, 100), None);
        assert_eq!(estimate_remaining_ms(10000, 20, 100), Some(40000));
        assert_eq!(estimate_remaining_ms(15000, 20, 100), Some(60000));
        assert_eq!(estimate_remaining_ms(15000, 100, 100), Some(0));
        assert_eq!(
            estimate_remaining_ms(u64::MAX, 10, u32::MAX),
            Some(u64::MAX)
        );
    }
}
