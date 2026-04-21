use crate::{RerankParams, SearchKind, SearchQuery, SearchResult};
use vecgraph_core::VecGraphError;

impl TryFrom<SearchQuery> for vecgraph_core::SearchQuery {
    type Error = VecGraphError;

    fn try_from(proto: SearchQuery) -> Result<Self, Self::Error> {
        let proto_search_kind = SearchKind::try_from(proto.search_kind).map_err(|_| {
            VecGraphError::Other(format!("invalid search_kind value: {}", proto.search_kind))
        })?;

        let search_kind: vecgraph_core::SearchKind = proto_search_kind.try_into()?;

        let rerank = proto.rerank.map(|rp| vecgraph_core::RerankParams {
            vector: rp.vectors,
            kind: rp.kind,
            weight: rp.weight,
        });

        Ok(vecgraph_core::SearchQuery {
            query_vec: proto.query_vecs,
            top_k: proto.top_k as usize,
            search_kind,
            rerank,
            exclude_names: proto.exclude_names,
            kind: proto.kind,
            namespace: proto.namespace,
        })
    }
}

impl TryFrom<vecgraph_core::SearchQuery> for SearchQuery {
    type Error = VecGraphError;

    fn try_from(core: vecgraph_core::SearchQuery) -> Result<Self, Self::Error> {
        let search_kind = SearchKind::from(core.search_kind);
        let rerank = core.rerank.map(|rp| RerankParams {
            vectors: rp.vector,
            kind: rp.kind,
            weight: rp.weight,
        });

        Ok(SearchQuery {
            query_vecs: core.query_vec,
            top_k: core.top_k as u32,
            search_kind: search_kind.into(),
            rerank,
            exclude_names: core.exclude_names,
            kind: core.kind,
            namespace: core.namespace,
        })
    }
}

impl TryFrom<RerankParams> for vecgraph_core::RerankParams {
    type Error = VecGraphError;

    fn try_from(proto: RerankParams) -> Result<Self, Self::Error> {
        let vector = proto.vectors;

        Ok(vecgraph_core::RerankParams {
            vector,
            kind: proto.kind,
            weight: proto.weight,
        })
    }
}

impl TryFrom<SearchKind> for vecgraph_core::SearchKind {
    type Error = VecGraphError;

    fn try_from(proto: SearchKind) -> Result<Self, Self::Error> {
        match proto {
            SearchKind::Edge => Ok(vecgraph_core::SearchKind::Edge),
            SearchKind::Node => Ok(vecgraph_core::SearchKind::Node),
            SearchKind::AllUnspecified => Ok(vecgraph_core::SearchKind::All),
        }
    }
}

impl TryFrom<SearchResult> for vecgraph_core::SearchResult {
    type Error = VecGraphError;

    fn try_from(proto: SearchResult) -> Result<Self, Self::Error> {
        let search_kind = match SearchKind::try_from(proto.hit_kind).ok() {
            Some(SearchKind::Edge) => vecgraph_core::SearchKind::Edge,
            Some(SearchKind::Node) => vecgraph_core::SearchKind::Node,
            Some(SearchKind::AllUnspecified) => vecgraph_core::SearchKind::All,
            _ => {
                return Err(VecGraphError::Other(format!(
                    "Invalid search hit kind: {}",
                    proto.hit_kind
                )));
            }
        };
        Ok(vecgraph_core::SearchResult {
            node_id: proto.node_id.into(),
            kind: proto.kind,
            score: proto.score,
            hit_kind: search_kind,
        })
    }
}

// Reverse

impl From<vecgraph_core::SearchKind> for SearchKind {
    fn from(core: vecgraph_core::SearchKind) -> Self {
        match core {
            vecgraph_core::SearchKind::Edge => SearchKind::Edge,
            vecgraph_core::SearchKind::Node => SearchKind::Node,
            vecgraph_core::SearchKind::All => SearchKind::AllUnspecified,
        }
    }
}

impl From<vecgraph_core::SearchResult> for SearchResult {
    fn from(core: vecgraph_core::SearchResult) -> Self {
        let hit_kind = SearchKind::from(core.hit_kind);
        SearchResult {
            node_id: core.node_id.0,
            kind: core.kind,
            score: core.score,
            hit_kind: hit_kind.into(),
        }
    }
}

impl From<vecgraph_core::RerankParams> for RerankParams {
    fn from(core: vecgraph_core::RerankParams) -> Self {
        RerankParams {
            vectors: core.vector,
            kind: core.kind,
            weight: core.weight,
        }
    }
}
