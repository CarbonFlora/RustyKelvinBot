use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use serenity::all::Message;
use tokio::time::interval;

use crate::{split_action, RustyKelvinBot};

impl RustyKelvinBot {
    pub async fn timer(mut self) {
        let raw = split_action(self.msg.content.clone()).1;
        let r_delta = try_delta_time(raw);
        if let Err(err) = r_delta {
            self.send_message(err).await;
            return;
        }
        let delta = r_delta.unwrap();
        let utc = Utc::now() + delta;
        let delta_string = format!("{} ", utc.to_rfc3339());
        self.rsc.active_timers.push(utc);
        let message = self.clone().send_message(delta_string).await.unwrap();
        self.msg
            .channel_id
            .pin(&self.ctx.http, message.id)
            .await
            .expect("Bot missing Manage Messages permissions.");
        self.refresh_timers().await;
    }

    pub async fn startup_refresh_timers(self) {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            self.clone().refresh_timers().await;
        }
    }

    // this is ass, redesign so it saves a file locally instead since it's just for me.
    pub async fn refresh_timers(self) {
        todo!();
        // this is shit performance
        self.clone().refresh_timers_get_messages().await;
        self.refresh_timers_reminders().await;
    }

    pub async fn refresh_timers_reminders(self) {
        for timer in &self.rsc.active_timers {
            println!("a: {}", timer);
            if &Utc::now() >= timer {
                println!("b: {}", timer);
                self.clone()
                    .send_message(String::from("Timer is up."))
                    .await;
            }
        }
    }

    async fn refresh_timers_get_messages(mut self) {
        for gid in self.ctx.cache.guilds() {
            let Ok(hm_cid) = gid.channels(&self.ctx.http).await else {
                continue;
            };
            for (cid, _gc) in hm_cid {
                let Ok(messages) = cid.pins(&self.ctx.http).await else {
                    continue;
                };
                self.refresh_timers_update_resource(messages).await;
            }
        }
    }

    async fn refresh_timers_update_resource(&mut self, messages: Vec<Message>) {
        let mut active_timers = Vec::new();
        for message in messages {
            let Ok(dt) = DateTime::parse_from_rfc3339(&message.content) else {
                continue;
            };
            let dt = dt.to_utc();
            if Utc::now() >= dt {
                message
                    .unpin(&self.ctx.http)
                    .await
                    .expect("Unable to unpin message.");
            }
            active_timers.push(dt);
        }
        println!("a: {:?}", active_timers);
        self.rsc.active_timers = active_timers;
    }
}

fn try_delta_time(string: String) -> Result<TimeDelta, String> {
    let mut chars = string.chars();
    let mut buffer = chars
        .next()
        .map(|v| v.to_string())
        .ok_or_else(|| String::from("No time given."))?;
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
        let base = buffer
            .parse::<i64>()
            .map_err(|_| String::from("Time metric is invalid."))?;
        let delta_additional = &TimeDelta::try_minutes(base * multiplier)
            .ok_or_else(|| String::from("Time metric is invalid."))?;
        delta = TimeDelta::checked_add(&delta, delta_additional)
            .ok_or_else(|| String::from("Time metric is too large."))?;
        buffer.clear();
    }

    Ok(delta)
}
