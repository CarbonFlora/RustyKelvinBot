use deepseek_rs::{
    client::chat_completions::request::{Message, RequestBody},
    request::{Model, Role},
    DeepSeekClient,
};

use crate::{split_action, token::TokenType, RustyKelvinBot};

const CONTEXT_SIZE: u8 = 21;

impl RustyKelvinBot {
    pub async fn deepseek_chat(self, reasoning: bool) {
        let api_key = self.tokens.get(&TokenType::DeepSeek);
        let client = DeepSeekClient::new_with_api_key(api_key.to_string());
        let request_body = match reasoning {
            true => self.clone().reasoning_body().await,
            false => self.clone().chat_body().await,
        };
        let mut skeleton_message = self
            .clone()
            .send_message(String::from("*Thinking...*"))
            .await
            .expect("Failed to send skeleton message.");
        let cc_response = client
            .chat_completions(request_body)
            .await
            .expect("Failed to get a valid response from DeepSeek.");
        let cc_choices = cc_response
            .choices
            .first()
            .expect("DeekSeek responded with no choices.")
            .to_owned();
        let response = cc_choices
            .message
            .content
            .expect("DeekSeek responded with an empty message.");
        self.edit_message(&mut skeleton_message, &response).await;
    }

    async fn chat_body(self) -> RequestBody {
        let messages = self
            .clone()
            .read_latest_messages(self.msg.channel_id, CONTEXT_SIZE)
            .await
            .into_iter()
            .map(|v| v.to_deekseek_message(self.ctx.cache.current_user().id))
            .rev()
            .collect::<Vec<Message>>();
        println!("{:#?}", messages.clone());
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
        println!("{:#?}", messages.clone());
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
