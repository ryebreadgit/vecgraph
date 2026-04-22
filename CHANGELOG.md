# Changelog


## [0.1.1] - 2026-04-22

### Added

- Add gitt-cliff
- Add cargo-dist and github release workflow
- Add vegraph-proto README.md
- Add client example, direct copy of model2vec example
- Add binary default database path to .gitignore
- Initial server binary implementation
- Add server and client features
- Add vecgraph-client
- Add TryFrom for core to proto traits
- Add VecGraphResult type
- Add README.md
- Add initial vecgraph-server implementation
- Add proto trait impl to and from core
- Add initial protobuf definitions

### Changed

- Update vecgraph readme with new features
- Update version to 0.1.1
- Update README.md to reflect additional feature flags
- Update dependancy syntax

### Chores

- Cargo machete



## [0.1.0] - 2026-04-22

### Added

- Add full documentation link
- Add initial README.md for crates
- Add descriptions for each crate and standardize workspace project imports
- Add README.md and LICENSE.md
- Add basic and model2vec examples
- Add target_node_id to edge to allow unique edges in the same kind and namespace
- Add reverse index for edges, track and remove during deletion
- Switch to insert_edge_with_vector and insert_edge to allow both, add insert_node_with_vector
- Add NodeWithVector, implement SearchKind across searches to select correct partition
- Add target_node_id
- Add embed-model2vec
- Initial store implementation
- Initial commit - core traits pulled from devsetta impl
- Use crates.io kvwrap
- Use thiserror

### Changed

- Reformat print statement
- Rename edge_kind to kind
- Break out embedding options, allowing for additional models to be used. AI assited with Mean pooling strategy and layer_norm_inplace
- Gets partition in rerank
- Pull vector from the correct location

### Removed

- Remove unused enricher
- Remove kvwrap client feature
- Remove unused dependancies
- Remove enrich, copy OnnxEmbedder from devsetta
- Remove unnecessary mut
