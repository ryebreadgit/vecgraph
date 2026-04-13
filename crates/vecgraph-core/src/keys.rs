use crate::{EdgeId, NodeId, SearchKind};
pub enum StorageKey {
    Node(NodeId),
    NodeVector {
        kind: String,
        namespace: Option<String>,
        node_id: NodeId,
    },

    Edge(EdgeId),
    EdgeVector {
        edge_kind: String,
        namespace: Option<String>,
        node_id: NodeId,
    },

    EdgesForNode(NodeId),
    EdgesTargetingNode(NodeId),

    NameMapping {
        kind: String,
        name: String,
    },
}

impl StorageKey {
    pub fn partition(&self) -> &'static str {
        match self {
            StorageKey::Node(_) => "nodes",
            StorageKey::NodeVector { .. } => "node_vectors",
            StorageKey::Edge(_) => "edges",
            StorageKey::EdgeVector { .. } => "edge_vectors",
            StorageKey::EdgesForNode(_) => "edges_for_node",
            StorageKey::EdgesTargetingNode(_) => "edges_targeting_node",
            StorageKey::NameMapping { .. } => "names",
        }
    }

    pub fn key(&self) -> String {
        match self {
            StorageKey::Node(id) => format!("node:{}", id.as_str()),
            StorageKey::NodeVector {
                kind,
                namespace,
                node_id,
            } => {
                let namespace = namespace.as_deref().unwrap_or("_"); // Default to _ for no namespace
                format!("vec:{}:{}:{}", kind, namespace, node_id.as_str())
            }
            StorageKey::Edge(id) => format!("edge:{}", id.as_str()),
            StorageKey::EdgeVector {
                edge_kind,
                namespace,
                node_id,
            } => {
                let namespace = namespace.as_deref().unwrap_or("_"); // Default to _ for no namespace
                format!("vec:{}:{}:{}", edge_kind, namespace, node_id.as_str())
            }
            StorageKey::EdgesForNode(node_id) => format!("edges_for_node:{}", node_id.as_str()),
            StorageKey::EdgesTargetingNode(node_id) => {
                format!("edges_targeting_node:{}", node_id.as_str())
            }

            StorageKey::NameMapping { kind, name } => {
                format!("name:{}:{}", kind, name)
            }
        }
    }
}

pub struct VectorScanQuery {
    pub kind: String,
    pub namespace: Option<String>,
    pub search_kind: SearchKind,
}

impl VectorScanQuery {
    pub fn new(edge_kind: impl Into<String>, search_kind: impl Into<SearchKind>) -> Self {
        Self {
            kind: edge_kind.into(),
            namespace: None,
            search_kind: search_kind.into(),
        }
    }

    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn partitions(&self) -> Vec<String> {
        let mut partitions = Vec::new();
        match self.search_kind {
            SearchKind::Edge => {
                partitions.push("edge_vectors".to_string());
            }
            SearchKind::Node => {
                partitions.push("node_vectors".to_string());
            }
            SearchKind::All => {
                partitions.push("edge_vectors".to_string());
                partitions.push("node_vectors".to_string());
            }
        }
        partitions
    }

    pub fn scan_prefix(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("vec:{}:{}:", self.kind, ns),
            None => format!("vec:{}:", self.kind),
        }
    }

    pub fn search_kind_from_partition(partition: &str) -> Option<SearchKind> {
        match partition {
            "node_vectors" => Some(SearchKind::Node),
            "edge_vectors" => Some(SearchKind::Edge),
            _ => None,
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
