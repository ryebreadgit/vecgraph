mod edge_ops;
mod node_ops;
mod search_ops;
mod store;

use edge_ops::{delete_edge, get_edge, get_edge_vector, get_edges_for_node, insert_edge};
use node_ops::{
    delete_name_mapping, delete_node, get_name_mapping, get_node, insert_node, set_name_mapping,
};
use search_ops::search;

pub use store::VecGraphStore;
