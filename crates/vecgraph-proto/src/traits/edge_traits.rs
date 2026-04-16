use crate::{Edge, EdgeWithVector};
use vecgraph_core::VecGraphError;

impl From<Edge> for vecgraph_core::Edge {
    fn from(proto: Edge) -> Self {
        vecgraph_core::Edge {
            id: proto.id.into(),
            source_node_id: proto.source_node_id.into(),
            target_node_id: proto.target_node_id.into(),
            kind: proto.kind,
            content: proto.content,
            metadata: proto
                .metadata
                .and_then(|bytes| serde_json::from_slice(&bytes).ok()),
        }
    }
}

impl From<vecgraph_core::Edge> for Edge {
    fn from(core: vecgraph_core::Edge) -> Self {
        Edge {
            id: core.id.0,
            source_node_id: core.source_node_id.0,
            target_node_id: core.target_node_id.0,
            kind: core.kind,
            content: core.content,
            metadata: core
                .metadata
                .map(|m| serde_json::to_vec(&m).unwrap_or_default()),
        }
    }
}

impl TryFrom<EdgeWithVector> for vecgraph_core::EdgeWithVector {
    type Error = VecGraphError;

    fn try_from(proto: EdgeWithVector) -> Result<Self, Self::Error> {
        let edge = proto
            .edge
            .ok_or_else(|| VecGraphError::Other("EdgeWithVector missing edge field".into()))?;

        Ok(vecgraph_core::EdgeWithVector {
            edge: edge.into(),
            vector: proto.vectors,
        })
    }
}
