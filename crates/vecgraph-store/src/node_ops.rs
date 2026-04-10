use crate::VecGraphStore;
use vecgraph_core::{Node, NodeId, StorageKey, VecGraphError};

pub async fn insert_node(store: &VecGraphStore, node: &Node) -> Result<(), VecGraphError> {
    let key = StorageKey::Node(node.id.clone());
    let value =
        serde_json::to_vec(node).map_err(|e| VecGraphError::SerializationError(e.to_string()))?;
    store
        .kv
        .set(key.partition(), key.key().as_bytes(), &value)
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
