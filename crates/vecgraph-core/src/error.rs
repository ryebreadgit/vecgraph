#[derive(Debug)]
pub enum VecGraphError {
    DimensionMismatch { expected: usize, got: usize },
    EmbedderError(String),
    EmptyQuery,
    StorageError(String),
    SerializationError(String),
    TokenizerError(String),
    Other(String),
}

impl std::fmt::Display for VecGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VecGraphError::DimensionMismatch { expected, got } => {
                write!(f, "Dimension mismatch: expected {}, got {}", expected, got)
            }
            VecGraphError::EmbedderError(msg) => write!(f, "Embedder error: {}", msg),
            VecGraphError::TokenizerError(msg) => write!(f, "Tokenizer error: {}", msg),
            VecGraphError::Other(msg) => write!(f, "Error: {}", msg),
            VecGraphError::EmptyQuery => write!(f, "Query is empty"),
            VecGraphError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            VecGraphError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for VecGraphError {}
