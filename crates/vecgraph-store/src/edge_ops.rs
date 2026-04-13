use crate::VecGraphStore;
use vecgraph_core::{Edge, EdgeId, EdgeWithVector, Node, NodeId, StorageKey, VecGraphError};

pub async fn insert_edge(store: &VecGraphStore, edge: &Edge) -> Result<(), VecGraphError> {
    let edge_key = StorageKey::Edge(edge.id.clone());
    let edge_bytes =
        serde_json::to_vec(&edge).map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
    store
        .kv
        .set(edge_key.partition(), edge_key.key().as_bytes(), &edge_bytes)
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;

    // Write forward index
    let fwd_key = StorageKey::EdgesForNode {
        node_id: edge.source_node_id.clone(),
        edge_id: edge.id.clone(),
    };
    store
        .kv
        .set(
            fwd_key.partition(),
            fwd_key.key().as_bytes(),
            edge.id.as_str().as_bytes(),
        )
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;

    // Write reverse index
    let rev_key = StorageKey::EdgesTargetingNode {
        node_id: edge.target_node_id.clone(),
        edge_id: edge.id.clone(),
    };
    store
        .kv
        .set(
            rev_key.partition(),
            rev_key.key().as_bytes(),
            edge.id.as_str().as_bytes(),
        )
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;

    Ok(())
}

pub async fn insert_edge_with_vector(
    store: &VecGraphStore,
    edge_with_vec: &EdgeWithVector,
) -> Result<(), VecGraphError> {
    // Store edge metadata + both indexes
    insert_edge(store, &edge_with_vec.edge).await?;

    // Store vector separately
    let namespace = get_node_namespace(store, &edge_with_vec.edge.source_node_id).await;
    let vec_key = StorageKey::EdgeVector {
        edge_kind: edge_with_vec.edge.edge_kind.clone(),
        namespace,
        source_node_id: edge_with_vec.edge.source_node_id.clone(),
        target_node_id: edge_with_vec.edge.target_node_id.clone(),
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
    scan_edge_index(store, StorageKey::edges_for_node_prefix(node_id)).await
}

pub async fn get_edges_targeting_node(
    store: &VecGraphStore,
    node_id: &NodeId,
) -> Result<Vec<Edge>, VecGraphError> {
    scan_edge_index(store, StorageKey::edges_targeting_node_prefix(node_id)).await
}

async fn scan_edge_index(
    store: &VecGraphStore,
    (partition, prefix): (&str, String),
) -> Result<Vec<Edge>, VecGraphError> {
    let rx = store.kv.scan(partition, Some(prefix.as_bytes()), 64);

    let mut edges = Vec::new();
    while let Ok(result) = rx.recv().await {
        match result {
            Ok((_key, value)) => {
                let edge_id_str = String::from_utf8_lossy(&value);
                let edge_id = EdgeId::new(edge_id_str.into_owned());
                match get_edge(store, &edge_id).await {
                    Ok(Some(edge)) => edges.push(edge),
                    Ok(None) => {
                        eprintln!("Dangling index entry for edge: {}", edge_id);
                    }
                    Err(e) => {
                        eprintln!("Error fetching edge {}: {}", edge_id, e);
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
    // Get edge metadata to reconstruct all associated keys
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

    // Delete vector (may not exist if edge was inserted without one)
    let namespace = get_node_namespace(store, &edge.source_node_id).await;
    let vec_key = StorageKey::EdgeVector {
        edge_kind: edge.edge_kind,
        namespace,
        source_node_id: edge.source_node_id.clone(),
        target_node_id: edge.target_node_id.clone(),
    };
    let _ = store
        .kv
        .delete(vec_key.partition(), vec_key.key().as_bytes())
        .await;

    // Delete forward index entry
    let fwd_key = StorageKey::EdgesForNode {
        node_id: edge.source_node_id,
        edge_id: id.clone(),
    };
    let _ = store
        .kv
        .delete(fwd_key.partition(), fwd_key.key().as_bytes())
        .await;

    // Delete reverse index entry
    let rev_key = StorageKey::EdgesTargetingNode {
        node_id: edge.target_node_id,
        edge_id: id.clone(),
    };
    let _ = store
        .kv
        .delete(rev_key.partition(), rev_key.key().as_bytes())
        .await;

    Ok(())
}

pub async fn get_edge_vector(
    store: &VecGraphStore,
    id: &EdgeId,
) -> Result<Option<Vec<f32>>, VecGraphError> {
    let edge = match get_edge(store, id).await? {
        Some(e) => e,
        None => return Ok(None),
    };

    let namespace = get_node_namespace(store, &edge.source_node_id).await;
    let vec_key = StorageKey::EdgeVector {
        edge_kind: edge.edge_kind,
        namespace,
        source_node_id: edge.source_node_id,
        target_node_id: edge.target_node_id,
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
