use crate::RemoteGraphStoreConfig;
use async_compat::CompatExt;
use async_trait::async_trait;
use tonic::transport::{Channel, Endpoint};
use vecgraph_core::{
    Edge, EdgeId, EdgeWithVector, GraphStore, Node, NodeId, NodeWithVector, SearchQuery,
    SearchResult, VecGraphError, VecGraphResult,
};
use vecgraph_proto::{
    DeleteEdgeRequest, DeleteNameMappingRequest, DeleteNodeRequest, GetEdgeRequest,
    GetEdgeVectorRequest, GetEdgesForNodeRequest, GetEdgesTargetingNodeRequest,
    GetNameMappingRequest, GetNodeRequest, GetNodeVectorRequest, InsertEdgeRequest,
    InsertEdgeWithVectorRequest, InsertNodeRequest, InsertNodeWithVectorRequest, SearchRequest,
    SetNameMappingRequest, graph_store_service_client::GraphStoreServiceClient,
};

#[derive(Clone)]
pub struct RemoteGraphStore {
    client: GraphStoreServiceClient<Channel>,
}

impl RemoteGraphStore {
    pub async fn connect(config: RemoteGraphStoreConfig) -> VecGraphResult<Self> {
        let mut endpoint: Endpoint = Endpoint::from_shared(config.endpoint)
            .map_err(|e| VecGraphError::Other(format!("invalid endpoint: {}", e)))?;

        if let Some(timeout) = config.connect_timeout {
            endpoint = endpoint.connect_timeout(timeout);
        }

        if let Some(timeout) = config.request_timeout {
            endpoint = endpoint.timeout(timeout);
        }

        let channel = endpoint
            .connect()
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("connection failed: {}", e)))?;

        Ok(Self {
            client: GraphStoreServiceClient::new(channel),
        })
    }

    pub async fn connect_lazy(config: RemoteGraphStoreConfig) -> VecGraphResult<Self> {
        async {
            let mut endpoint: Endpoint = Endpoint::from_shared(config.endpoint)
                .map_err(|e| VecGraphError::Other(format!("invalid endpoint: {}", e)))?;

            if let Some(timeout) = config.connect_timeout {
                endpoint = endpoint.connect_timeout(timeout);
            }

            if let Some(timeout) = config.request_timeout {
                endpoint = endpoint.timeout(timeout);
            }

            let channel = endpoint.connect_lazy();

            Ok(Self {
                client: GraphStoreServiceClient::new(channel),
            })
        }
        .compat()
        .await
    }
}

