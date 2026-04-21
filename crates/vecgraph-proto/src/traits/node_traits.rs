use crate::{Node, NodeWithVector};
use vecgraph_core::VecGraphError;

impl TryFrom<Node> for vecgraph_core::Node {
    type Error = VecGraphError;

    fn try_from(proto: Node) -> Result<Self, Self::Error> {
        let payload = if proto.payload.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&proto.payload).map_err(|e| {
                VecGraphError::SerializationError(format!("invalid node payload: {}", e))
            })?
        };

        Ok(vecgraph_core::Node {
            id: vecgraph_core::NodeId::try_new(proto.id)?,
            kind: proto.kind,
            name: proto.name,
            namespace: proto.namespace,
            payload,
        })
    }
}

impl TryFrom<vecgraph_core::Node> for Node {
    type Error = VecGraphError;

    fn try_from(core: vecgraph_core::Node) -> Result<Self, Self::Error> {
        let payload = serde_json::to_vec(&core.payload).map_err(|e| {
            VecGraphError::SerializationError(format!("failed to serialize payload: {}", e))
        })?;

        Ok(Node {
            id: core.id.0,
            kind: core.kind,
            name: core.name,
            namespace: core.namespace,
            payload,
        })
    }
}

impl TryFrom<NodeWithVector> for vecgraph_core::NodeWithVector {
    type Error = VecGraphError;

    fn try_from(proto: NodeWithVector) -> Result<Self, Self::Error> {
        let node = proto
            .node
            .ok_or_else(|| VecGraphError::Other("NodeWithVector missing node field".into()))?;

        Ok(vecgraph_core::NodeWithVector {
            node: node.try_into()?,
            vector: proto.vectors,
        })
    }
}

impl TryFrom<vecgraph_core::NodeWithVector> for NodeWithVector {
    type Error = VecGraphError;

    fn try_from(core: vecgraph_core::NodeWithVector) -> Result<Self, Self::Error> {
        Ok(NodeWithVector {
            node: Some(core.node.try_into()?),
            vectors: core.vector,
        })
    }
}
