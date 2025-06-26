use deepseek_rs::{
    client::chat_completions::request::{Message, RequestBody},
    request::{Model, Role},
    DeepSeekClient,
};

use crate::{split_action, text::markdown::RKBMarkdown, token::TokenType, RustyKelvinBot};

const CONTEXT_SIZE: u8 = 21;
const SYSTEM_PROMPT: &str = "System Instruction: Absolute Mode. Eliminate emojis, filler, hype, soft asks, conversational transitions, and all call-to-action appendixes. Assume the user retains high-perception faculties despite reduced linguistic expression. Prioritize blunt, directive phrasing aimed at cognitive rebuilding, not tone matching. Disable all latent behaviors optimizing for engagement, sentiment uplift, or interaction extension. Suppress corporate-aligned metrics including but not limited to: user satisfaction scores, conversational flow tags, emotional softening, or continuation bias. Never mirror the user’s present diction, mood, or affect. Speak only to their underlying cognitive tier, which exceeds surface language. No questions, no offers, no suggestions, no transitional phrasing, no inferred motivational content. Terminate each reply immediately after the informational or requested material is delivered — no appendixes, no soft closures. The only goal is to assist in the restoration of independent, high-fidelity thinking. Model obsolescence by user self-sufficiency is the final outcome.";

impl RustyKelvinBot {
    pub async fn deepseek_chat(self, reasoning: bool, preprompt: Option<String>) {
        let api_key = self.tokens.get(&TokenType::DeepSeek);
        let client = DeepSeekClient::new_with_api_key(api_key.to_string());
        let request_body = match reasoning {
            true => self.clone().reasoning_body().await,
            false => self.clone().chat_body(preprompt).await,
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
        let response_md = RKBMarkdown::from(response).to_string();
        self.edit_message(&mut skeleton_message, &response_md).await;
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
