use serde_json;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum VCSError {
    IOError(String),
    SerializationError(String),
    Other(String),
    Uninitialized,
}

impl fmt::Display for VCSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VCSError::Uninitialized => write!(f, "VCS Uninitialized!: run rust-vcs init"),
            VCSError::IOError(msg) => write!(f, "IO Error: {}", msg),
            VCSError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            VCSError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for VCSError {}

impl From<io::Error> for VCSError {
    fn from(error: io::Error) -> Self {
        VCSError::IOError(error.to_string())
    }
}

impl From<serde_json::Error> for VCSError {
    fn from(error: serde_json::Error) -> Self {
        VCSError::SerializationError(error.to_string())
    }
}

pub type VCSResult<T> = Result<T, VCSError>;
