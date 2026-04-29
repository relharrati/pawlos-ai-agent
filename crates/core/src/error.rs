use thiserror::Error;

#[derive(Error, Debug)]
pub enum PawlosError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for PawlosError {
    fn from(e: anyhow::Error) -> Self {
        PawlosError::Other(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PawlosError>;
