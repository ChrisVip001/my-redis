use thiserror::Error;
#[derive(Error, Debug)]
pub enum UserError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    TomlDeError(#[from] toml::de::Error),

    
    #[error("User '{0}' not found")]
    UserNotFound(String),
    #[error("Wrong user password")]
    WrongPassWord,
}