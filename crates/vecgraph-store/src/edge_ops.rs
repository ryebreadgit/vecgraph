use crate::VecGraphStore;
use vecgraph_core::{Edge, EdgeId, EdgeWithVector, Node, NodeId, StorageKey, VecGraphError};

pub async fn insert_edge(
    store: &VecGraphStore,
    edge_with_vec: &EdgeWithVector,
) -> Result<(), VecGraphError> {
    // Store edge metadata
    let edge_key = StorageKey::Edge(edge_with_vec.edge.id.clone());
    let edge_bytes = serde_json::to_vec(&edge_with_vec.edge)
        .map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
    store
        .kv
        .set(edge_key.partition(), edge_key.key().as_bytes(), &edge_bytes)
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;

    // Store vector separately under `vector:{edge_kind}:{namespace?}:{node_id}`
    let namespace = match get_node_namespace(store, &edge_with_vec.edge.source_node_id).await {
        Some(ns) => Some(ns),
        None => None,
    };
    let vec_key = StorageKey::Vector {
        edge_kind: edge_with_vec.edge.edge_kind.clone(),
        namespace,
        node_id: edge_with_vec.edge.source_node_id.clone(),
    };
    let vec_bytes: &[u8] = bytemuck::cast_slice(&edge_with_vec.vector);
    store
        .kv
        .set(vec_key.partition(), vec_key.key().as_bytes(), vec_bytes)
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;

    Ok(())
}

pub async fn get_edge(store: &VecGraphStore, id: &EdgeId) -> Result<Option<Edge>, VecGraphError> {
    let key = StorageKey::Edge(id.clone());
    match store
        .kv
        .get(key.partition(), key.key().as_bytes())
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?
    {
        Some(bytes) => {
            let edge: Edge = serde_json::from_slice(&bytes)
                .map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
            Ok(Some(edge))
        }
        None => Ok(None),
    }
}

pub async fn get_edges_for_node(
    store: &VecGraphStore,
    node_id: &NodeId,
) -> Result<Vec<Edge>, VecGraphError> {
    // Prefix scan with `edge:{node_id}:` to find all edges for this node
    let prefix = format!("edge:{}:", node_id.as_str());
    let rx = store.kv.scan("edges", Some(prefix.as_bytes()), 64);

    let mut edges = Vec::new();
    while let Ok(result) = rx.recv().await {
        match result {
            Ok((_key, value)) => {
                match serde_json::from_slice::<Edge>(&value) {
                    Ok(edge) => edges.push(edge),
                    Err(e) => {
                        // Log and skip malformed entries rather than failing the whole scan
                        eprintln!("Skipping malformed edge: {}", e);
                    }
                }
            }
            Err(e) => {
                return Err(VecGraphError::StorageError(e.to_string()));
            }
        }
    }

    Ok(edges)
}

pub async fn delete_edge(store: &VecGraphStore, id: &EdgeId) -> Result<(), VecGraphError> {
    // Get edge metadata to reconstruct the vector key
    let edge = match get_edge(store, id).await? {
        Some(e) => e,
        None => return Ok(()), // Already gone
    };

    // Delete edge metadata
    let edge_key = StorageKey::Edge(id.clone());
    store
        .kv
        .delete(edge_key.partition(), edge_key.key().as_bytes())
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;

    // Delete the vector
    let namespace = get_node_namespace(store, &edge.source_node_id).await;
    let vec_key = StorageKey::Vector {
        edge_kind: edge.edge_kind,
        namespace,
        node_id: edge.source_node_id,
    };
    store
        .kv
        .delete(vec_key.partition(), vec_key.key().as_bytes())
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;

    Ok(())
}

pub async fn get_vector(
    store: &VecGraphStore,
    id: &EdgeId,
) -> Result<Option<Vec<f32>>, VecGraphError> {
    // Get edge metadata to reconstruct the vector key
    let edge = match get_edge(store, id).await? {
        Some(e) => e,
        None => return Ok(None),
    };

    let namespace = get_node_namespace(store, &edge.source_node_id).await;
    let vec_key = StorageKey::Vector {
        edge_kind: edge.edge_kind,
        namespace,
        node_id: edge.source_node_id,
    };

    match store
        .kv
        .get(vec_key.partition(), vec_key.key().as_bytes())
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?
    {
        Some(bytes) => {
            let vector: &[f32] = bytemuck::try_cast_slice(&bytes)
                .map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
            Ok(Some(vector.to_vec()))
        }
        None => Ok(None),
    }
}

async fn get_node_namespace(store: &VecGraphStore, node_id: &NodeId) -> Option<String> {
    let key = StorageKey::Node(node_id.clone());
    let bytes = store
        .kv
        .get(key.partition(), key.key().as_bytes())
        .await
        .ok()??;
    let node: Node = serde_json::from_slice(&bytes).ok()?;
    node.namespace
}
