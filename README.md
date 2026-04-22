# Vecgraph

Vecgraph is a vector graph database for Rust written on top of kvwrap.

## Usage

```bash
cargo add vecgraph --features embed-model2vec
```

```rust
use kvwrap::{LocalConfig, LocalStore};
use vecgraph::{
    Edge, EdgeWithVector, GraphStore, Node, NodeWithVector, SearchQuery, VecGraphStore,
};

// Replace with a real embedder - vecgraph ships Model2VecEmbedder and OnnxEmbedder behind the embed-model2vec and embed-onnx feature flags.
fn embed(text: &str) -> Vec<f32> {
    let mut v = vec![0.0f32; 64];
    for (i, b) in text.bytes().enumerate() {
        v[i % 64] += b as f32;
    }
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 { v.iter_mut().for_each(|x| *x /= norm); }
    v
}

// Create a store
let store = VecGraphStore {
    kv: Box::new(LocalStore::new(LocalConfig {
        path: "./data/mydb".to_string(),
        cache_size: 1024 * 1024 * 10, // 10MB cache
    })?),
};

// Insert nodes with vectors
let alice = Node::new("alice", "person", "Alice", serde_json::json!({ "role": "engineer" }));
let bob = Node::new("bob", "person", "Bob", serde_json::json!({ "role": "designer" }));

store.insert_node_with_vector(&NodeWithVector::new(alice, embed("alice engineer"))).await?;
store.insert_node_with_vector(&NodeWithVector::new(bob, embed("bob designer"))).await?;

// Connect nodes with an edge
let edge = Edge::new("alice", "bob", "collaborates_with", "alice works with bob");
store.insert_edge_with_vector(&EdgeWithVector::new(edge, embed("alice works with bob"))).await?;

// Traverse: outgoing edges from alice
let outgoing = store.get_edges_for_node(&"alice".into()).await?;
for edge in &outgoing {
    let target = store.get_node(&edge.target_node_id).await?;
    println!("{} --[{}]--> {:?}", edge.source_node_id, edge.kind, target);
}

// Reverse traverse: who points to bob?
let incoming = store.get_edges_targeting_node(&"bob".into()).await?;

// Vector search across nodes
let results = store
    .search(&SearchQuery::new(embed("engineer"), "node", "person", 5))
    .await?;
```

## Features

| Flag | Default | Description |
|------|---------|-------------|
| `embed-model2vec` | off | Model2Vec embedder via [model2vec-rs](https://github.com/MinishLab/model2vec-rs) |
| `embed-onnx` | off | ONNX Runtime embedder for custom models |
| `client` | off | Remote client for connecting to a VecGraph server |
| `server` | off | gRPC server for hosting a VecGraph instance accessible by remote clients |
| `full` | off | Enables all features (equivalent to `--features "embed-model2vec,embed-onnx,client,server"`). |

## Examples

[See here](./crates/vecgraph/examples/) for practical examples of using Vecgraph.

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## AI Disclosure

Github Copilot autocomplete was used in the development of this project, but no other Copilot features. Claude was used to generate small snippets of code, specifically portions of the math, which has comments indicating as such.

## License

[MIT](./LICENSE.md)