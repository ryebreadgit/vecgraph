mod server;

use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use vecgraph_core::GraphStore;
use vecgraph_proto::graph_store_service_server::GraphStoreServiceServer;

pub use server::GraphStoreServiceImpl;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
}

pub async fn run_server<S>(store: Arc<S>, addr: SocketAddr) -> Result<(), ServerError>
where
    S: GraphStore + Send + Sync + 'static,
{
    let graph_store_service = GraphStoreServiceImpl::new(store);

    tracing::info!(%addr, "starting gRPC server");

    tonic::transport::Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .add_service(GraphStoreServiceServer::new(graph_store_service))
        .serve(addr)
        .await?;

    Ok(())
}
