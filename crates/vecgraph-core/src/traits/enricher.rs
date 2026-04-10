use crate::Node;
use crate::error::VecGraphError;

pub struct EnrichmentResult {
    pub edge_kind: String,
    pub text: String,
}

impl EnrichmentResult {
    pub fn new(edge_kind: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            edge_kind: edge_kind.into(),
            text: text.into(),
        }
    }
}

pub trait Enricher: Send + Sync {
    fn enrich(&self, node: &Node) -> Result<Vec<EnrichmentResult>, VecGraphError>;
}
