use thiserror::Error;

use crate::{store::kv_error::KvError, user::user_error::UserError};

#[derive(Debug, Error)]
pub enum RorError {
    #[error("Error from storage module: {0}")]
    KvError(#[from] KvError),
    #[error("Error from user module: {0}")]
    UserError(#[from] UserError),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    TomlDeError(#[from] toml::de::Error),
    #[error("{0}")]
    TomlSeError(#[from] toml::ser::Error),
    #[error("{0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    #[error("Unable to connect to server: {0}")]
    ConnectFailed(std::io::Error),
    #[error("Server cannot to open datafile")]
    OpenFileFailed,
    #[error("Server cannot parse the request correctly")]
    RequestError,
    #[error("Server cannot parse the path correctly")]
    PathError,
    #[error("Server encountered an unexpected error")]
    ServerError,
    #[error("The client actively disconnected")]
    Disconnect,
    #[error("Unable to communicate with the server, the connection may be interrupted, you can try to reconnect or check the server")]
    ConnectionLost,
    #[error("Unable to parse data, probably it is incomplete")]
    IncompleteData,
}
pub type Result<T> = std::result::Result<T, RorError>;
