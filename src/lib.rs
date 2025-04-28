use std::collections::VecDeque;

use serenity::all::{ChannelId, Context, EditMessage, GetMessages, Message};
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
        let (action, _content) = split_action(self.msg.content.clone());
        let rkb_binding = self.clone();
        match action.as_str() {
            "weather" | "temperature" | "temp" => tokio::spawn(rkb_binding.weather()),
            "geo" => tokio::spawn(rkb_binding.geo()),
            "chat" => tokio::spawn(rkb_binding.deepseek_chat(false)),
            "reason" => tokio::spawn(rkb_binding.deepseek_chat(true)),
            "test" => tokio::spawn(rkb_binding.test()),
            _ => tokio::spawn(rkb_binding.nonaction()),
        };
    }

    async fn nonaction(self) {
        self.send_message("non-action. 🎣".to_owned()).await;
    }

    async fn send_message(self, response: String) -> Option<Message> {
        let responses = breakdown_string(response);
        self.send_message_batch(responses).await
    }

    async fn send_message_batch(self, responses: VecDeque<String>) -> Option<Message> {
        let mut latest_message = None;
        for response in responses {
            match self.msg.channel_id.say(&self.ctx.http, response).await {
                Ok(message) => latest_message = Some(message),
                Err(e) => error!("Error sending message: {:?}", e),
            };
        }
        latest_message
    }

    async fn read_latest_messages(self, channel_id: ChannelId, count: u8) -> Vec<Message> {
        channel_id
            .messages(self.ctx.http, GetMessages::new().limit(count))
            .await
            .expect("Failed to get text channel history.")
    }

    async fn edit_message(self, message: &mut Message, response: &str) -> Option<Message> {
        let mut responses = breakdown_string(response.to_string());
        let first_response = responses.pop_front()?;
        let builder = EditMessage::new().content(first_response);
        message
            .edit(self.ctx.clone(), builder)
            .await
            .expect("Failed to edit Discord message.");
        self.send_message_batch(responses).await
    }
}

pub fn split_action(message: String) -> (String, String) {
    let stripped_msg = message.trim_start_matches(ENTRY_STRING).to_string();
    stripped_msg
        .split_once(' ')
        .map(|v| (v.0.to_string(), v.1.to_string()))
        .unwrap_or((stripped_msg, String::new()))
}

const MAX_MESSAGE_BREAKS: usize = 3;

fn breakdown_string(string: String) -> VecDeque<String> {
    let mut strings = VecDeque::new();
    let mut remaining_string = string;
    let mut available_message_breaks = MAX_MESSAGE_BREAKS;
    while available_message_breaks > 0 && !remaining_string.is_empty() {
        let (string_block, remaining_string_block) = remaining_string
            .split_at_checked(2000)
            .map(|v| (v.0.to_string(), v.1.to_string()))
            .unwrap_or((remaining_string, String::new()));
        remaining_string = remaining_string_block.to_string();
        available_message_breaks -= 1;
        strings.push_back(string_block.to_string());
    }
    strings
}
