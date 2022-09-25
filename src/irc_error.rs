#[derive(thiserror::Error, Debug)]
pub enum IrcError {
    #[error("Irc Error {0:?}")]
    IrcError(irc::error::Error)
}

pub fn convert_irc_error(error: irc::error::Error) -> anyhow::Error {
    anyhow::Error::from(IrcError::IrcError(error))
}