#[async_trait]
impl GraphStore for RemoteGraphStore {
    async fn insert_node(&self, node: &Node) -> VecGraphResult<()> {
        let request = InsertNodeRequest {
            node: Some(
                node.clone()
                    .try_into()
                    .map_err(|e| VecGraphError::Other(format!("invalid node data: {}", e)))?,
            ),
        };
        self.client
            .clone()
            .insert_node(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;
        Ok(())
    }

    async fn insert_node_with_vector(&self, node: &NodeWithVector) -> VecGraphResult<()> {
        let request = InsertNodeWithVectorRequest {
            node: Some(
                node.clone()
                    .try_into()
                    .map_err(|e| VecGraphError::Other(format!("invalid node data: {}", e)))?,
            ),
        };
        self.client
            .clone()
            .insert_node_with_vector(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;
        Ok(())
    }

    async fn get_node(&self, node: &NodeId) -> VecGraphResult<Option<Node>> {
        let request = GetNodeRequest {
            id: node.to_string(),
        };
        let response = self
            .client
            .clone()
            .get_node(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;

        if let Some(node) = response.into_inner().node {
            let node: Node = node
                .try_into()
                .map_err(|e| VecGraphError::Other(format!("invalid node data: {}", e)))?;
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    async fn get_node_vector(&self, node: &NodeId) -> VecGraphResult<Option<Vec<f32>>> {
        let request = GetNodeVectorRequest {
            id: node.to_string(),
        };
        let response = self
            .client
            .clone()
            .get_node_vector(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;

        if let Some(vector) = response.into_inner().vector {
            Ok(Some(vector.values))
        } else {
            Ok(None)
        }
    }

    async fn delete_node(&self, node: &NodeId) -> VecGraphResult<()> {
        let request = DeleteNodeRequest {
            id: node.to_string(),
        };
        self.client
            .clone()
            .delete_node(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;
        Ok(())
    }

    async fn insert_edge(&self, edge: &Edge) -> VecGraphResult<()> {
        let request = InsertEdgeRequest {
            edge: Some(
                edge.clone()
                    .try_into()
                    .map_err(|e| VecGraphError::Other(format!("invalid edge data: {}", e)))?,
            ),
        };
        self.client
            .clone()
            .insert_edge(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;
        Ok(())
    }

    async fn insert_edge_with_vector(&self, edge: &EdgeWithVector) -> VecGraphResult<()> {
        let request = InsertEdgeWithVectorRequest {
            edge: Some(
                edge.clone()
                    .try_into()
                    .map_err(|e| VecGraphError::Other(format!("invalid edge data: {}", e)))?,
            ),
        };
        self.client
            .clone()
            .insert_edge_with_vector(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;
        Ok(())
    }

    async fn get_edge(&self, edge: &EdgeId) -> VecGraphResult<Option<Edge>> {
        let request = GetEdgeRequest {
            id: edge.to_string(),
        };
        let response = self
            .client
            .clone()
            .get_edge(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;

        if let Some(edge) = response.into_inner().edge {
            let edge: Edge = edge
                .try_into()
                .map_err(|e| VecGraphError::Other(format!("invalid edge data: {}", e)))?;
            Ok(Some(edge))
        } else {
            Ok(None)
        }
    }

    async fn get_edges_for_node(&self, node_id: &NodeId) -> VecGraphResult<Vec<Edge>> {
        let request = GetEdgesForNodeRequest {
            node_id: node_id.to_string(),
        };
        let response = self
            .client
            .clone()
            .get_edges_for_node(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;

        let edges = response
            .into_inner()
            .edges
            .into_iter()
            .filter_map(|edge| {
                edge.try_into()
                    .map_err(|e| {
                        VecGraphError::Other(format!("invalid edge data in response: {}", e))
                    })
                    .ok()
            })
            .collect();

        Ok(edges)
    }

    async fn get_edges_targeting_node(&self, node_id: &NodeId) -> VecGraphResult<Vec<Edge>> {
        let request = GetEdgesTargetingNodeRequest {
            node_id: node_id.to_string(),
        };
        let response = self
            .client
            .clone()
            .get_edges_targeting_node(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;

        let edges = response
            .into_inner()
            .edges
            .into_iter()
            .filter_map(|edge| {
                edge.try_into()
                    .map_err(|e| {
                        VecGraphError::Other(format!("invalid edge data in response: {}", e))
                    })
                    .ok()
            })
            .collect();

        Ok(edges)
    }

    async fn delete_edge(&self, edge: &EdgeId) -> VecGraphResult<()> {
        let request = DeleteEdgeRequest {
            id: edge.to_string(),
        };
        self.client
            .clone()
            .delete_edge(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;
        Ok(())
    }

    async fn get_edge_vector(&self, edge: &EdgeId) -> VecGraphResult<Option<Vec<f32>>> {
        let request = GetEdgeVectorRequest {
            id: edge.to_string(),
        };
        let response = self
            .client
            .clone()
            .get_edge_vector(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;

        if let Some(vector) = response.into_inner().vector {
            Ok(Some(vector.values))
        } else {
            Ok(None)
        }
    }

    async fn set_name_mapping(
        &self,
        kind: &str,
        name: &str,
        node_id: &NodeId,
    ) -> VecGraphResult<()> {
        let request = SetNameMappingRequest {
            kind: kind.to_string(),
            name: name.to_string(),
            node_id: node_id.to_string(),
        };
        self.client
            .clone()
            .set_name_mapping(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;
        Ok(())
    }

    async fn get_name_mapping(&self, kind: &str, name: &str) -> VecGraphResult<Option<NodeId>> {
        let request = GetNameMappingRequest {
            kind: kind.to_string(),
            name: name.to_string(),
        };
        let response = self
            .client
            .clone()
            .get_name_mapping(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;

        if let Some(node_id) = response.into_inner().node_id {
            Ok(Some(NodeId(node_id)))
        } else {
            Ok(None)
        }
    }

    async fn delete_name_mapping(&self, kind: &str, name: &str) -> VecGraphResult<()> {
        let request = DeleteNameMappingRequest {
            kind: kind.to_string(),
            name: name.to_string(),
        };
        self.client
            .clone()
            .delete_name_mapping(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;
        Ok(())
    }

    async fn search(&self, query: &SearchQuery) -> VecGraphResult<Vec<SearchResult>> {
        let request = SearchRequest {
            query: Some(
                query
                    .clone()
                    .try_into()
                    .map_err(|e| VecGraphError::Other(format!("invalid search query: {}", e)))?,
            ),
        };
        let response = self
            .client
            .clone()
            .search(request)
            .compat()
            .await
            .map_err(|e| VecGraphError::Other(format!("request failed: {}", e)))?;

        let results = response
            .into_inner()
            .results
            .into_iter()
            .filter_map(|result| {
                result
                    .try_into()
                    .map_err(|e| {
                        VecGraphError::Other(format!(
                            "invalid search result data in response: {}",
                            e
                        ))
                    })
                    .ok()
            })
            .collect();

        Ok(results)
    }
}
