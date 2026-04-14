//! Basic example demonstrating node/edge insertion, traversal, and search
//! using the Model2Vec embedder.
//!
//! Run with:
//!   cargo run --example model2vec_basic --features embed-model2vec

use std::sync::Arc;

use kvwrap::{LocalConfig, LocalStore};
use vecgraph::{
    Edge, EdgeWithVector, Embedder, GraphStore, Model2VecEmbedder, Model2VecEmbedderSettings, Node,
    NodeWithVector, SearchQuery, VecGraphStore,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        // Initialise store and embedder

        let store: Arc<dyn GraphStore> = Arc::new(VecGraphStore {
            kv: Box::new(LocalStore::new(LocalConfig {
                path: "./data/example-kvstore".to_string(),
                cache_size: 1024 * 1024 * 10,
            })?),
        });

        let embedder: Arc<dyn Embedder> =
            Arc::new(Model2VecEmbedder::new(Model2VecEmbedderSettings {
                model_path: "minishlab/potion-base-8M".to_string(),
                ..Default::default()
            })?);

        // Create nodes

        let hub_node = Node::new("hub", "category", "All Examples", serde_json::json!({}))
            .with_namespace("examples");

        let node_a = Node::new(
            "node-a",
            "example",
            "Example Node A",
            serde_json::json!({ "data": "first example" }),
        )
        .with_namespace("examples");

        let node_b = Node::new(
            "node-b",
            "example",
            "Example Node B",
            serde_json::json!({ "data": "second example" }),
        )
        .with_namespace("examples");

        // Embed meaningful searchable vectors for nodes based on their content

        let hub_vec = embedder.embed("All examples hub node")?;
        let a_vec = embedder.embed("First example node with sample data")?;
        let b_vec = embedder.embed("Second example node with sample data")?;

        store
            .insert_node_with_vector(&NodeWithVector::new(hub_node, hub_vec))
            .await?;
        store
            .insert_node_with_vector(&NodeWithVector::new(node_a, a_vec))
            .await?;
        store
            .insert_node_with_vector(&NodeWithVector::new(node_b, b_vec))
            .await?;

        // Create edges (hub → a, hub → b)

        let edge_a = Edge::new("hub", "node-a", "contains", "hub links to example A")
            .with_metadata(serde_json::json!({ "weight": 1.0 }));
        let edge_b = Edge::new("hub", "node-b", "contains", "hub links to example B");

        let edge_a_vec = embedder.embed(&edge_a.content)?;
        let edge_b_vec = embedder.embed(&edge_b.content)?;

        store
            .insert_edge_with_vector(&EdgeWithVector::new(edge_a, edge_a_vec))
            .await?;
        store
            .insert_edge_with_vector(&EdgeWithVector::new(edge_b, edge_b_vec))
            .await?;

        // Traverse from hub to its connected nodes

        println!("Graph Traversal\n");

        let hub = store.get_node(&"hub".into()).await?;
        println!("Starting node: {:#?}\n", hub);

        let outgoing = store.get_edges_for_node(&"hub".into()).await?;
        println!("Outgoing edges from hub: {}\n", outgoing.len());

        for edge in &outgoing {
            println!("  edge {:?} → target {:?}", edge.id, edge.target_node_id);
            if let Some(target) = store.get_node(&edge.target_node_id).await? {
                println!("    target node: {} (kind={})", target.name, target.kind);
            }
        }

        // Reverse traversal: find edges targeting node-a, then get their source nodes

        println!("\nReverse Traversal\n");

        let incoming = store.get_edges_targeting_node(&"node-a".into()).await?;
        println!("Edges targeting node-a: {}", incoming.len());
        for edge in &incoming {
            println!("  {:?} from source {:?}", edge.id, edge.source_node_id);
        }

        // Vector search: find nodes by semantic similarity

        println!("\nNode Search\n");

        let query_vec = embedder.embed("example data")?;
        let node_results = store
            .search(
                &SearchQuery::new(query_vec.clone(), "node", "example", 5)
                    .with_namespace("examples"),
            )
            .await?;

        for result in &node_results {
            println!(
                "  node={} kind={} score={:.4} hit_kind={}",
                result.node_id, result.kind, result.score, result.hit_kind
            );
        }

        // Vector search: find edges by semantic similarity

        println!("\nEdge Search\n");

        let edge_results = store
            .search(&SearchQuery::new(query_vec, "edge", "contains", 5))
            .await?;

        for result in &edge_results {
            println!(
                "  node={} kind={} score={:.4} hit_kind={}",
                result.node_id, result.kind, result.score, result.hit_kind
            );
        }

        println!("\nDone.");
        Ok(())
    })
}
