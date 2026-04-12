#[derive(Debug, Clone)]
pub enum PoolingStrategy {
    CLS,
    Mean,
    Last,
}

pub enum PrefixMode {
    /// No prefix.
    None,
    /// Same prefix for all text.
    Symmetric(String),
    /// Different prefixes for ingest vs query.
    Asymmetric { document: String, query: String },
}

#[derive(Debug, Clone)]
pub enum MatryoshkaMode {
    Off,
    Truncate { dim: usize },
    LayerNormThenTruncate { dim: usize },
}

#[derive(Debug, Clone)]
pub struct InputTensorNames {
    pub input_ids: String,
    pub attention_mask: String,
    pub token_type_ids: Option<String>,
}

impl Default for InputTensorNames {
    fn default() -> Self {
        Self {
            input_ids: "input_ids".to_string(),
            attention_mask: "attention_mask".to_string(),
            token_type_ids: None,
        }
    }
}

pub struct OnnxEmbedderSettings {
    /// Path to the ONNX model file.
    pub model_path: String,
    /// Path to the tokenizer file (tokenizer.json).
    pub tokenizer_path: String,
    /// Dimensionality of the output embeddings (after any matryoshka processing).
    pub embed_dim: usize,
    /// Pooling strategy to use when converting token embeddings to a single vector.
    pub pooling: PoolingStrategy,
    /// Optional prefixing strategy for input text.
    pub prefix: PrefixMode,
    /// Optional "matryoshka" mode for handling models that produce higher-dimensional outputs.
    pub matryoshka: MatryoshkaMode,
    /// Whether to L2-normalize the output embeddings.
    pub normalize: bool,
    /// Names of the input tensors expected by the ONNX model.
    pub input_names: InputTensorNames,
    /// Number of threads to use for ONNX inference.
    pub intra_threads: usize,
}
