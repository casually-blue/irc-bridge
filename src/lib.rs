use crate::irc_bot::IrcBot;

use sqlx::PgPool;
use types::ShuttleIrcBot;

mod irc_error;
mod irc_bot;
mod types;

mod irc_side;
mod discord_side;

mod configuration;

#[shuttle_service::main]
async fn bot(#[shared::Postgres] pool: PgPool) -> ShuttleIrcBot {
    Ok(IrcBot::new(pool))
}
