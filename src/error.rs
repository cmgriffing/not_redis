use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Cannot parse value")]
    ParseError,

    #[error("No such key: {0}")]
    NoSuchKey(String),

    #[error("Wrong type operation")]
    WrongType,

    #[error("Not supported command")]
    NotSupported,

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type RedisResult<T> = Result<T, RedisError>;
