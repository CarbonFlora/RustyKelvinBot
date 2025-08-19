use deepseek_rs::{
    client::chat_completions::request::{Message, RequestBody},
    request::{Model, Role},
    DeepSeekClient,
};
use thiserror::Error;

use crate::{err::RKBServiceRequestErr, split_action, token::TokenType, RKBServiceRequest};

const CONTEXT_SIZE: u8 = 21;
const SYSTEM_PROMPT: &str = "Be short and concise. Cite your sources.";

#[derive(Debug, Error)]
pub enum Error {
    #[error("placeholder")]
    Placeholder,
    #[error("invalid deepseek response")]
    DeepseekError,
}

impl RKBServiceRequest {
    pub async fn deepseek_chat(
        self,
        reasoning: bool,
        preprompt: Option<String>,
    ) -> Result<(), RKBServiceRequestErr> {
        let api_key = self.tkn.get(&TokenType::DeepSeek)?;
        let client = DeepSeekClient::new_with_api_key(api_key.to_string());
        let request_body = match reasoning {
            true => self.clone().reasoning_body().await,
            false => self.clone().chat_body(preprompt).await,
        };
        let mut skeleton_message = self.clone().try_send_message(String::from("*..*")).await?;
        let cc_response = client
            .chat_completions(request_body)
            .await
            .map_err(|_| Error::DeepseekError)?;
        let cc_choices = cc_response.choices.first().ok_or(Error::DeepseekError)?;
        let response = cc_choices
            .message
            .content
            .clone()
            .ok_or(Error::DeepseekError)?;
        // let response_md = RKBMarkdown::from(response).to_string();
        self.try_edit_message(&mut skeleton_message, &response)
            .await?;
        Ok(())
    }

    async fn chat_body(self, preprompt: Option<String>) -> RequestBody {
        let mut messages = self
            .clone()
            .read_latest_messages(self.msg.channel_id, CONTEXT_SIZE)
            .await
            .into_iter()
            .map(|v| v.to_deekseek_message(self.ctx.cache.current_user().id))
            .rev()
            .collect::<Vec<Message>>();
        messages.insert(0, Message::new_system_message(SYSTEM_PROMPT.to_string()));
        if let Some(preprompt) = preprompt {
            messages.insert(0, Message::new_system_message(preprompt));
        }
        // println!("{:#?}", messages.clone());
        RequestBody::new_messages(messages).with_model(Model::DeepseekChat)
    }

    async fn reasoning_body(self) -> RequestBody {
        let messages = self
            .clone()
            .read_latest_messages(self.msg.channel_id, 1)
            .await
            .into_iter()
            .map(|v| v.to_deekseek_message(self.ctx.cache.current_user().id))
            .collect::<Vec<Message>>();
        // println!("{:#?}", messages.clone());
        RequestBody::new_messages(messages).with_model(Model::DeepSeekReasoner)
    }
}

trait ToDeepseekMessage {
    fn to_deekseek_message(self, bot_userid: serenity::all::UserId) -> Message;
}

impl ToDeepseekMessage for serenity::all::Message {
    fn to_deekseek_message(self, bot_userid: serenity::all::UserId) -> Message {
        let (role, (_, content)) = match bot_userid == self.author.id {
            true => (Role::Assistant, (String::new(), self.content)),
            false => (Role::User, split_action(self.content)),
        };
        Message::new(role, content, None)
    }
}
