use serenity::{prelude::{Context, EventHandler, GatewayIntents}, model::{prelude::{ChannelId, Message}, user::User}, http::Http, async_trait, Client};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::configuration::Configuration;

pub async fn discord_bot_send_side(cfg: Configuration, mut bot_message_stream: UnboundedReceiver<String>) -> Result<(), anyhow::Error> {
    let connection = Http::new(cfg.discord_bot_token.as_ref());
    while let Some(message) = bot_message_stream.recv().await {
        ChannelId(cfg.discord_channel_id)
            .send_message(&connection, |m| {
                m.content(message) 
            }).await?;
    }

    Ok(())
}


pub struct DiscordBot {
    configuration: Configuration,
    irc_forwarder: UnboundedSender<String>,
    #[allow(unused)]
    bot_message_forwarder: UnboundedSender<String>,
}

#[async_trait]
impl EventHandler for DiscordBot {
    async fn message(&self, ctx: Context, msg: Message) {
        let user = ctx.cache.current_user_id().to_user(&ctx).await.unwrap();
        if msg.author != user && msg.channel_id == self.configuration.discord_channel_id {
            println!("sending to irc {}", msg.content);
            self.irc_forwarder.send(format!("<{}> {}", msg.author.name, msg.content)).unwrap();
        }
    }
}

pub async fn discord_bot_receive_side(cfg: Configuration, irc_forwarder: UnboundedSender<String>, bot_message_forwarder: UnboundedSender<String>) -> Result<(), serenity::Error> {
    let mut client = Client::builder(cfg.clone().discord_bot_token, GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES)
        .event_handler(DiscordBot{configuration: cfg, irc_forwarder, bot_message_forwarder})
        .await?;
    client.start().await
}
