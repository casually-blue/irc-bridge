use futures::StreamExt;
use irc::{client::{ClientStream, prelude::Config, Client}, proto::Command};
use linkify::{LinkFinder, LinkKind};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::configuration::Configuration;

pub async fn irc_bot_send_side(cfg: Configuration, irc_sender: irc::client::Sender, mut bot_message_stream: UnboundedReceiver<String>) -> Result<(), anyhow::Error> {
    while let Some(message) = bot_message_stream.recv().await {
        irc_sender.send_privmsg(cfg.irc_channel_name.as_str(), message.clone())?;
        irc_message_response(cfg.irc_channel_name.clone(), message, irc_sender.clone()).await;
    }
    Ok(())
}

pub async fn irc_message_response(channel: String, message: String, irc_sender: irc::client::Sender){
    let links: Vec<_> = LinkFinder::new().kinds(&[LinkKind::Url]).links(message.as_str()).collect();
    for link in links {
        let Ok(url) = url::Url::parse(link.as_str()) else {
            continue;
        };
        
        let Some(host) = url::Url::host_str(&url) else {
            continue;
        };

        let cert = checkssl::CheckSSL::from_domain(host);
        if cert.is_ok() && cert.unwrap().server.is_valid {
        if let Ok(info) = opengraph::scrape(link.as_str(), Default::default()) {
            if let Some(description) = info.description {
                let _ = irc_sender.send_privmsg(channel.clone(), format!("[ {} ]", description));
            }
        }
        } else {
            let _ = irc_sender.send_privmsg(channel.clone(), "Error, certificate has expired!".to_string());
        }
    }
}

pub async fn irc_bot_receive_side(cfg: Configuration, mut irc_stream: ClientStream, irc_sender: irc::client::Sender, discord_message_forwarder: UnboundedSender<String>) -> Result<(), anyhow::Error> {
    while let Some(message) = irc_stream.next().await.transpose().unwrap() {
        if let Command::PRIVMSG(channel, msg) = message.clone().command {
            if message.source_nickname() != Some(cfg.irc_nick.as_str()) {
                discord_message_forwarder.send(msg.clone()).unwrap();
            }

            irc_message_response(channel, msg, irc_sender.clone()).await;
        } 
    }
    Ok(())
}
