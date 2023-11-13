use std::{array::TryFromSliceError, string::FromUtf8Error};
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
    #[error("FromUtf8 Error: {0}")]
    DecodeUtf8Error(#[from] FromUtf8Error),
    #[error("Key not found: \"{0}\"")]
    KeyNotFound(String),
}

pub type Result<T> = std::result::Result<T, KvError>;
