use std::{fmt::Display, time::Duration};

use chrono::{DateTime, Local, TimeDelta, Utc};
use thiserror::Error;

use crate::{err::RKBServiceRequestErr, RKBServiceRequest};

#[derive(Debug, Error)]
pub enum Error {
    #[error("user input time is required")]
    CannotParseEmptyUserInputTime,
    #[error("user input time is out of bounds")]
    Overflow,
    #[error("user input time does not follow standard pattern")]
    DoesntFollowPattern,
}

#[derive(Debug, Clone, PartialEq)]
struct Timer {
    pub dob: DateTime<Utc>,
    pub delta: Duration,
    pub recalled_message: String,
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut remainder = self.delta.as_secs();
        let mut delta = String::new();
        let days = remainder / 86400;
        remainder %= 86400;
        let hours = remainder / 3600;
        remainder %= 3600;
        let minutes = remainder / 60;
        remainder %= 60;
        let seconds = remainder;

        if days > 0 {
            delta += &format!("{}d", days);
        };
        if hours > 0 {
            delta += &format!("{}h", hours);
        };
        if minutes > 0 {
            delta += &format!("{}m", minutes);
        };
        if seconds > 0 {
            delta += &format!("{}s", seconds);
        };
        let start = Local::now();
        let end = Local::now() + self.delta;
        writeln!(
            f,
            "[Time: {delta} | Start: {start} | End: {end}]
            \n{}
            ",
            self.recalled_message
        )
    }
}

impl TryFrom<RKBServiceRequest> for Timer {
    type Error = RKBServiceRequestErr;

    fn try_from(value: RKBServiceRequest) -> Result<Self, Self::Error> {
        let mut content = value
            .get_content()
            .ok_or(Error::CannotParseEmptyUserInputTime)?;
        let mut recalled_message = String::new();
        if content.contains(' ') {
            (content, recalled_message) = content
                .split_once(' ')
                .map(|(a, b)| (a, b.to_string()))
                .ok_or(Error::DoesntFollowPattern)?;
        }
        let timedelta = try_deltatime(content.to_string())?;
        if timedelta.is_zero() {
            Err(Error::CannotParseEmptyUserInputTime)?;
        }
        let duration = timedelta.to_std().map_err(|_| Error::Overflow)?;
        Ok(Timer {
            dob: Utc::now(),
            delta: duration,
            recalled_message,
        })
    }
}

impl RKBServiceRequest {
    pub async fn timer(&self) -> Result<(), RKBServiceRequestErr> {
        let timer = Timer::try_from(self.clone())?;
        let timer_message = self.try_send_message(timer.to_string()).await?;
        self.try_pin(timer_message.id).await?;
        tokio::time::sleep(timer.delta).await;
        self.try_delete_message(timer_message.id).await?;
        self.try_send_message(timer.recalled_message).await?;
        Ok(())
    }
}

fn try_deltatime(string: String) -> Result<TimeDelta, RKBServiceRequestErr> {
    let mut chars = string.chars();
    let mut buffer = chars
        .next()
        .map(|v| v.to_string())
        .ok_or(Error::DoesntFollowPattern)?;
    let mut delta = TimeDelta::zero();
    for char in chars {
        let o_multiplier = match char {
            'd' => Some(86400),
            'h' => Some(3600),
            'm' => Some(60),
            's' => Some(1),
            _ => None,
        };
        if o_multiplier.is_none() {
            buffer.push(char);
            continue;
        }
        let multiplier = o_multiplier.unwrap();
        let base = buffer.parse::<i64>().map_err(|_| Error::Overflow)?;
        let delta_additional = &TimeDelta::try_seconds(base * multiplier).ok_or(Error::Overflow)?;
        delta = TimeDelta::checked_add(&delta, delta_additional).ok_or(Error::Overflow)?;
        buffer.clear();
    }

    Ok(delta)
}
