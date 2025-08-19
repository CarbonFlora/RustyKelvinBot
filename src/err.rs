use thiserror::Error;

#[derive(Debug, Error)]
pub enum RKBServiceRequestErr {
    #[error("error caused by unknown means")]
    Unknown,
    #[error("timer action error")]
    Timer(#[from] crate::action::timer::Error),
    #[error("failed to send discord message")]
    DiscordMessageSendFailure(String),
    #[error("attempted to send no messages")]
    DiscordMessageSendEmpty,
}
