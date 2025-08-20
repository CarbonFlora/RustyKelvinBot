use std::collections::VecDeque;

use err::RKBServiceRequestErr;
use resource::Resources;
use serenity::all::{ChannelId, Context, EditMessage, GetMessages, Message, MessageId};
use token::Tokens;
use tracing::error;

pub mod action;
pub mod err;
pub mod resource;
pub mod text;
mod token;

#[derive(Debug, Clone)]
pub struct RKBServiceRequest {
    pub ctx: Context,
    pub msg: Message,
    pub tkn: Tokens,
    pub rsc: Resources,
}

const ENTRY_STRING: &str = "?";

impl RKBServiceRequest {
    pub fn new(ctx: Context, msg: Message) -> Self {
        RKBServiceRequest {
            ctx,
            msg,
            tkn: Tokens::default(),
            rsc: Resources::default(),
        }
    }

    pub fn get_content(&self) -> Option<&str> {
        self.msg
            .content
            .trim_start_matches(ENTRY_STRING)
            .split_once(' ')
            .map(|v| v.1.trim())
    }

    pub async fn is_user_message(&self) -> bool {
        !self.msg.author.bot && !self.msg.author.system
    }

    pub async fn handle_message(self) -> Result<(), RKBServiceRequestErr> {
        if !self.msg.content.starts_with(ENTRY_STRING) {
            return Ok(());
        }
        let (action, _content) = split_action(self.msg.content.clone());
        let rkb_binding = self.clone();

        match action.as_str() {
            "help" | "" => rkb_binding.help().await?,
            "weather" | "temperature" | "temp" => rkb_binding.weather().await?,
            "geo" => rkb_binding.geo().await?,
            "chat" => rkb_binding.deepseek_chat(false, None).await?,
            "reason" => rkb_binding.deepseek_chat(true, None).await?,
            // "test" => tokio::spawn(rkb_binding.test()),
            "timer" => rkb_binding.timer().await?,
            _ => rkb_binding.nonaction().await?,
        };
        Ok(())
    }

    pub async fn pinned_handle_message(self) -> bool {
        let pinned_messages = self
            .msg
            .channel_id
            .pins(&self.ctx.http)
            .await
            .expect("Bot missing read history permissions.");
        let Some(pinned_message) = pinned_messages
            .into_iter()
            .find(|msg| msg.content.starts_with(ENTRY_STRING))
        else {
            return false;
        };
        let (pinned_action, pinned_content) = split_action(pinned_message.content);
        let (_action, _content) = split_action(self.msg.content.clone());
        let rkb_binding = self.clone();
        match pinned_action.as_str() {
            "chat" => tokio::spawn(rkb_binding.deepseek_chat(false, Some(pinned_content))),
            _ => tokio::spawn(rkb_binding.nonaction_pinned()),
        };
        true
    }

    async fn nonaction(self) -> Result<(), RKBServiceRequestErr> {
        self.try_send_message("non-action. 🎣".to_owned()).await?;
        Ok(())
    }

    async fn nonaction_pinned(self) -> Result<(), RKBServiceRequestErr> {
        self.try_send_message("Pinned message is a non-action. 🎣".to_owned())
            .await?;
        Ok(())
    }

    async fn send_message(self, response: String) -> Option<Message> {
        let responses = breakdown_string(response);
        self.send_message_batch(responses).await
    }

    async fn try_send_message(&self, response: String) -> Result<Message, RKBServiceRequestErr> {
        let responses = breakdown_string(response);
        self.try_send_message_batch(responses).await
    }

    async fn try_send_message_batch(
        &self,
        responses: VecDeque<String>,
    ) -> Result<Message, RKBServiceRequestErr> {
        let mut latest_message = None;
        for response in &responses {
            latest_message = self
                .msg
                .channel_id
                .say(&self.ctx.http, response)
                .await
                .map(Some)
                .map_err(|_| RKBServiceRequestErr::DiscordMessageSendFailure(response.clone()))?;
        }
        let Some(last_message) = latest_message else {
            Err(RKBServiceRequestErr::DiscordMessageSendEmpty)?
        };
        Ok(last_message)
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

    async fn try_edit_message(
        self,
        message: &mut Message,
        response: &str,
    ) -> Result<Message, RKBServiceRequestErr> {
        let mut responses = breakdown_string(response.to_string());
        let first_response = responses
            .pop_front()
            .ok_or(RKBServiceRequestErr::DiscordMessageSendEmpty)?;
        let builder = EditMessage::new().content(first_response);
        message
            .edit(self.ctx.clone(), builder)
            .await
            .expect("Failed to edit Discord message.");
        self.try_send_message_batch(responses).await
    }

    pub async fn try_pin(
        &self,
        message_id: impl Into<MessageId>,
    ) -> Result<(), RKBServiceRequestErr> {
        self.msg
            .channel_id
            .pin(&self.ctx.http, message_id)
            .await
            .map_err(|_| RKBServiceRequestErr::DiscordMissingPermissions)
    }

    pub async fn try_delete_message(
        &self,
        message_id: impl Into<MessageId>,
    ) -> Result<(), RKBServiceRequestErr> {
        self.msg
            .channel_id
            .delete_message(&self.ctx.http, message_id)
            .await
            .map_err(|_| RKBServiceRequestErr::DiscordMissingPermissions)
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
