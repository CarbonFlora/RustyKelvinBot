use crate::{RustyKelvinBot, ENTRY_STRING};

impl RustyKelvinBot {
    pub async fn help(self) {
        let help_text = format!(
            "```USAGE: 
{}[ACTION] [CONTEXT]

ACTION:
CHAT   - Ask DeepSeek AI what you put in [CONTEXT].
REASON - Ask DeepSeek AI what you put in [CONTEXT].
TIMER  - Set a timer to trigger after time elapsed. (#d#h#m)```",
            ENTRY_STRING
        );
        self.send_message(help_text).await;
    }
}
