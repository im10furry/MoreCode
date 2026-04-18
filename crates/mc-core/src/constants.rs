pub const CONTROL_CHANNEL_CAPACITY: usize = 32;
pub const STATE_CHANNEL_CAPACITY: usize = 64;
pub const DATA_LINK_CHANNEL_CAPACITY: usize = 128;
pub const BROADCAST_CHANNEL_CAPACITY: usize = 64;
pub const APPROVAL_CHANNEL_CAPACITY: usize = 10;
pub const SEND_TIMEOUT_MS: u64 = 30_000;
pub const QUEUE_DEPTH_ALERT_PERCENT: usize = 80;
pub const MORECODE_ENV_PREFIX: &str = "MORECODE_";
pub const GLOBAL_CONFIG_SUBDIR: &str = ".config/morecode";
pub const PROJECT_CONFIG_SUBDIR: &str = ".morecode";
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const CONFIG_EVENT_CHANNEL_CAPACITY: usize = 32;
pub const HOT_RELOAD_SETTLE_MILLIS: u64 = 250;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_constants_match_contract() {
        assert_eq!(CONTROL_CHANNEL_CAPACITY, 32);
        assert_eq!(STATE_CHANNEL_CAPACITY, 64);
        assert_eq!(DATA_LINK_CHANNEL_CAPACITY, 128);
        assert_eq!(BROADCAST_CHANNEL_CAPACITY, 64);
        assert_eq!(APPROVAL_CHANNEL_CAPACITY, 10);
        assert_eq!(SEND_TIMEOUT_MS, 30_000);
        assert_eq!(QUEUE_DEPTH_ALERT_PERCENT, 80);
        assert_eq!(MORECODE_ENV_PREFIX, "MORECODE_");
        assert_eq!(GLOBAL_CONFIG_SUBDIR, ".config/morecode");
        assert_eq!(PROJECT_CONFIG_SUBDIR, ".morecode");
        assert_eq!(CONFIG_FILE_NAME, "config.toml");
        assert_eq!(CONFIG_EVENT_CHANNEL_CAPACITY, 32);
        assert_eq!(HOT_RELOAD_SETTLE_MILLIS, 250);
    }
}
