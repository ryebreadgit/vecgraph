use crate::{EdgeId, NodeId};
pub enum StorageKey {
    Node(NodeId),

    Edge(EdgeId),

    Vector {
        edge_kind: String,
        namespace: Option<String>,
        node_id: NodeId,
    },

    EdgesForNode(NodeId),

    NameMapping {
        kind: String,
        name: String,
    },
}

impl StorageKey {
    pub fn partition(&self) -> &'static str {
        match self {
            StorageKey::Node(_) => "nodes",
            StorageKey::Edge(_) => "edges",
            StorageKey::Vector { .. } => "vectors",
            StorageKey::EdgesForNode(_) => "edges_for_node",
            StorageKey::NameMapping { .. } => "names",
        }
    }

    pub fn key(&self) -> String {
        match self {
            StorageKey::Node(id) => format!("node:{}", id.as_str()),
            StorageKey::Edge(id) => format!("edge:{}", id.as_str()),
            StorageKey::Vector {
                edge_kind,
                namespace,
                node_id,
            } => {
                let namespace = namespace.as_deref().unwrap_or("_"); // Default to _ for no namespace
                format!("vec:{}:{}:{}", edge_kind, namespace, node_id.as_str())
            }
            StorageKey::EdgesForNode(node_id) => format!("edges_for_node:{}", node_id.as_str()),
            StorageKey::NameMapping { kind, name } => {
                format!("name:{}:{}", kind, name)
            }
        }
    }
}

pub struct VectorScanQuery {
    pub edge_kind: String,
    pub namespace: Option<String>,
}

impl VectorScanQuery {
    pub fn new(edge_kind: impl Into<String>) -> Self {
        Self {
            edge_kind: edge_kind.into(),
            namespace: None,
        }
    }

    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn partition(&self) -> &'static str {
        "vectors"
    }

    pub fn scan_prefix(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("vec:{}:{}:", self.edge_kind, ns),
            None => format!("vec:{}:", self.edge_kind),
        }
    }

    pub fn node_id_from_key(key: &str) -> Option<&str> {
        // Split the key into at most 4 parts: "vec", edge_kind, namespace (or _), node_id
        let mut parts = key.splitn(4, ':');
        let _prefix = parts.next()?;
        let _edge_kind = parts.next()?;
        let _namespace = parts.next()?;
        parts.next() // node_id
    }
}
