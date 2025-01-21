use serde_json;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum VcsError {
    IOError(String),
    SerializationError(String),
    Other(String),
}

impl fmt::Display for VcsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VcsError::IOError(msg) => write!(f, "IO Error: {}", msg),
            VcsError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            VcsError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for VcsError {}

impl From<io::Error> for VcsError {
    fn from(error: io::Error) -> Self {
        VcsError::IOError(error.to_string())
    }
}

impl From<serde_json::Error> for VcsError {
    fn from(error: serde_json::Error) -> Self {
        VcsError::SerializationError(error.to_string())
    }
}

pub type VcsResult<T> = Result<T, VcsError>;
