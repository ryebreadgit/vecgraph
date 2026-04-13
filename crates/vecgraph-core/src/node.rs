use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T: Into<String>> From<T> for NodeId {
    fn from(val: T) -> Self {
        Self(val.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: String,
    pub namespace: Option<String>,
    pub name: String,
    pub payload: serde_json::Value,
}

impl Node {
    pub fn new(
        id: impl Into<NodeId>,
        kind: impl Into<String>,
        name: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            namespace: None,
            name: name.into(),
            payload,
        }
    }

    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn payload_as<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_value(self.payload.clone()).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWithVector {
    pub node: Node,
    pub vector: Vec<f32>,
}

impl NodeWithVector {
    pub fn new(node: Node, vector: Vec<f32>) -> Self {
        Self { node, vector }
    }
}
