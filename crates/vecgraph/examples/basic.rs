//! Basic example demonstrating node/edge insertion, traversal, and vector
//! search using manually constructed vectors (no embedder required).
//!
//! Run with:
//!   cargo run --example basic

use kvwrap::{LocalConfig, LocalStore};
use vecgraph::{
    Edge, EdgeWithVector, GraphStore, Node, NodeWithVector, SearchQuery, VecGraphStore,
};

/// Generate a simple deterministic vector from a string. Not a real embedder, just for testing.
fn fake_embed(text: &str, dims: usize) -> Vec<f32> {
    let mut vec = vec![0.0f32; dims];
    for (i, byte) in text.bytes().enumerate() {
        vec[i % dims] += byte as f32;
    }
    // Normalise to unit length
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
    vec
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        // Initialise store

        let store = VecGraphStore {
            kv: Box::new(LocalStore::new(LocalConfig {
                path: "./data/basic-example".to_string(),
                cache_size: 1024 * 1024 * 10,
            })?),
        };

        const DIMS: usize = 64;

        // Create and insert nodes

        let hub = Node::new("hub", "category", "Project Hub", serde_json::json!({}))
            .with_namespace("demo");

        let alice = Node::new(
            "alice",
            "person",
            "Alice",
            serde_json::json!({ "role": "engineer" }),
        )
        .with_namespace("demo");

        let bob = Node::new(
            "bob",
            "person",
            "Bob",
            serde_json::json!({ "role": "designer" }),
        )
        .with_namespace("demo");

        store
            .insert_node_with_vector(&NodeWithVector::new(hub, fake_embed("project hub", DIMS)))
            .await?;
        store
            .insert_node_with_vector(&NodeWithVector::new(
                alice,
                fake_embed("alice engineer", DIMS),
            ))
            .await?;
        store
            .insert_node_with_vector(&NodeWithVector::new(bob, fake_embed("bob designer", DIMS)))
            .await?;

        // Create and insert edges

        let edge_alice = Edge::new("hub", "alice", "member", "alice is a project member");
        let edge_bob = Edge::new("hub", "bob", "member", "bob is a project member");
        let edge_collab = Edge::new("alice", "bob", "collaborates_with", "alice works with bob");

        store
            .insert_edge_with_vector(&EdgeWithVector::new(
                edge_alice,
                fake_embed("alice is a project member", DIMS),
            ))
            .await?;
        store
            .insert_edge_with_vector(&EdgeWithVector::new(
                edge_bob,
                fake_embed("bob is a project member", DIMS),
            ))
            .await?;
        store
            .insert_edge_with_vector(&EdgeWithVector::new(
                edge_collab,
                fake_embed("alice works with bob", DIMS),
            ))
            .await?;

        // Traverse from hub to its connected nodes

        println!("Forward Traversal\n");

        let outgoing = store.get_edges_for_node(&"hub".into()).await?;
        for edge in &outgoing {
            let target = store.get_node(&edge.target_node_id).await?;
            println!(
                "  hub --[{}]--> {} ({})",
                edge.kind,
                edge.target_node_id,
                target.map(|n| n.name).unwrap_or_else(|| "???".into())
            );
        }

        // Reverse traversal: who points to bob?

        println!("\nReverse Traversal\n");

        let incoming = store.get_edges_targeting_node(&"bob".into()).await?;
        for edge in &incoming {
            println!("  {} --[{}]--> bob", edge.source_node_id, edge.kind);
        }

        // Multi-hop traversal: hub → ? → ?

        println!("\nMulti-hop Traversal\n");

        let hub_edges = store.get_edges_for_node(&"hub".into()).await?;
        for hop1 in &hub_edges {
            let hop1_edges = store.get_edges_for_node(&hop1.target_node_id).await?;
            for hop2 in &hop1_edges {
                println!(
                    "  hub → {} --[{}]--> {}",
                    hop1.target_node_id, hop2.kind, hop2.target_node_id
                );
            }
        }

        // Node search

        println!("\nNode Search (query: \"engineer\")\n");

        let results = store
            .search(
                &SearchQuery::new(fake_embed("engineer", DIMS), "node", "person", 5)
                    .with_namespace("demo"),
            )
            .await?;

        for r in &results {
            println!(
                "  {} (score={:.4}, hit_kind={})",
                r.node_id, r.score, r.hit_kind
            );
        }

        // Edge search

        println!("\nEdge Search (query: \"project member\")\n");

        let results = store
            .search(&SearchQuery::new(
                fake_embed("project member", DIMS),
                "edge",
                "member",
                5,
            ))
            .await?;

        for r in &results {
            println!(
                "  node={} (score={:.4}, hit_kind={})",
                r.node_id, r.score, r.hit_kind
            );
        }

        // Cleanup: delete bob and verify cascade

        println!("\nDelete bob (cascades edges)\n");

        store.delete_node(&"bob".into()).await?;

        let hub_edges_after = store.get_edges_for_node(&"hub".into()).await?;
        println!(
            "  Hub outgoing edges after delete: {}",
            hub_edges_after.len()
        );

        let alice_edges_after = store.get_edges_for_node(&"alice".into()).await?;
        println!(
            "  Alice outgoing edges after delete: {}",
            alice_edges_after.len()
        );

        println!("\nDone.");
        Ok(())
    })
}
