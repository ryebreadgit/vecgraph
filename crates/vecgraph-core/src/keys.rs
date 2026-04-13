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
        source_node_id: NodeId,
        target_node_id: NodeId,
    },

    EdgesForNode {
        node_id: NodeId,
        edge_id: EdgeId,
    },

    EdgesTargetingNode {
        node_id: NodeId,
        edge_id: EdgeId,
    },

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
            StorageKey::EdgesForNode { .. } => "edges_for_node",
            StorageKey::EdgesTargetingNode { .. } => "edges_targeting_node",
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
                let namespace = namespace.as_deref().unwrap_or("_");
                format!("vec:{}:{}:{}", kind, namespace, node_id.as_str())
            }
            StorageKey::Edge(id) => format!("edge:{}", id.as_str()),
            StorageKey::EdgeVector {
                edge_kind,
                namespace,
                source_node_id,
                target_node_id,
            } => {
                let namespace = namespace.as_deref().unwrap_or("_");
                format!(
                    "vec:{}:{}:{}:{}",
                    edge_kind,
                    namespace,
                    source_node_id.as_str(),
                    target_node_id.as_str()
                )
            }
            StorageKey::EdgesForNode { node_id, edge_id } => {
                format!("efn:{}:{}", node_id.as_str(), edge_id.as_str())
            }
            StorageKey::EdgesTargetingNode { node_id, edge_id } => {
                format!("etn:{}:{}", node_id.as_str(), edge_id.as_str())
            }
            StorageKey::NameMapping { kind, name } => {
                format!("name:{}:{}", kind, name)
            }
        }
    }

    pub fn edges_for_node_prefix(node_id: &NodeId) -> (&'static str, String) {
        ("edges_for_node", format!("efn:{}:", node_id.as_str()))
    }

    pub fn edges_targeting_node_prefix(node_id: &NodeId) -> (&'static str, String) {
        ("edges_targeting_node", format!("etn:{}:", node_id.as_str()))
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
        let mut parts = key.splitn(5, ':');
        let _prefix = parts.next()?; // "vec"
        let _kind = parts.next()?; // edge_kind or node kind
        let _namespace = parts.next()?; // namespace or "_"
        parts.next() // source_node_id (for edges) or node_id (for nodes)
    }
}
