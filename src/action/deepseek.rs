use deepseek_rs::{
    client::chat_completions::request::{Message, RequestBody},
    DeepSeekClient,
};

use crate::{token::TokenType, RustyKelvinBot};

impl RustyKelvinBot {
    pub async fn deepseek_chat(self, content: String) {
        let api_key = self.tokens.get(&TokenType::DeepSeek);
        let client = DeepSeekClient::new_with_api_key(api_key.to_string());
        let request = RequestBody::new_messages(vec![Message::new_user_message(content)]);
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
