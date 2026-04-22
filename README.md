# Vecgraph

Vecgraph is a vector graph database for Rust written on top of kvwrap.

## Usage

```bash
cargo add vecgraph --features embed-model2vec
```

```rust
use kvwrap::{LocalConfig, LocalStore};
use vecgraph::{
    Edge, EdgeWithVector, Node, NodeWithVector, SearchQuery,
    GraphStore, Embedder, VecGraphStore, Model2VecEmbedder, Model2VecEmbedderSettings,
};

// Create a store
let store = VecGraphStore {
    kv: Box::new(LocalStore::new(LocalConfig {
        path: "./data/mydb".to_string(),
        cache_size: 1024 * 1024 * 10, // 10MB cache
    })?),
};

// Intialize embedder (using model2vec in this example)
let embedder: Arc<dyn Embedder> =
    Arc::new(Model2VecEmbedder::new(Model2VecEmbedderSettings {
        model_path: "minishlab/potion-base-8M".to_string(),
        ..Default::default()
    })?);

// Insert nodes with vectors
let alice = Node::new("alice", "person", "Alice", serde_json::json!({ "role": "engineer" }));
let bob = Node::new("bob", "person", "Bob", serde_json::json!({ "role": "designer" }));

store.insert_node_with_vector(&NodeWithVector::new(alice, embedder.embed("alice engineer")?)).await?;
store.insert_node_with_vector(&NodeWithVector::new(bob, embedder.embed("bob designer")?)).await?;

// Connect nodes with an edge
let edge = Edge::new("alice", "bob", "collaborates_with", "alice works with bob");
store.insert_edge_with_vector(&EdgeWithVector::new(edge, embedder.embed("alice works with bob")?)).await?;

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
    .search(&SearchQuery::new(embedder.embed("engineer")?, "node", "person", 5))
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

## Server Binary

The `vecgraph` binary provides a gRPC server for remote access through the `client` feature. Prebuilt binaries are available on the [releases page](https://github.com/ryebreadgit/vecgraph/releases). To run the server, use:

```bash
vecgraph --listen 0.0.0.0:50051 --database ./my_data
```

All options can also be set via environment variables:

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--listen` | `VECGRAPH_LISTEN_ADDR` | `0.0.0.0:50051` | Address and port to bind |
| `--database` | `VECGRAPH_DATA_PATH` | `.vecgraph_data/` | Storage directory (or remote URI) |
| `--database-cache-size` | `VECGRAPH_DATABASE_CACHE_SIZE` | `67108864` (64 MiB) | In-memory cache size in bytes |
| `--is-db-remote` | `VECGRAPH_IS_DATABASE_REMOTE` | `false` | Treat `--database` as a remote address |
| `--verbose` | `VECGRAPH_VERBOSE` | `false` | Enable verbose logging |

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