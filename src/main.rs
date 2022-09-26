use irc::client::{prelude::Config, Client};
//use shuttle_service::Service;
//use sqlx::PgPool;
use tokio::sync::mpsc::unbounded_channel;

use irc_bot::configuration::Configuration;


pub struct IrcBot {
//    #[allow(unused)]
    //pool: PgPool
}


impl IrcBot {
    pub fn new() -> Self {
        IrcBot {}
    }

    pub async fn run(self: Box<Self>) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = Configuration {
            discord_channel_id: str::parse::<u64>(std::env::var("CHAN_ID").unwrap().as_str()).unwrap(),
            irc_channel_name: "#openutd".into(),
            irc_server_name: "irc.oftc.net".into(),
            irc_server_port: 6697,
            irc_nick: "openutd-irc-bot".into(),
            discord_bot_token: std::env::var("DISCORD_BOT_TOKEN").unwrap(),
        };

        let irc_config = Config {
            nickname: Some("openutd-irc-bot".to_owned()),
            server: Some("irc.oftc.net".to_owned()),
            channels: vec!["#openutd".to_owned()],
            port: Some(6697),
            use_tls: Some(true),
            ..Default::default()
        };

        let mut irc_client = Client::from_config(irc_config).await.unwrap();
        irc_client.identify().unwrap();
        irc_client.send_join("#openutd").unwrap();

        println!("initialized client");

        let (discord_message_forwarder, discord_bot_message_stream) = unbounded_channel();
        let (irc_message_forwarder, irc_bot_message_stream) = unbounded_channel();

        tokio::select! {
            _ = tokio::task::spawn(irc_bot::discord_side::discord_bot_send_side(
                    cfg.clone(), 
                    discord_bot_message_stream)) => {},
            _ = tokio::task::spawn(irc_bot::discord_side::discord_bot_receive_side(
                    cfg.clone(), 
                    irc_message_forwarder.clone(),
                    discord_message_forwarder.clone(), 
                    )) => {}
            _ = tokio::task::spawn(irc_bot::irc_side::irc_bot_send_side(
                    cfg.clone(), 
                    irc_client.sender(), 
                    irc_bot_message_stream)) => {},
            _ = tokio::task::spawn(irc_bot::irc_side::irc_bot_receive_side(
                    cfg.clone(), 
                    irc_client.stream().unwrap(), 
                    irc_client.sender(),
                    discord_message_forwarder.clone(), 
                    )) => {},
        }

        Ok(())
    }
}

#[tokio::main]
pub async fn main() {
    let bot = IrcBot::new();
    Box::new(bot).run().await.unwrap();


}

/*impl Service for IrcBot {
    fn bind< 'async_trait>(self: Box<Self>, addr:std::net::SocketAddr) ->  core::pin::Pin<Box<ServiceOutput<'async_trait>>>where Self: 'async_trait {
        Box::pin(self.run(addr))
    }
}
*/
