use crate::{MatryoshkaMode, OnnxEmbedderSettings, PoolingStrategy, PrefixMode};
use ndarray::Array2;
use ort::{session::Session, value::TensorRef};
use std::sync::Mutex;
use tokenizers::Tokenizer;
use vecgraph_core::{Embedder, VecGraphError};

pub struct OnnxEmbedder {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
    embed_dim: usize,
    settings: OnnxEmbedderSettings,
}

impl OnnxEmbedder {
    pub fn new(settings: OnnxEmbedderSettings) -> Result<Self, VecGraphError> {
        let session = Session::builder()
            .map_err(|e| {
                VecGraphError::Other(format!("Failed to create ONNX session builder: {e}"))
            })?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
            .map_err(|e| VecGraphError::Other(format!("Failed to set optimization level: {e}")))?
            .with_intra_threads(settings.intra_threads)
            .map_err(|e| VecGraphError::Other(format!("Failed to set intra threads: {e}")))?
            .commit_from_file(&settings.model_path)
            .map_err(|e| {
                VecGraphError::Other(format!("Failed to commit session from file: {e}"))
            })?;

        let tokenizer = Tokenizer::from_file(&settings.tokenizer_path)
            .map_err(|e| VecGraphError::Other(format!("Failed to load tokenizer: {e}")))?;

        let embed_dim = match &settings.matryoshka {
            MatryoshkaMode::Off => settings.embed_dim,
            MatryoshkaMode::Truncate { dim } => *dim,
            MatryoshkaMode::LayerNormThenTruncate { dim } => *dim,
        };

        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
            embed_dim,
            settings,
        })
    }

    pub fn embed_query(&self, text: &str) -> Result<Vec<f32>, VecGraphError> {
        let prefixed = match &self.settings.prefix {
            PrefixMode::None => text.to_string(),
            PrefixMode::Symmetric(prefix) => format!("{prefix}{text}"),
            PrefixMode::Asymmetric { query, .. } => format!("{query}{text}"),
        };
        self.embed_raw(&prefixed)
    }

    fn apply_document_prefix(&self, text: &str) -> String {
        match &self.settings.prefix {
            PrefixMode::None => text.to_string(),
            PrefixMode::Symmetric(prefix) => format!("{prefix}{text}"),
            PrefixMode::Asymmetric { document, .. } => format!("{document}{text}"),
        }
    }

    fn embed_raw(&self, text: &str) -> Result<Vec<f32>, VecGraphError> {
        let encoding = self
            .tokenizer
            .encode(text, true)
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
                    &self.settings.input_names.input_ids => input_ids_tensor,
                    &self.settings.input_names.attention_mask => attention_mask_tensor,
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
        let mut embedding_full = match self.settings.pooling {
            PoolingStrategy::CLS => raw_data[0..hidden_dim].to_vec(),
            PoolingStrategy::Mean => {
                // Mean polling AI generated
                let attention = encoding.get_attention_mask();
                let mut mean_vec = vec![0.0f32; hidden_dim];
                let mut count = 0.0f32;
                for (i, &m) in attention.iter().enumerate() {
                    if m == 1 {
                        let offset = i * hidden_dim;
                        for j in 0..hidden_dim {
                            mean_vec[j] += raw_data[offset + j];
                        }
                        count += 1.0;
                    }
                }
                if count > 0.0 {
                    for j in 0..hidden_dim {
                        mean_vec[j] /= count;
                    }
                }
                mean_vec
            }
            PoolingStrategy::Last => {
                let last_token_idx = encoding
                    .get_attention_mask()
                    .iter()
                    .rposition(|&m| m == 1)
                    .unwrap_or(seq_len - 1);

                let offset = last_token_idx * hidden_dim;
                raw_data[offset..offset + hidden_dim].to_vec()
            }
        };

        let mut embedding = match &self.settings.matryoshka {
            MatryoshkaMode::Off => embedding_full,
            MatryoshkaMode::Truncate { dim } => {
                embedding_full.truncate(*dim);
                embedding_full
            }
            MatryoshkaMode::LayerNormThenTruncate { dim } => {
                layer_norm_inplace(&mut embedding_full);
                embedding_full.truncate(*dim);
                embedding_full
            }
        };

        if self.settings.normalize {
            l2_normalize_inplace(&mut embedding);
        }

        Ok(embedding)
    }
}

impl Embedder for OnnxEmbedder {
    fn embed(&self, input: &str) -> Result<Vec<f32>, VecGraphError> {
        let prefixed = self.apply_document_prefix(input);
        self.embed_raw(&prefixed)
    }

    fn dimensions(&self) -> usize {
        self.embed_dim
    }
}

fn l2_normalize_inplace(vec: &mut [f32]) {
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
}

// Function AI generated
fn layer_norm_inplace(vec: &mut [f32]) {
    let n: f32 = vec.len() as f32;
    let mean: f32 = vec.iter().sum::<f32>() / n;
    let variance: f32 = vec.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / n;
    let std = (variance + 1e-5).sqrt();
    for x in vec.iter_mut() {
        *x = (*x - mean) / std;
    }
}
