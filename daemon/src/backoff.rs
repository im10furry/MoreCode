use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    base_delay: Duration,
    max_delay: Duration,
    current_delay: Duration,
}

impl ExponentialBackoff {
    pub fn new(base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            base_delay,
            max_delay,
            current_delay: base_delay,
        }
    }

    pub fn current_delay(&self) -> Duration {
        self.current_delay
    }

    pub fn next_delay(&mut self) -> Duration {
        let delay = self.current_delay;
        self.current_delay = (self.current_delay * 2).min(self.max_delay);
        delay
    }

    pub fn reset(&mut self) {
        self.current_delay = self.base_delay;
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(Duration::from_secs(1), Duration::from_secs(300))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::ExponentialBackoff;

    #[test]
    fn backoff_grows_and_resets() {
        let mut backoff = ExponentialBackoff::new(Duration::from_secs(1), Duration::from_secs(8));
        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
        assert_eq!(backoff.next_delay(), Duration::from_secs(4));
        assert_eq!(backoff.next_delay(), Duration::from_secs(8));
        assert_eq!(backoff.next_delay(), Duration::from_secs(8));
        backoff.reset();
        assert_eq!(backoff.current_delay(), Duration::from_secs(1));
    }
}
