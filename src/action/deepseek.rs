use deepseek_rs::{
    client::chat_completions::request::{Message, RequestBody},
    request::Role,
    DeepSeekClient,
};

use crate::{token::TokenType, RustyKelvinBot};

const CONTEXT_SIZE: u8 = 21;

impl RustyKelvinBot {
    pub async fn deepseek_chat(self) {
        let api_key = self.tokens.get(&TokenType::DeepSeek);
        let client = DeepSeekClient::new_with_api_key(api_key.to_string());
        let messages = self
            .clone()
            .read_latest_messages(self.msg.channel_id, CONTEXT_SIZE)
            .await
            .into_iter()
            .map(|v| v.to_deekseek_message(self.ctx.cache.current_user().id))
            .rev()
            .collect::<Vec<Message>>();
        let request = RequestBody::new_messages(messages);
        let mut skeleton_message = self
            .clone()
            .send_message(String::from("*Thinking...*"))
            .await
            .expect("Failed to send skeleton message.");
        let cc_response = client
            .chat_completions(request)
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
}

trait ToDeepseekMessage {
    fn to_deekseek_message(self, bot_userid: serenity::all::UserId) -> Message;
}

impl ToDeepseekMessage for serenity::all::Message {
    fn to_deekseek_message(self, bot_userid: serenity::all::UserId) -> Message {
        let role = match bot_userid == self.author.id {
            true => Role::Assistant,
            false => Role::User,
        };
        Message::new(role, self.content, None)
    }
}
