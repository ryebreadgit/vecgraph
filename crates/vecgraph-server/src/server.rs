use std::sync::Arc;
use tonic::{Request, Response, Status};
use vecgraph_core::GraphStore;
use vecgraph_proto::{
    DeleteEdgeRequest, DeleteEdgeResponse, DeleteNameMappingRequest, DeleteNameMappingResponse,
    DeleteNodeRequest, DeleteNodeResponse, GetEdgeRequest, GetEdgeResponse, GetEdgeVectorRequest,
    GetEdgeVectorResponse, GetEdgesForNodeRequest, GetEdgesForNodeResponse,
    GetEdgesTargetingNodeRequest, GetEdgesTargetingNodeResponse, GetNameMappingRequest,
    GetNameMappingResponse, GetNodeRequest, GetNodeResponse, GetNodeVectorRequest,
    GetNodeVectorResponse, InsertEdgeRequest, InsertEdgeResponse, InsertEdgeWithVectorRequest,
    InsertEdgeWithVectorResponse, InsertEdgesWithVectorRequest, InsertEdgesWithVectorResponse,
    InsertNodeRequest, InsertNodeResponse, InsertNodeWithVectorRequest,
    InsertNodeWithVectorResponse, InsertNodesWithVectorRequest, InsertNodesWithVectorResponse,
    SearchRequest, SearchResponse, SetNameMappingRequest, SetNameMappingResponse, VectorValue,
    graph_store_service_server::GraphStoreService,
};

pub struct GraphStoreServiceImpl<S> {
    store: Arc<S>,
}

