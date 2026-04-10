#[derive(Debug)]
pub enum VecGraphError {
    DimensionMismatch { expected: usize, got: usize },
    EmbedderError(String),
    EmptyQuery,
    Other(String),
}

impl std::fmt::Display for VecGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VecGraphError::DimensionMismatch { expected, got } => {
                write!(f, "Dimension mismatch: expected {}, got {}", expected, got)
            }
            VecGraphError::EmbedderError(msg) => write!(f, "Embedder error: {}", msg),
            VecGraphError::Other(msg) => write!(f, "Error: {}", msg),
            VecGraphError::EmptyQuery => write!(f, "Query is empty"),
        }
    }
}

impl std::error::Error for VecGraphError {}
