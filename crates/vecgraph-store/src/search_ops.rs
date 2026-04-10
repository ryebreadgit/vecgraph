use crate::VecGraphStore;
use std::collections::BinaryHeap;
use vecgraph_core::{
    Node, NodeId, ScoredHit, SearchQuery, SearchResult, StorageKey, VecGraphError, VectorScanQuery,
    cosine_distance,
};

pub async fn search(
    store: &VecGraphStore,
    request: &SearchQuery,
) -> Result<Vec<SearchResult>, VecGraphError> {
    let scan = VectorScanQuery {
        edge_kind: request.edge_kind.clone(),
        namespace: request.namespace.clone(),
    };

    // Over-fetch when we need to post-filter or re-rank
    let fetch_k = if !request.exclude_names.is_empty() {
        request.top_k * 5
    } else if request.rerank.is_some() {
        request.top_k * 3
    } else {
        request.top_k
    };

    let heap = scan_vectors(store, &request.query_vec, &scan, fetch_k).await;
    let mut results: Vec<ScoredHit> = heap.into_sorted_vec();

    // Exclude hits whose node has a name in the exclude list (case-insensitive match)
    if !request.exclude_names.is_empty() {
        results = filter_excluded(store, results, &request.exclude_names).await;
    }

    // Re-rank if requested, then sort by new score
    if let Some(rerank) = &request.rerank {
        rerank_results(store, &mut results, rerank).await;
        results.sort_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // Truncate to requested top_k and convert to SearchHit
    results.truncate(request.top_k);
    Ok(results
        .into_iter()
        .map(|hit| SearchResult {
            node_id: NodeId::new(String::from_utf8_lossy(&hit.node_id_bytes).into_owned()),
            edge_kind: hit.edge_kind,
            score: hit.score,
        })
        .collect())
}

async fn scan_vectors(
    store: &VecGraphStore,
    query_vec: &[f32],
    scan: &VectorScanQuery,
    top_k: usize,
) -> BinaryHeap<ScoredHit> {
    let mut heap: BinaryHeap<ScoredHit> = BinaryHeap::new();
    let prefix = scan.scan_prefix();
    let rx = store.kv.scan(scan.partition(), Some(prefix.as_bytes()), 64);

    while let Ok(entry) = rx.recv().await {
        let (key_bytes, value) = match entry {
            Ok(pair) => pair,
            Err(_) => continue,
        };

        let candidate: &[f32] = match bytemuck::try_cast_slice(&value) {
            Ok(slice) => slice,
            Err(_) => continue,
        };

        if candidate.len() != query_vec.len() {
            continue;
        }

        let dist = cosine_distance(query_vec, candidate);

        if heap.len() < top_k || dist < heap.peek().map_or(f32::MAX, |top| top.score) {
            if heap.len() == top_k {
                heap.pop(); // evict worst
            }

            let key_str = String::from_utf8_lossy(&key_bytes);
            let node_id = VectorScanQuery::node_id_from_key(&key_str)
                .unwrap_or("")
                .as_bytes()
                .to_vec();

            heap.push(ScoredHit {
                node_id_bytes: node_id,
                edge_kind: scan.edge_kind.clone(),
                score: dist,
            });
        }
    }

    heap
}

async fn filter_excluded(
    store: &VecGraphStore,
    results: Vec<ScoredHit>,
    exclude_names: &[String],
) -> Vec<ScoredHit> {
    let mut filtered = Vec::with_capacity(results.len());

    for hit in results {
        let node_id = NodeId::new(String::from_utf8_lossy(&hit.node_id_bytes).into_owned());
        let key = StorageKey::Node(node_id);

        // Fetch the node to check its name
        if let Ok(Some(bytes)) = store.kv.get(key.partition(), key.key().as_bytes()).await {
            if let Ok(node) = serde_json::from_slice::<Node>(&bytes) {
                if exclude_names
                    .iter()
                    .any(|name| node.name.eq_ignore_ascii_case(name))
                {
                    continue;
                }
            }
        }

        filtered.push(hit);
    }

    filtered
}

async fn rerank_results(
    store: &VecGraphStore,
    results: &mut [ScoredHit],
    rerank: &vecgraph_core::RerankParams,
) {
    let rerank_scan = VectorScanQuery {
        edge_kind: rerank.edge_kind.clone(),
        namespace: None, // re-rank across all namespaces
    };
    let rerank_prefix = rerank_scan.scan_prefix();

    for hit in results.iter_mut() {
        let node_id_str = String::from_utf8_lossy(&hit.node_id_bytes);
        let rerank_key = format!("{}{}", rerank_prefix, node_id_str);

        if let Ok(Some(bytes)) = store
            .kv
            .get(rerank_scan.partition(), rerank_key.as_bytes())
            .await
        {
            if let Ok(rerank_vec) = bytemuck::try_cast_slice::<u8, f32>(&bytes) {
                if rerank_vec.len() == rerank.vector.len() {
                    let rerank_dist = cosine_distance(&rerank.vector, rerank_vec);
                    hit.score = (1.0 - rerank.weight) * hit.score + rerank.weight * rerank_dist;
                }
            }
        }
    }
}
