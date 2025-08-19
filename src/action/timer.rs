use std::{fmt::Display, time::Duration};

use chrono::{DateTime, TimeDelta, Utc};
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
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut remainder = self.delta.as_secs();
        let mut string = String::new();
        let days = remainder / 86400;
        remainder %= 86400;
        let hours = remainder / 3600;
        remainder %= 3600;
        let minutes = remainder / 60;
        remainder %= 60;
        let seconds = remainder;

        if days > 0 {
            string += &format!("{} Days", days);
        };
        if hours > 0 {
            string += &format!("{} Hours", hours);
        };
        if minutes > 0 {
            string += &format!("{} Minutes", minutes);
        };
        if seconds > 0 {
            string += &format!("{} Seconds", seconds);
        };

        writeln!(f, "{}", string)
    }
}

impl TryFrom<RKBServiceRequest> for Timer {
    type Error = RKBServiceRequestErr;

    fn try_from(value: RKBServiceRequest) -> Result<Self, Self::Error> {
        let content = value
            .get_content()
            .ok_or(Error::CannotParseEmptyUserInputTime)?;
        let r_delta = try_delta_time(content.to_string())?;
        let delta = r_delta.to_std().map_err(|_| Error::Overflow)?;
        let dob = Utc::now();
        Ok(Timer { dob, delta })
    }
}

impl RKBServiceRequest {
    pub async fn timer(self) -> Result<(), RKBServiceRequestErr> {
        let timer = Timer::try_from(self.clone())?;
        let rkbmessage_start = self.clone().try_send_message(timer.to_string()).await?;
        // let raw = split_action(self.msg.content.clone()).1;
        // let r_delta = try_delta_time(raw);
        // if let Err(err) = r_delta {
        //     self.send_message(err).await;
        //     return;
        // }
        // let delta = r_delta.unwrap();
        // let utc = Utc::now() + delta;
        // let delta_string = format!("{} ", utc.to_rfc3339());
        // let message = self.clone().send_message(delta_string).await.unwrap();
        self.msg
            .channel_id
            .pin(&self.ctx.http, rkbmessage_start.id)
            .await
            .expect("Bot missing Manage Messages permissions.");
        self.clone().timer_loop(timer.delta).await;
        let _ = self
            .msg
            .channel_id
            .delete_message(&self.ctx.http, rkbmessage_start.id)
            .await;
        Ok(())
    }

    pub async fn timer_loop(self, period: Duration) {
        tokio::time::sleep(period).await;
        self.clone().send_message("Timers dead.".to_string()).await;
    }

    // pub async fn minute_timer(self) {
    //     let mut interval = time::interval(Duration::from_secs(30));
    //     loop {
    //         interval.tick().await;
    //         self.clone().send_message(Utc::now().to_string()).await;
    //     }
    // }
}

fn try_delta_time(string: String) -> Result<TimeDelta, RKBServiceRequestErr> {
    let mut chars = string.chars();
    let mut buffer = chars
        .next()
        .map(|v| v.to_string())
        .ok_or(Error::DoesntFollowPattern)?;
    let mut delta = TimeDelta::zero();
    for char in chars {
        let o_multiplier = match char {
            'd' => Some(1440),
            'h' => Some(60),
            'm' => Some(1),
            _ => None,
        };
        if o_multiplier.is_none() {
            buffer.push(char);
            continue;
        }
        let multiplier = o_multiplier.unwrap();
        let base = buffer.parse::<i64>().map_err(|_| Error::Overflow)?;
        let delta_additional = &TimeDelta::try_minutes(base * multiplier).ok_or(Error::Overflow)?;
        delta = TimeDelta::checked_add(&delta, delta_additional).ok_or(Error::Overflow)?;
        buffer.clear();
    }

    Ok(delta)
}
