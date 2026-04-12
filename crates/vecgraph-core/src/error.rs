#[derive(Debug, thiserror::Error)]
pub enum VecGraphError {
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("Embedder error: {0}")]
    EmbedderError(String),
    #[error("Query is empty")]
    EmptyQuery,
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Tokenizer error: {0}")]
    TokenizerError(String),
    #[error("Error: {0}")]
    Other(String),
}
