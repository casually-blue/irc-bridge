use futures::StreamExt;
use irc::{client::ClientStream, proto::Command};
use linkify::{LinkFinder, LinkKind};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::configuration::Configuration;

pub async fn irc_bot_send_side(cfg: Configuration, irc_sender: irc::client::Sender, mut bot_message_stream: UnboundedReceiver<String>) -> Result<(), anyhow::Error> {
    while let Some(message) = bot_message_stream.recv().await {
        println!("sending message ( {} )", message);
        irc_sender.send_privmsg(cfg.irc_channel_name.as_str(), message)?;
    }
    Ok(())
}

pub async fn irc_bot_receive_side(cfg: Configuration, mut irc_stream: ClientStream, irc_sender: irc::client::Sender, discord_message_forwarder: UnboundedSender<String>) -> Result<(), anyhow::Error> {
    while let Some(message) = irc_stream.next().await.transpose().unwrap() {
        if let Command::PRIVMSG(channel, msg) = message.clone().command {
            if message.source_nickname() != Some(cfg.irc_nick.as_str()) {
                discord_message_forwarder.send(msg.clone()).unwrap();
            }

            let links: Vec<_> = LinkFinder::new().kinds(&[LinkKind::Url]).links(msg.as_str()).collect();
            for link in links {
                if let Ok(info) = opengraph::scrape(link.as_str(), Default::default()) {
                    if let Some(description) = info.description {
                        let _ = irc_sender.send_privmsg(channel.clone(), format!("[ {} ]", description));
                    }

                }
            }
        } 
    }
    Ok(())
}
