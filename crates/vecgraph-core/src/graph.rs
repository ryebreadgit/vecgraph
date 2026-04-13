use crate::{
    Edge, EdgeId, EdgeWithVector, Node, NodeId, NodeWithVector, SearchQuery, SearchResult,
    error::VecGraphError,
};
use async_trait::async_trait;

#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn insert_node(&self, node: &Node) -> Result<(), VecGraphError>;

    async fn insert_node_with_vector(&self, node: &NodeWithVector) -> Result<(), VecGraphError>;

    async fn insert_nodes_with_vector(
        &self,
        nodes: &[NodeWithVector],
    ) -> Result<(), VecGraphError> {
        for node in nodes {
            self.insert_node_with_vector(node).await?;
        }
        Ok(())
    }

    async fn get_node(&self, id: &NodeId) -> Result<Option<Node>, VecGraphError>;

    async fn delete_node(&self, id: &NodeId) -> Result<(), VecGraphError>;

    async fn insert_edge(&self, edge: &Edge) -> Result<(), VecGraphError>;

    async fn insert_edge_with_vector(&self, edge: &EdgeWithVector) -> Result<(), VecGraphError>;

    async fn insert_edges_with_vector(
        &self,
        edges: &[EdgeWithVector],
    ) -> Result<(), VecGraphError> {
        for edge in edges {
            self.insert_edge_with_vector(edge).await?;
        }
        Ok(())
    }

    async fn get_edge(&self, id: &EdgeId) -> Result<Option<Edge>, VecGraphError>;

    async fn get_edges_for_node(&self, node_id: &NodeId) -> Result<Vec<Edge>, VecGraphError>;

    async fn get_edges_targeting_node(&self, node_id: &NodeId) -> Result<Vec<Edge>, VecGraphError>;

    async fn delete_edge(&self, id: &EdgeId) -> Result<(), VecGraphError>;

    async fn get_edge_vector(&self, id: &EdgeId) -> Result<Option<Vec<f32>>, VecGraphError>;

    async fn get_node_vector(&self, id: &NodeId) -> Result<Option<Vec<f32>>, VecGraphError>;

    async fn set_name_mapping(
        &self,
        kind: &str,
        name: &str,
        node_id: &NodeId,
    ) -> Result<(), VecGraphError>;

    async fn get_name_mapping(
        &self,
        kind: &str,
        name: &str,
    ) -> Result<Option<NodeId>, VecGraphError>;

    async fn delete_name_mapping(&self, kind: &str, name: &str) -> Result<(), VecGraphError>;

    async fn search(&self, request: &SearchQuery) -> Result<Vec<SearchResult>, VecGraphError>;
}
