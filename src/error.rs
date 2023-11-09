use std::array::TryFromSliceError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvError {
    #[error("IO Of File")]
    IOError(#[from] std::io::Error),
    #[error("End Of File")]
    EOF,
    #[error("Unknown error")]
    Unknown,
    #[error("{0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    #[error("Slice Decode Error: {0}")]
    SliceDecodeError(#[from] TryFromSliceError),
}

pub type Result<T> = std::result::Result<T, KvError>;