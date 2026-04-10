// crates/vecgraph-core/src/graph.rs

use crate::{
    Edge, EdgeId, EdgeWithVector, Node, NodeId, SearchQuery, SearchResult, error::VecGraphError,
};
use async_trait::async_trait;

#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn insert_node(&self, node: &Node) -> Result<(), VecGraphError>;

    async fn get_node(&self, id: &NodeId) -> Result<Option<Node>, VecGraphError>;

    async fn delete_node(&self, id: &NodeId) -> Result<(), VecGraphError>;

    async fn insert_edge(&self, edge: &EdgeWithVector) -> Result<(), VecGraphError>;

    async fn insert_edges(&self, edges: &[EdgeWithVector]) -> Result<(), VecGraphError> {
        for edge in edges {
            self.insert_edge(edge).await?;
        }
        Ok(())
    }

    async fn get_edge(&self, id: &EdgeId) -> Result<Option<Edge>, VecGraphError>;

    async fn get_edges_for_node(&self, node_id: &NodeId) -> Result<Vec<Edge>, VecGraphError>;

    async fn delete_edge(&self, id: &EdgeId) -> Result<(), VecGraphError>;

    async fn get_vector(&self, id: &EdgeId) -> Result<Option<Vec<f32>>, VecGraphError>;

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
