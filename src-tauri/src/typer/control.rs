use parking_lot::{Condvar, Mutex};
use std::time::{Duration, Instant};

/// Cancellation and pause state owned by one typing run.
#[derive(Default)]
pub(crate) struct RunControl {
    state: Mutex<State>,
    changed: Condvar,
}

#[derive(Default)]
struct State {
    stopped: bool,
    paused_since: Option<Instant>,
    paused_duration: Duration,
}

impl State {
    fn total_paused(&self, now: Instant) -> Duration {
        self.paused_duration
            + self
                .paused_since
                .map(|since| now.saturating_duration_since(since))
                .unwrap_or_default()
    }

    fn resume(&mut self, now: Instant) {
        if let Some(since) = self.paused_since.take() {
            self.paused_duration += now.saturating_duration_since(since);
        }
    }
}

impl RunControl {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Stop is permanent; a later typing run must use a new control.
    pub(crate) fn stop(&self) {
        let mut state = self.state.lock();
        state.stopped = true;
        state.resume(Instant::now());
        self.changed.notify_all();
    }

    pub(crate) fn pause(&self) {
        let mut state = self.state.lock();
        if !state.stopped && state.paused_since.is_none() {
            state.paused_since = Some(Instant::now());
            self.changed.notify_all();
        }
    }

    pub(crate) fn resume(&self) {
        self.state.lock().resume(Instant::now());
        self.changed.notify_all();
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.state.lock().stopped
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.state.lock().paused_since.is_some()
    }

    /// Wait for active time, excluding pauses. Returns false when stopped.
    /// A zero duration is a checkpoint that still blocks while paused.
    pub(crate) fn wait(&self, duration: Duration) -> bool {
        let mut state = self.state.lock();
        let started = Instant::now();
        let paused_at_start = state.total_paused(started);

        loop {
            if state.stopped {
                return false;
            }
            if state.paused_since.is_some() {
                self.changed.wait(&mut state);
                continue;
            }

            let now = Instant::now();
            // Accumulating pause time also accounts for pause/resume pairs
            // that happen before this waiting thread is scheduled again.
            let paused = state.total_paused(now).saturating_sub(paused_at_start);
            let active = now
                .saturating_duration_since(started)
                .saturating_sub(paused);
            let remaining = duration.saturating_sub(active);
            if remaining.is_zero() {
                return true;
            }
            self.changed.wait_for(&mut state, remaining);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RunControl;
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn stop_interrupts_a_long_wait() {
        let control = Arc::new(RunControl::new());
        let worker_control = control.clone();
        let (result_tx, result_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            result_tx
                .send(worker_control.wait(Duration::from_secs(60)))
                .unwrap();
        });

        assert!(matches!(
            result_rx.recv_timeout(Duration::from_millis(30)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        control.stop();
        assert!(!result_rx.recv_timeout(Duration::from_secs(1)).unwrap());
        worker.join().unwrap();
    }

    #[test]
    fn stop_releases_a_paused_checkpoint() {
        let control = Arc::new(RunControl::new());
        control.pause();
        let worker_control = control.clone();
        let (result_tx, result_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            result_tx.send(worker_control.wait(Duration::ZERO)).unwrap();
        });

        assert!(control.is_paused());
        assert!(matches!(
            result_rx.recv_timeout(Duration::from_millis(30)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        control.stop();
        assert!(!result_rx.recv_timeout(Duration::from_secs(1)).unwrap());
        assert!(control.is_stopped());
        assert!(!control.is_paused());
        worker.join().unwrap();
    }

    #[test]
    fn paused_time_does_not_consume_the_wait() {
        let control = Arc::new(RunControl::new());
        control.pause();
        let worker_control = control.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let delay = Duration::from_millis(120);
        let worker = thread::spawn(move || {
            started_tx.send(()).unwrap();
            result_tx.send(worker_control.wait(delay)).unwrap();
        });

        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(matches!(
            result_rx.recv_timeout(delay * 2),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        let resumed = Instant::now();
        control.resume();
        assert!(result_rx.recv_timeout(Duration::from_secs(1)).unwrap());
        assert!(resumed.elapsed() >= delay);
        worker.join().unwrap();
    }

    #[test]
    fn stopped_controls_cannot_be_resumed_or_paused() {
        let control = RunControl::new();
        control.stop();
        control.resume();
        control.pause();

        assert!(control.is_stopped());
        assert!(!control.is_paused());
        assert!(!control.wait(Duration::ZERO));
        assert!(!control.wait(Duration::from_secs(60)));
    }
}
