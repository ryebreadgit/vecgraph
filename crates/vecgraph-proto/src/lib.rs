// vecgraph-proto/src/lib.rs

mod traits;

pub mod proto {
    tonic::include_proto!("graphstorepb");
}

pub use proto::*;
pub use traits::*;
