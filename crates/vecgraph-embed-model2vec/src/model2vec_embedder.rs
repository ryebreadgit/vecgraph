use model2vec_rs::model::StaticModel;
use vecgraph_core::{Embedder, VecGraphError};

pub struct Model2VecEmbedderSettings {
    /// Path to the model directory or HuggingFace model ID.
    pub model_path: String,
    /// Whether to L2 normalize the output embeddings.
    pub normalize: bool,
    /// Max input token length. Inputs longer than this are truncated. Before embedding. None uses the model's default (typically 512).
    pub max_length: Option<usize>,
    /// Batch size for internal processing. This does not affect the batch size of the `embed_batch` method, which can be any size. This is only for controlling how many inputs are processed at once when the model is called, and can help with memory usage.
    pub batch_size: usize,
}

impl Default for Model2VecEmbedderSettings {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            normalize: true,
            max_length: None,
            batch_size: 1024,
        }
    }
}

pub struct Model2VecEmbedder {
    model: StaticModel,
    dimensions: usize,
    max_length: Option<usize>,
    batch_size: usize,
}

impl Model2VecEmbedder {
    pub fn new(settings: Model2VecEmbedderSettings) -> Result<Self, VecGraphError> {
        let model = StaticModel::from_pretrained(
            &settings.model_path,
            None,
            Some(settings.normalize),
            None,
        )
        .map_err(|e| VecGraphError::Other(format!("Failed to load Model2Vec model: {e}")))?;
        let probe = model.encode(&["test".to_string()]);
        let dimensions = probe
            .first()
            .ok_or_else(|| VecGraphError::Other("Model returned no embeddings on probe".into()))?
            .len();
        Ok(Self {
            model,
            dimensions,
            max_length: settings.max_length,
            batch_size: settings.batch_size,
        })
    }
}

impl Embedder for Model2VecEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, VecGraphError> {
        let embedding = self
            .embed_batch(&[text])?
            .into_iter()
            .next()
            .ok_or_else(|| VecGraphError::Other("Model returned no embeddings".into()))?;
        Ok(embedding)
    }
    fn embed_batch(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, VecGraphError> {
        let owned: Vec<String> = inputs.iter().map(|s| s.to_string()).collect();
        let embeddings = self
            .model
            .encode_with_args(&owned, self.max_length, self.batch_size);
        if embeddings.len() != inputs.len() {
            return Err(VecGraphError::EmbedderError(format!(
                "Batch size mismatch: got {}, expected {}",
                embeddings.len(),
                inputs.len()
            )));
        }
        Ok(embeddings)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}
