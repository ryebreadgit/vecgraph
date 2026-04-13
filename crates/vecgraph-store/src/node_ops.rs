use crate::VecGraphStore;
use vecgraph_core::{Node, NodeId, NodeWithVector, StorageKey, VecGraphError};

pub async fn insert_node(store: &VecGraphStore, node: &Node) -> Result<(), VecGraphError> {
    let node_key = StorageKey::Node(node.id.clone());
    let node_bytes =
        serde_json::to_vec(&node).map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
    store
        .kv
        .set(node_key.partition(), node_key.key().as_bytes(), &node_bytes)
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;
    Ok(())
}

pub async fn insert_node_with_vector(
    store: &VecGraphStore,
    node_with_vector: &NodeWithVector,
) -> Result<(), VecGraphError> {
    insert_node(store, &node_with_vector.node).await?;
    // Store vector separately under `vector:{kind}:{namespace?}:{node_id}`
    let vec_key = StorageKey::NodeVector {
        kind: node_with_vector.node.kind.clone(),
        namespace: node_with_vector.node.namespace.clone(),
        node_id: node_with_vector.node.id.clone(),
    };
    let vec_bytes: &[u8] = bytemuck::cast_slice(&node_with_vector.vector);
    store
        .kv
        .set(vec_key.partition(), vec_key.key().as_bytes(), vec_bytes)
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;
    Ok(())
}

pub async fn get_node(store: &VecGraphStore, id: &NodeId) -> Result<Option<Node>, VecGraphError> {
    let key = StorageKey::Node(id.clone());
    if let Some(value) = store
        .kv
        .get(key.partition(), key.key().as_bytes())
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?
    {
        let node = serde_json::from_slice(&value)
            .map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
        Ok(Some(node))
    } else {
        Ok(None)
    }
}
pub async fn delete_node(store: &VecGraphStore, id: &NodeId) -> Result<(), VecGraphError> {
    let key = StorageKey::Node(id.clone());
    store
        .kv
        .delete(key.partition(), key.key().as_bytes())
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;
    Ok(())
}

pub async fn set_name_mapping(
    store: &VecGraphStore,
    kind: &str,
    name: &str,
    node_id: &NodeId,
) -> Result<(), VecGraphError> {
    let key = StorageKey::NameMapping {
        kind: kind.to_string(),
        name: name.to_string(),
    };
    let value = serde_json::to_vec(node_id)
        .map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
    store
        .kv
        .set(key.partition(), key.key().as_bytes(), &value)
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;
    Ok(())
}

pub async fn get_name_mapping(
    store: &VecGraphStore,
    kind: &str,
    name: &str,
) -> Result<Option<NodeId>, VecGraphError> {
    let key = StorageKey::NameMapping {
        kind: kind.to_string(),
        name: name.to_string(),
    };
    if let Some(value) = store
        .kv
        .get(key.partition(), key.key().as_bytes())
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?
    {
        let node_id = serde_json::from_slice(&value)
            .map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
        Ok(Some(node_id))
    } else {
        Ok(None)
    }
}

pub async fn delete_name_mapping(
    store: &VecGraphStore,
    kind: &str,
    name: &str,
) -> Result<(), VecGraphError> {
    let key = StorageKey::NameMapping {
        kind: kind.to_string(),
        name: name.to_string(),
    };
    store
        .kv
        .delete(key.partition(), key.key().as_bytes())
        .await
        .map_err(|e| VecGraphError::StorageError(e.to_string()))?;
    Ok(())
}

pub async fn get_node_vector(
    store: &VecGraphStore,
    id: &NodeId,
) -> Result<Option<Vec<f32>>, VecGraphError> {
    // We need the node's kind and namespace to reconstruct the vector key
    let node = match get_node(store, id).await? {
        Some(n) => n,
        None => return Ok(None),
    };

    let vec_key = StorageKey::NodeVector {
        kind: node.kind,
        namespace: node.namespace,
        node_id: id.clone(),
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
