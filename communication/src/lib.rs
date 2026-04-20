pub mod approval;
pub mod broadcast;
mod channel_group;
mod channels;
pub mod constants;
pub mod control;
pub mod data_plane;
mod error;
pub mod state;
pub mod throttle;

pub use approval::{ApprovalRequest, ApprovalResponse};
pub use broadcast::{BroadcastEvent, BroadcastSubscriber};
pub use channel_group::ChannelGroup;
pub use channels::CommunicationChannels;
pub use constants::{
    capacity_for, ChannelKind, APPROVAL_CHANNEL_CAPACITY, BROADCAST_CHANNEL_CAPACITY,
    CONTROL_CHANNEL_CAPACITY, DATA_LINK_CHANNEL_CAPACITY, DEFAULT_PROGRESS_FLUSH_LIMIT,
    DEFAULT_PROGRESS_THROTTLE_MS, QUEUE_DEPTH_ALERT_PERCENT, SEND_TIMEOUT_MS,
    STATE_CHANNEL_CAPACITY,
};
pub use control::ControlMessage;
pub use data_plane::DataPlaneMessage;
pub use error::CommunicationError;
pub use state::StateMessage;
pub use throttle::{StateThrottler, ThrottleOutcome};

#[cfg(test)]
mod test_support;