impl<S> GraphStoreServiceImpl<S> {
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl<S> GraphStoreService for GraphStoreServiceImpl<S>
where
    S: GraphStore + Send + Sync + 'static,
{
    async fn insert_edge(
        &self,
        request: Request<InsertEdgeRequest>,
    ) -> Result<Response<InsertEdgeResponse>, Status> {
        let req = request.into_inner();
        let core_edge: Option<vecgraph_core::Edge> = req.edge.map(|e| e.into());
        if let Some(edge) = &core_edge {
            self.store
                .insert_edge(edge)
                .await
                .map_err(|e| Status::internal(format!("Failed to insert edge: {}", e)))?;
            Ok(Response::new(InsertEdgeResponse {}))
        } else {
            Err(Status::invalid_argument("Edge data is missing"))
        }
    }

    async fn get_edge(
        &self,
        request: Request<GetEdgeRequest>,
    ) -> Result<Response<GetEdgeResponse>, Status> {
        let req = request.into_inner();
        match self.store.get_edge(&req.id.into()).await {
            Ok(Some(edge)) => Ok(Response::new(GetEdgeResponse {
                edge: Some(edge.into()),
            })),
            Ok(None) => Err(Status::not_found("Edge not found")),
            Err(e) => Err(Status::internal(format!("Failed to get edge: {}", e))),
        }
    }

    async fn delete_edge(
        &self,
        request: Request<DeleteEdgeRequest>,
    ) -> Result<Response<DeleteEdgeResponse>, Status> {
        let req = request.into_inner();
        self.store
            .delete_edge(&req.id.into())
            .await
            .map_err(|e| Status::internal(format!("Failed to delete edge: {}", e)))?;
        Ok(Response::new(DeleteEdgeResponse {}))
    }

    async fn insert_edge_with_vector(
        &self,
        request: Request<InsertEdgeWithVectorRequest>,
    ) -> Result<Response<InsertEdgeWithVectorResponse>, Status> {
        let req = request.into_inner();
        let core_edge_with_vector: Option<vecgraph_core::EdgeWithVector> =
            req.edge.map(|e| e.try_into()).transpose().map_err(|e| {
                Status::invalid_argument(format!("Invalid EdgeWithVector data: {}", e))
            })?;
        if let Some(edge_with_vector) = &core_edge_with_vector {
            self.store
                .insert_edge_with_vector(edge_with_vector)
                .await
                .map_err(|e| {
                    Status::internal(format!("Failed to insert edge with vector: {}", e))
                })?;
            Ok(Response::new(InsertEdgeWithVectorResponse {}))
        } else {
            Err(Status::invalid_argument("EdgeWithVector data is missing"))
        }
    }

    async fn insert_node_with_vector(
        &self,
        request: Request<InsertNodeWithVectorRequest>,
    ) -> Result<Response<InsertNodeWithVectorResponse>, Status> {
        let req = request.into_inner();
        let core_node_with_vector: Option<vecgraph_core::NodeWithVector> =
            req.node.map(|n| n.try_into()).transpose().map_err(|e| {
                Status::invalid_argument(format!("Invalid NodeWithVector data: {}", e))
            })?;
        if let Some(node_with_vector) = &core_node_with_vector {
            self.store
                .insert_node_with_vector(node_with_vector)
                .await
                .map_err(|e| {
                    Status::internal(format!("Failed to insert node with vector: {}", e))
                })?;
            Ok(Response::new(InsertNodeWithVectorResponse {}))
        } else {
            Err(Status::invalid_argument("NodeWithVector data is missing"))
        }
    }

    async fn insert_node(
        &self,
        request: Request<InsertNodeRequest>,
    ) -> Result<Response<InsertNodeResponse>, Status> {
        let req = request.into_inner();
        let core_node: Option<vecgraph_core::Node> = req
            .node
            .map(|n| n.try_into())
            .transpose()
            .map_err(|e| Status::invalid_argument(format!("Invalid Node data: {}", e)))?;
        if let Some(node) = &core_node {
            self.store
                .insert_node(node)
                .await
                .map_err(|e| Status::internal(format!("Failed to insert node: {}", e)))?;
            Ok(Response::new(InsertNodeResponse {}))
        } else {
            Err(Status::invalid_argument("Node data is missing"))
        }
    }

    async fn get_node(
        &self,
        request: Request<GetNodeRequest>,
    ) -> Result<Response<GetNodeResponse>, Status> {
        let req = request.into_inner();
        match self.store.get_node(&req.id.into()).await {
            Ok(Some(node)) => Ok(Response::new(GetNodeResponse {
                node: Some(node.try_into().map_err(|e| {
                    Status::internal(format!("Failed to convert node data: {}", e))
                })?),
            })),
            Ok(None) => Err(Status::not_found("Node not found")),
            Err(e) => Err(Status::internal(format!("Failed to get node: {}", e))),
        }
    }

    async fn delete_node(
        &self,
        request: Request<DeleteNodeRequest>,
    ) -> Result<Response<DeleteNodeResponse>, Status> {
        let req = request.into_inner();
        self.store
            .delete_node(&req.id.into())
            .await
            .map_err(|e| Status::internal(format!("Failed to delete node: {}", e)))?;
        Ok(Response::new(DeleteNodeResponse {}))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        let core_query: vecgraph_core::SearchQuery = req
            .query
            .ok_or_else(|| Status::invalid_argument("Search query is missing"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("Invalid search query data: {}", e)))?;
        match self.store.search(&core_query).await {
            Ok(results) => Ok(Response::new(SearchResponse {
                results: results.into_iter().map(|r| r.into()).collect(),
            })),
            Err(e) => Err(Status::internal(format!("Failed to perform search: {}", e))),
        }
    }

    async fn insert_edges_with_vector(
        &self,
        request: Request<InsertEdgesWithVectorRequest>,
    ) -> Result<Response<InsertEdgesWithVectorResponse>, Status> {
        let req = request.into_inner();
        let core_edges_with_vector: Vec<vecgraph_core::EdgeWithVector> = req
            .edges
            .into_iter()
            .map(|e| e.try_into())
            .collect::<Result<_, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid EdgeWithVector data: {}", e)))?;
        self.store
            .insert_edges_with_vector(&core_edges_with_vector)
            .await
            .map_err(|e| Status::internal(format!("Failed to insert edges with vector: {}", e)))?;
        Ok(Response::new(InsertEdgesWithVectorResponse {}))
    }

    async fn insert_nodes_with_vector(
        &self,
        request: Request<InsertNodesWithVectorRequest>,
    ) -> Result<Response<InsertNodesWithVectorResponse>, Status> {
        let req = request.into_inner();
        let core_nodes_with_vector: Vec<vecgraph_core::NodeWithVector> = req
            .nodes
            .into_iter()
            .map(|n| n.try_into())
            .collect::<Result<_, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid NodeWithVector data: {}", e)))?;
        self.store
            .insert_nodes_with_vector(&core_nodes_with_vector)
            .await
            .map_err(|e| Status::internal(format!("Failed to insert nodes with vector: {}", e)))?;
        Ok(Response::new(InsertNodesWithVectorResponse {}))
    }

    async fn get_edges_for_node(
        &self,
        request: Request<GetEdgesForNodeRequest>,
    ) -> Result<Response<GetEdgesForNodeResponse>, Status> {
        let req = request.into_inner();
        match self.store.get_edges_for_node(&req.node_id.into()).await {
            Ok(edges) => Ok(Response::new(GetEdgesForNodeResponse {
                edges: edges.into_iter().map(|e| e.into()).collect(),
            })),
            Err(e) => Err(Status::internal(format!(
                "Failed to get edges for node: {}",
                e
            ))),
        }
    }

    async fn get_edges_targeting_node(
        &self,
        request: Request<GetEdgesTargetingNodeRequest>,
    ) -> Result<Response<GetEdgesTargetingNodeResponse>, Status> {
        let req = request.into_inner();
        match self
            .store
            .get_edges_targeting_node(&req.node_id.into())
            .await
        {
            Ok(edges) => Ok(Response::new(GetEdgesTargetingNodeResponse {
                edges: edges.into_iter().map(|e| e.into()).collect(),
            })),
            Err(e) => Err(Status::internal(format!(
                "Failed to get edges targeting node: {}",
                e
            ))),
        }
    }

    async fn get_edge_vector(
        &self,
        request: Request<GetEdgeVectorRequest>,
    ) -> Result<Response<GetEdgeVectorResponse>, Status> {
        let req = request.into_inner();
        match self.store.get_edge_vector(&req.id.into()).await {
            Ok(Some(vector)) => Ok(Response::new(GetEdgeVectorResponse {
                vector: Some(VectorValue { values: vector }),
            })),
            Ok(None) => Err(Status::not_found("Edge vector not found")),
            Err(e) => Err(Status::internal(format!(
                "Failed to get edge vector: {}",
                e
            ))),
        }
    }

    async fn get_node_vector(
        &self,
        request: Request<GetNodeVectorRequest>,
    ) -> Result<Response<GetNodeVectorResponse>, Status> {
        let req = request.into_inner();
        match self.store.get_node_vector(&req.id.into()).await {
            Ok(Some(vector)) => Ok(Response::new(GetNodeVectorResponse {
                vector: Some(VectorValue { values: vector }),
            })),
            Ok(None) => Err(Status::not_found("Node vector not found")),
            Err(e) => Err(Status::internal(format!(
                "Failed to get node vector: {}",
                e
            ))),
        }
    }

    async fn set_name_mapping(
        &self,
        request: Request<SetNameMappingRequest>,
    ) -> Result<Response<SetNameMappingResponse>, Status> {
        let req = request.into_inner();
        self.store
            .set_name_mapping(&req.kind, &req.name, &req.node_id.into())
            .await
            .map_err(|e| Status::internal(format!("Failed to set name mapping: {}", e)))?;
        Ok(Response::new(SetNameMappingResponse {}))
    }

    async fn get_name_mapping(
        &self,
        request: Request<GetNameMappingRequest>,
    ) -> Result<Response<GetNameMappingResponse>, Status> {
        let req = request.into_inner();
        match self.store.get_name_mapping(&req.kind, &req.name).await {
            Ok(Some(node_id)) => Ok(Response::new(GetNameMappingResponse {
                node_id: Some(node_id.0),
            })),
            Ok(None) => Err(Status::not_found("Name mapping not found")),
            Err(e) => Err(Status::internal(format!(
                "Failed to get name mapping: {}",
                e
            ))),
        }
    }

    async fn delete_name_mapping(
        &self,
        request: Request<DeleteNameMappingRequest>,
    ) -> Result<Response<DeleteNameMappingResponse>, Status> {
        let req = request.into_inner();
        self.store
            .delete_name_mapping(&req.kind, &req.name)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete name mapping: {}", e)))?;
        Ok(Response::new(DeleteNameMappingResponse {}))
    }
}
