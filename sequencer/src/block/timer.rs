use tokio::sync::mpsc;
use tokio::time::{self, Duration, Instant, MissedTickBehavior};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, warn};

/// Represents the current state of the block timer
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerState {
    /// Timer is running and collecting messages
    Running,
    /// Timer has been paused (e.g., during maintenance)
    Paused,
    /// Timer has been stopped and should not restart
    Stopped,
}

/// Controls the timing of block creation
pub struct BlockTimer {
    /// Current state of the timer
    state: Arc<Mutex<TimerState>>,
    /// When the current block window started
    window_start: Arc<Mutex<Instant>>,
    /// Duration of each block window
    window_duration: Duration,
    /// Channel to signal when a block should be created
    timer_tx: mpsc::Sender<Instant>,
    /// How many times we've hit our window exactly
    precise_windows: Arc<Mutex<u64>>,
    /// How many times we've exceeded our target window
    exceeded_windows: Arc<Mutex<u64>>,
}

impl BlockTimer {
    /// Create a new block timer with the specified window duration
    pub fn new(timer_tx: mpsc::Sender<Instant>, window_duration: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(TimerState::Running)),
            window_start: Arc::new(Mutex::new(Instant::now())),
            window_duration,
            timer_tx,
            precise_windows: Arc::new(Mutex::new(0)),
            exceeded_windows: Arc::new(Mutex::new(0)),
        }
    }

    /// Start the timer process
    pub async fn run(&self) {
        // Create an interval that ticks slightly more frequently than our window
        // This ensures we don't miss our window due to scheduling delays
        let mut interval = time::interval(self.window_duration - Duration::from_micros(100));
        
        // Configure how to handle missed ticks
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            // Check if we should continue
            if *self.state.lock() == TimerState::Stopped {
                break;
            }

            // Skip if paused
            if *self.state.lock() == TimerState::Paused {
                continue;
            }

            self.check_window().await;
        }
    }

    /// Check if the current window has expired and signal if necessary
    async fn check_window(&self) {
        let now = Instant::now();
        let window_start = *self.window_start.lock();
        let elapsed = now - window_start;

        if elapsed >= self.window_duration {
            // Calculate how close we were to our target window
            let overage = elapsed - self.window_duration;
            
            if overage < Duration::from_micros(100) {
                // We hit our window very precisely
                *self.precise_windows.lock() += 1;
            } else {
                // We exceeded our target window
                *self.exceeded_windows.lock() += 1;
                warn!(
                    "Block window exceeded target by {:?}",
                    overage
                );
            }

            // Signal that it's time to create a block
            if let Err(e) = self.timer_tx.send(window_start).await {
                warn!(
                    "Failed to send timer signal: {}. Continuing...",
                    e
                );
            }

            // Start a new window
            *self.window_start.lock() = now;
        }
    }

    /// Pause the timer
    pub fn pause(&self) {
        let mut state = self.state.lock();
        if *state == TimerState::Running {
            *state = TimerState::Paused;
            info!("Block timer paused");
        }
    }

    /// Resume the timer
    pub fn resume(&self) {
        let mut state = self.state.lock();
        if *state == TimerState::Paused {
            *state = TimerState::Running;
            // Reset the window start when resuming
            *self.window_start.lock() = Instant::now();
            info!("Block timer resumed");
        }
    }

    /// Stop the timer completely
    pub fn stop(&self) {
        let mut state = self.state.lock();
        *state = TimerState::Stopped;
        info!("Block timer stopped");
    }

    /// Get timing statistics
    pub fn get_stats(&self) -> TimerStats {
        TimerStats {
            precise_windows: *self.precise_windows.lock(),
            exceeded_windows: *self.exceeded_windows.lock(),
            current_state: *self.state.lock(),
        }
    }
}

/// Statistics about timer performance
#[derive(Debug, Clone, Copy)]
pub struct TimerStats {
    /// Number of precisely hit windows
    pub precise_windows: u64,
    /// Number of exceeded windows
    pub exceeded_windows: u64,
    /// Current timer state
    pub current_state: TimerState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_timer_basic_operation() {
        let (tx, mut rx) = mpsc::channel(100);
        let timer = BlockTimer::new(tx, Duration::from_millis(100));
        
        // Start the timer
        let timer_clone = timer.clone();
        tokio::spawn(async move {
            timer_clone.run().await;
        });

        // Wait for a tick
        let tick = rx.recv().await;
        assert!(tick.is_some());

        // Stop the timer
        timer.stop();
    }

    #[tokio::test]
    async fn test_timer_pause_resume() {
        let (tx, mut rx) = mpsc::channel(100);
        let timer = BlockTimer::new(tx, Duration::from_millis(100));
        
        // Start the timer
        let timer_clone = timer.clone();
        tokio::spawn(async move {
            timer_clone.run().await;
        });

        // Let it run for a bit
        sleep(Duration::from_millis(50)).await;
        
        // Pause the timer
        timer.pause();
        let stats_paused = timer.get_stats();
        assert_eq!(stats_paused.current_state, TimerState::Paused);

        // Wait while paused
        sleep(Duration::from_millis(200)).await;
        
        // Resume the timer
        timer.resume();
        let stats_resumed = timer.get_stats();
        assert_eq!(stats_resumed.current_state, TimerState::Running);

        // Stop the timer
        timer.stop();
    }
}