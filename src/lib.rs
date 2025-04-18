use serenity::all::{Context, Message};
use token::RKBTokens;
use tracing::error;

pub mod action;
mod token;

#[derive(Debug, Clone)]
pub struct RustyKelvinBot {
    pub ctx: Context,
    pub msg: Message,
    pub tokens: RKBTokens,
}

const ENTRY_STRING: &str = "?";

impl RustyKelvinBot {
    pub fn new(ctx: Context, msg: Message) -> Self {
        RustyKelvinBot {
            ctx,
            msg,
            tokens: RKBTokens::default(),
        }
    }

    pub async fn handle_message(self) {
        if !self.msg.content.starts_with(ENTRY_STRING) {
            return;
        }
        let mut content = self
            .msg
            .content
            .trim_start_matches(ENTRY_STRING)
            .split_whitespace()
            .collect::<Vec<&str>>();
        let Some(action) = content.first().copied() else {
            return;
        };
        content.remove(0);
        let rkb_binding = self.clone();
        match action {
            "weather" | "temperature" | "temp" => tokio::spawn(rkb_binding.weather()),
            "geo" => tokio::spawn(rkb_binding.geo()),
            "test" => tokio::spawn(rkb_binding.test()),
            _ => tokio::spawn(rkb_binding.nonaction()),
        };
    }

    async fn nonaction(self) {
        self.send_message("non-action. 🎣".to_owned()).await;
    }

    async fn send_message(self, mut response: String) {
        response.truncate(2000);
        if let Err(e) = self.msg.channel_id.say(&self.ctx.http, response).await {
            error!("Error sending message: {:?}", e);
        }
    }
}
