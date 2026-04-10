use crate::NodeId;
use crate::error::VecGraphError;

#[derive(Debug)]
pub struct SearchResult {
    pub node_id: NodeId,
    pub edge_kind: String,
    pub score: f32,
}

pub struct RerankParams {
    pub vector: Vec<f32>,
    pub edge_kind: String,
    pub weight: f32,
}

pub struct SearchQuery {
    pub query_vec: Vec<f32>,
    pub edge_kind: String,
    pub namespace: Option<String>,
    pub top_k: usize,
    pub exclude_names: Vec<String>,
    pub rerank: Option<RerankParams>,
}

pub struct ScoredHit {
    pub node_id_bytes: Vec<u8>,
    pub edge_kind: String,
    pub score: f32,
}

impl SearchQuery {
    pub fn new(query_vec: Vec<f32>, edge_kind: impl Into<String>, top_k: usize) -> Self {
        Self {
            query_vec,
            edge_kind: edge_kind.into(),
            namespace: None,
            top_k,
            exclude_names: Vec::new(),
            rerank: None,
        }
    }

    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn with_excludes(mut self, names: Vec<String>) -> Self {
        self.exclude_names = names;
        self
    }

    pub fn with_rerank(
        mut self,
        vector: Vec<f32>,
        edge_kind: impl Into<String>,
        weight: f32,
    ) -> Self {
        self.rerank = Some(RerankParams {
            vector,
            edge_kind: edge_kind.into(),
            weight: weight.clamp(0.0, 1.0),
        });
        self
    }
}

impl PartialEq for ScoredHit {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for ScoredHit {}

impl PartialOrd for ScoredHit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredHit {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Max-heap: worst score at the top so we can evict it
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

// Helper functions for vector math - AI Generated as I'm not great at this math

pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "vector dimension mismatch");

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        return 1.0;
    }

    1.0 - (dot / denom)
}

pub fn normalize(vec: &mut [f32]) {
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
}

pub fn build_base_vector(
    named_vecs: &[Vec<f32>],
    free_text_vec: Option<&[f32]>,
    named_weight: f32,
) -> Result<Vec<f32>, VecGraphError> {
    let text_weight = 1.0 - named_weight;
    let has_named = !named_vecs.is_empty();
    let has_text = free_text_vec.is_some();

    if !has_named && !has_text {
        return Err(VecGraphError::EmptyQuery);
    }

    // Text only
    if !has_named {
        return Ok(free_text_vec.unwrap().to_vec());
    }

    // Compute named centroid
    let dim = named_vecs[0].len();
    let count = named_vecs.len() as f32;
    let mut centroid = vec![0.0f32; dim];
    for v in named_vecs {
        for (c, val) in centroid.iter_mut().zip(v.iter()) {
            *c += val / count;
        }
    }

    // Named only
    if !has_text {
        normalize(&mut centroid);
        return Ok(centroid);
    }

    // Both — weighted blend
    let text_vec = free_text_vec.unwrap();
    for i in 0..dim {
        centroid[i] = named_weight * centroid[i] + text_weight * text_vec[i];
    }
    normalize(&mut centroid);

    Ok(centroid)
}
