use chrono::Utc;
use rand::RngCore;
use snapdown_core::ports::{Clock, EntropySource};

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl SystemClock {
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    fn now_rfc3339(&self) -> String {
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }

    fn now_unix_millis(&self) -> u64 {
        Utc::now().timestamp_millis().max(0) as u64
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemEntropySource;

impl SystemEntropySource {
    pub fn new() -> Self {
        Self
    }
}

impl EntropySource for SystemEntropySource {
    fn random_bytes_10(&self) -> [u8; 10] {
        let mut bytes = [0u8; 10];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_produces_rfc3339_utc_with_z() {
        let clock = SystemClock::new();
        let ts = clock.now_rfc3339();
        assert!(ts.ends_with('Z'), "Timestamp must end with Z: {ts}");
        let millis = clock.now_unix_millis();
        assert!(millis > 1_700_000_000_000);
    }

    #[test]
    fn system_entropy_source_produces_non_zero_bytes() {
        let entropy = SystemEntropySource::new();
        let b1 = entropy.random_bytes_10();
        let b2 = entropy.random_bytes_10();
        assert_eq!(b1.len(), 10);
        assert_eq!(b2.len(), 10);
        assert_ne!(b1, b2);
    }
}
