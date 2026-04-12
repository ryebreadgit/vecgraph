use ndarray::Array2;
use ort::{session::Session, value::TensorRef};
use std::sync::Mutex;
use tokenizers::Tokenizer;
use vecgraph_core::{Embedder, VecGraphError};

pub struct OnnxEmbedderSettings {
    pub model_path: String,
    pub tokenizer_path: String,
    pub embed_dim: usize,
}

pub struct OnnxEmbedder {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
    embed_dim: usize,
}

impl OnnxEmbedder {
    pub fn new(settings: OnnxEmbedderSettings) -> Result<Self, VecGraphError> {
        let session = Session::builder()
            .map_err(|e| {
                VecGraphError::Other(format!("Failed to create ONNX session builder: {e}"))
            })?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
            .map_err(|e| VecGraphError::Other(format!("Failed to set optimization level: {e}")))?
            .with_intra_threads(4)
            .map_err(|e| VecGraphError::Other(format!("Failed to set intra threads: {e}")))?
            .commit_from_file(&settings.model_path)
            .map_err(|e| {
                VecGraphError::Other(format!("Failed to commit session from file: {e}"))
            })?;

        let tokenizer = Tokenizer::from_file(&settings.tokenizer_path)
            .map_err(|e| VecGraphError::Other(format!("Failed to load tokenizer: {e}")))?;

        let embed_dim = settings.embed_dim;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
            embed_dim,
        })
    }
}

impl Embedder for OnnxEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, VecGraphError> {
        let prefixed = format!("Document: {text}");

        let encoding = self
            .tokenizer
            .encode(prefixed, true)
            .map_err(|e| VecGraphError::TokenizerError(format!("Tokenizer error: {e}")))?;

        let ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let seq_len = ids.len();

        let input_ids = Array2::from_shape_vec((1, seq_len), ids).map_err(|e| {
            VecGraphError::TokenizerError(format!("Failed to create input_ids array: {e}"))
        })?;
        let attention_mask = Array2::from_shape_vec((1, seq_len), mask).map_err(|e| {
            VecGraphError::TokenizerError(format!("Failed to create attention_mask array: {e}"))
        })?;

        let input_ids_tensor =
            TensorRef::from_array_view(input_ids.view().into_dyn()).map_err(|e| {
                VecGraphError::TokenizerError(format!("Failed to create input_ids tensor: {e}"))
            })?;
        let attention_mask_tensor = TensorRef::from_array_view(attention_mask.view().into_dyn())
            .map_err(|e| {
                VecGraphError::TokenizerError(format!(
                    "Failed to create attention_mask tensor: {e}"
                ))
            })?;

        let (shape, raw_data) = {
            let mut session = self
                .session
                .lock()
                .map_err(|e| VecGraphError::Other(format!("session lock poisoned: {e}")))?;

            let outputs = session
                .run(ort::inputs![
                    "input_ids" => input_ids_tensor,
                    "attention_mask" => attention_mask_tensor,
                ])
                .map_err(|e| VecGraphError::EmbedderError(format!("ONNX inference error: {e}")))?;

            let (shape, data) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
                VecGraphError::EmbedderError(format!("Failed to extract output tensor: {e}"))
            })?;
            let shape: Vec<usize> = shape.iter().map(|&d| d as usize).collect();
            let data: Vec<f32> = data.to_vec();
            (shape, data)
        };

        let hidden_dim = shape[2];

        let last_token_idx = encoding
            .get_attention_mask()
            .iter()
            .rposition(|&m| m == 1)
            .unwrap_or(seq_len - 1);

        let offset = last_token_idx * hidden_dim;
        let embedding_full = &raw_data[offset..offset + hidden_dim];

        let truncated: Vec<f32> = embedding_full
            .iter()
            .take(self.embed_dim)
            .copied()
            .collect();

        let norm: f32 = truncated.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = if norm > 0.0 {
            truncated.iter().map(|x| x / norm).collect()
        } else {
            truncated
        };

        Ok(normalized)
    }

    fn dimensions(&self) -> usize {
        self.embed_dim
    }
}
