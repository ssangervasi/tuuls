use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Ringer {
    frequency: Duration,
    last_time: Option<Instant>,
    queued: bool,
}

impl Default for Ringer {
    fn default() -> Self {
        Self::new()
    }
}

impl Ringer {
    pub fn new() -> Self {
        Self {
            frequency: Duration::from_millis(500),
            last_time: None,
            queued: false,
        }
    }

    pub fn ring(&mut self) {
        let time_now = Instant::now();
        let time_elapsed = match self.last_time {
            Some(time_start) => time_now - time_start,
            None => self.frequency,
        };
        if time_elapsed < self.frequency {
            return;
        }
        self.last_time = Some(time_now);
        self.queued = true
    }

    pub fn flush(&mut self) -> bool {
        let res = self.queued;
        self.queued = false;
        res
    }
}
