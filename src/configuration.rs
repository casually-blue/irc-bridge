#[derive(Clone)]
pub struct Configuration {
    pub discord_channel_id: u64,
    pub irc_channel_name: String,
    pub irc_server_name: String,
    pub irc_server_port: u16,
    pub irc_nick: String,
    pub discord_bot_token: String,
}
