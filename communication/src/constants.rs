pub use mc_core::{
    APPROVAL_CHANNEL_CAPACITY, BROADCAST_CHANNEL_CAPACITY, CONTROL_CHANNEL_CAPACITY,
    DATA_LINK_CHANNEL_CAPACITY, QUEUE_DEPTH_ALERT_PERCENT, SEND_TIMEOUT_MS, STATE_CHANNEL_CAPACITY,
};

pub const DEFAULT_PROGRESS_THROTTLE_MS: u64 = 200;
pub const DEFAULT_PROGRESS_FLUSH_LIMIT: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelKind {
    Control,
    State,
    DataLink,
    Broadcast,
    Approval,
}

pub fn capacity_for(kind: ChannelKind) -> usize {
    match kind {
        ChannelKind::Control => CONTROL_CHANNEL_CAPACITY,
        ChannelKind::State => STATE_CHANNEL_CAPACITY,
        ChannelKind::DataLink => DATA_LINK_CHANNEL_CAPACITY,
        ChannelKind::Broadcast => BROADCAST_CHANNEL_CAPACITY,
        ChannelKind::Approval => APPROVAL_CHANNEL_CAPACITY,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        capacity_for, ChannelKind, APPROVAL_CHANNEL_CAPACITY, BROADCAST_CHANNEL_CAPACITY,
        CONTROL_CHANNEL_CAPACITY, DATA_LINK_CHANNEL_CAPACITY, DEFAULT_PROGRESS_FLUSH_LIMIT,
        DEFAULT_PROGRESS_THROTTLE_MS, QUEUE_DEPTH_ALERT_PERCENT, SEND_TIMEOUT_MS,
        STATE_CHANNEL_CAPACITY,
    };

    #[test]
    fn communication_constants_match_contract() {
        assert_eq!(CONTROL_CHANNEL_CAPACITY, 32);
        assert_eq!(STATE_CHANNEL_CAPACITY, 64);
        assert_eq!(DATA_LINK_CHANNEL_CAPACITY, 128);
        assert_eq!(BROADCAST_CHANNEL_CAPACITY, 64);
        assert_eq!(APPROVAL_CHANNEL_CAPACITY, 10);
        assert_eq!(SEND_TIMEOUT_MS, 30_000);
        assert_eq!(QUEUE_DEPTH_ALERT_PERCENT, 80);
        assert_eq!(DEFAULT_PROGRESS_THROTTLE_MS, 200);
        assert_eq!(DEFAULT_PROGRESS_FLUSH_LIMIT, 32);
    }

    #[test]
    fn channel_kind_maps_to_expected_capacities() {
        assert_eq!(capacity_for(ChannelKind::Control), CONTROL_CHANNEL_CAPACITY);
        assert_eq!(capacity_for(ChannelKind::State), STATE_CHANNEL_CAPACITY);
        assert_eq!(
            capacity_for(ChannelKind::DataLink),
            DATA_LINK_CHANNEL_CAPACITY
        );
        assert_eq!(
            capacity_for(ChannelKind::Broadcast),
            BROADCAST_CHANNEL_CAPACITY
        );
        assert_eq!(
            capacity_for(ChannelKind::Approval),
            APPROVAL_CHANNEL_CAPACITY
        );
    }
}
