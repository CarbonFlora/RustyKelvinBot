use crate::{err::RKBServiceRequestErr, RKBServiceRequest, ENTRY_STRING};

impl RKBServiceRequest {
    pub async fn help(self) -> Result<(), RKBServiceRequestErr> {
        let help_text = format!(
            "```USAGE: 
{}[ACTION] [CONTEXT]

ACTION:
CHAT   - Ask DeepSeek AI what you put in [CONTEXT].
REASON - Ask DeepSeek AI what you put in [CONTEXT].
TIMER  - Set a timer to trigger after time elapsed. (#d#h#m)```",
            ENTRY_STRING
        );
        self.try_send_message(help_text).await?;
        Ok(())
    }
}
