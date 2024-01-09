use thiserror::Error;

use crate::{store::kv_error::KvError, user::user_error::UserError, cmd::{statement::ValueType, cmd_error::CmdError}, request::OperateRequest};

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
    #[error("{0}")]
    CmdError(#[from] CmdError),
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
    ConnectionLost(OperateRequest),
    #[error("Unable to parse data, probably it is incomplete")]
    IncompleteData,
    #[error("Datafile Not found :{0}")]
    DataFileNotFound(String),
    #[error("Cannot convert '{0}' to {1}")]
    ConvertError(String, ValueType),
    #[error("nable to communicate with server, possibly high latency or lost connection, please try this operation again")]
    AbnormalConnection,
}
pub type Result<T> = std::result::Result<T, RorError>;
