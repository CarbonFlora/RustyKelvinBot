use serenity::all::{Context, EditMessage, Message};
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
        let stripped_msg = self
            .msg
            .content
            .trim_start_matches(ENTRY_STRING)
            .to_string();
        let (action, content) = stripped_msg
            .split_once(' ')
            .map(|v| (v.0.to_string(), v.1.to_string()))
            .unwrap_or((stripped_msg, String::new()));

        // .unwrap_or((&stripped_msg, ""));
        let rkb_binding = self.clone();
        match action.as_str() {
            "weather" | "temperature" | "temp" => tokio::spawn(rkb_binding.weather()),
            "geo" => tokio::spawn(rkb_binding.geo()),
            "chat" => tokio::spawn(rkb_binding.deepseek_chat(content.clone())),
            "test" => tokio::spawn(rkb_binding.test()),
            _ => tokio::spawn(rkb_binding.nonaction()),
        };
    }

    async fn nonaction(self) {
        self.send_message("non-action. 🎣".to_owned()).await;
    }

    async fn send_message(self, mut response: String) -> Option<Message> {
        response.truncate(2000);
        match self.msg.channel_id.say(&self.ctx.http, response).await {
            Ok(message) => return Some(message),
            Err(e) => error!("Error sending message: {:?}", e),
        };
        None
    }

    async fn edit_message(self, message: &mut Message, response: &str) {
        let builder = EditMessage::new().content(response);
        message
            .edit(self.ctx, builder)
            .await
            .expect("Failed to edit Discord message.");
    }
}
