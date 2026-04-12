mod onnx_embedder;
mod settings;

pub use onnx_embedder::OnnxEmbedder;
pub use settings::{
    InputTensorNames, MatryoshkaMode, OnnxEmbedderSettings, PoolingStrategy, PrefixMode,
};
