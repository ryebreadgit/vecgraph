use crate::{
    delete_edge, delete_name_mapping, delete_node, get_edge, get_edge_vector, get_edges_for_node,
    get_name_mapping, get_node, get_node_vector, insert_edge, insert_edge_with_vector, insert_node,
    insert_node_with_vector, search, set_name_mapping,
};
use async_trait::async_trait;
use kvwrap::KvStore;
use vecgraph_core::{
    Edge, EdgeId, EdgeWithVector, GraphStore, Node, NodeId, NodeWithVector, SearchQuery,
    SearchResult, VecGraphError,
};

pub struct VecGraphStore {
    pub kv: Box<dyn KvStore>,
}

#[async_trait]
impl GraphStore for VecGraphStore {
    async fn insert_node(&self, node: &Node) -> Result<(), VecGraphError> {
        insert_node(self, node).await
    }
    async fn insert_node_with_vector(&self, node: &NodeWithVector) -> Result<(), VecGraphError> {
        insert_node_with_vector(self, node).await
    }

    async fn get_node(&self, id: &NodeId) -> Result<Option<Node>, VecGraphError> {
        get_node(self, id).await
    }

    async fn get_node_vector(&self, id: &NodeId) -> Result<Option<Vec<f32>>, VecGraphError> {
        get_node_vector(self, id).await
    }

    async fn delete_node(&self, id: &NodeId) -> Result<(), VecGraphError> {
        // Cascade: delete all edges first, then the node
        let edges = get_edges_for_node(self, id).await?;
        for edge in &edges {
            delete_edge(self, &edge.id).await?;
        }
        delete_node(self, id).await
    }
    async fn insert_edge(&self, edge: &Edge) -> Result<(), VecGraphError> {
        insert_edge(self, edge).await
    }
    async fn insert_edge_with_vector(&self, edge: &EdgeWithVector) -> Result<(), VecGraphError> {
        insert_edge_with_vector(self, edge).await
    }
    async fn get_edge(&self, id: &EdgeId) -> Result<Option<Edge>, VecGraphError> {
        get_edge(self, id).await
    }
    async fn get_edges_for_node(&self, node_id: &NodeId) -> Result<Vec<Edge>, VecGraphError> {
        get_edges_for_node(self, node_id).await
    }
    async fn delete_edge(&self, id: &EdgeId) -> Result<(), VecGraphError> {
        delete_edge(self, id).await
    }
    async fn get_edge_vector(&self, id: &EdgeId) -> Result<Option<Vec<f32>>, VecGraphError> {
        get_edge_vector(self, id).await
    }
    async fn set_name_mapping(
        &self,
        kind: &str,
        name: &str,
        node_id: &NodeId,
    ) -> Result<(), VecGraphError> {
        set_name_mapping(self, kind, name, node_id).await
    }
    async fn get_name_mapping(
        &self,
        kind: &str,
        name: &str,
    ) -> Result<Option<NodeId>, VecGraphError> {
        get_name_mapping(self, kind, name).await
    }
    async fn delete_name_mapping(&self, kind: &str, name: &str) -> Result<(), VecGraphError> {
        delete_name_mapping(self, kind, name).await
    }
    async fn search(&self, request: &SearchQuery) -> Result<Vec<SearchResult>, VecGraphError> {
        search(self, request).await
    }
}
