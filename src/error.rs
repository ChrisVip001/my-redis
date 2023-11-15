use thiserror::Error;

use crate::{store::kv_error::KvError, user::user_error::UserError};

#[derive(Debug, Error)]
pub enum RorError {
    #[error("Error from storage module: {0}")]
    KvError(#[from] KvError),
    #[error("Error from user module: {0}")]
    UserError(#[from] UserError),
}
pub type Result<T> = std::result::Result<T, RorError>;
