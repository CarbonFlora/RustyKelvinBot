use std::thread::sleep;
use std::time::Duration;

use anyhow::Context as _;
use rustykelvinbot::RustyKelvinBot;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let rkb = RustyKelvinBot::new(ctx, msg);
        if !rkb.is_user_message().await {
            return;
        }
        if rkb.clone().pinned_handle_message().await {
            return;
        }
        rkb.clone().handle_message().await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        sleep(Duration::from_secs(2));
        let rkb = RustyKelvinBot::new(ctx, Message::default());
        tokio::spawn(rkb.startup_refresh_timers());
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
