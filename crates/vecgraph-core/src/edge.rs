use crate::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub String);

impl EdgeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn from_source_target_kind(source_id: &NodeId, target_id: &NodeId, kind: &str) -> Self {
        Self(format!(
            "{}:{}:{}",
            source_id.as_str(),
            kind,
            target_id.as_str()
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T: Into<String>> From<T> for EdgeId {
    fn from(val: T) -> Self {
        Self(val.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source_node_id: NodeId,
    pub target_node_id: NodeId,
    pub kind: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

impl Edge {
    pub fn new(
        source_node_id: impl Into<NodeId>,
        target_node_id: impl Into<NodeId>,
        kind: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        let source_node_id: NodeId = source_node_id.into();
        let target_node_id: NodeId = target_node_id.into();
        let kind: String = kind.into();
        let id = EdgeId::from_source_target_kind(&source_node_id, &target_node_id, &kind);

        Self {
            id,
            source_node_id,
            target_node_id,
            kind: kind,
            content: content.into(),
            metadata: None,
        }
    }

    pub fn with_suffix(
        source_node_id: impl Into<NodeId>,
        target_node_id: impl Into<NodeId>,
        kind: impl Into<String>,
        suffix: impl std::fmt::Display,
        content: impl Into<String>,
    ) -> Self {
        let source_node_id: NodeId = source_node_id.into();
        let target_node_id: NodeId = target_node_id.into();
        let kind: String = kind.into();
        let id = EdgeId::new(format!(
            "{}:{}:{}:{}",
            source_node_id.as_str(),
            kind,
            target_node_id.as_str(),
            suffix
        ));

        Self {
            id,
            source_node_id,
            target_node_id,
            kind: kind,
            content: content.into(),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeWithVector {
    pub edge: Edge,
    pub vector: Vec<f32>,
}

impl EdgeWithVector {
    pub fn new(edge: Edge, vector: Vec<f32>) -> Self {
        Self { edge, vector }
    }
}
