use thiserror::Error;

#[derive(Debug, Error)]
pub enum RKBServiceRequestErr {
    #[error("error caused by unknown means")]
    Unknown,
    #[error("token keys error")]
    Token(#[from] crate::token::Error),
    #[error("timer action error")]
    Timer(#[from] crate::action::timer::Error),
    #[error("deepseek action error")]
    Deepseek(#[from] crate::action::deepseek::Error),
    #[error("weather action error")]
    Weather(#[from] crate::action::weather::Error),
    #[error("failed to send discord message")]
    DiscordMessageSendFailure(String),
    #[error("attempted to send no messages")]
    DiscordMessageSendEmpty,
    #[error("missing permissions")]
    DiscordMissingPermissions,
}
