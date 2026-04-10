mod edge_ops;
mod node_ops;
mod search_ops;
mod store;

pub use edge_ops::{delete_edge, get_edge, get_edges_for_node, get_vector, insert_edge};
pub use node_ops::{
    delete_name_mapping, delete_node, get_name_mapping, get_node, insert_node, set_name_mapping,
};
pub use search_ops::search;
pub use store::VecGraphStore;
