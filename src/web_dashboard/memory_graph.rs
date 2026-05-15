use std::collections::BTreeMap;

use crate::memory;
use crate::web_dashboard::read::short;
use crate::web_dashboard::{GraphEdge, GraphNode, MemoryGraph};

pub fn build_memory_graph(records: &[memory::MemoryRecord]) -> MemoryGraph {
    let mut nodes = BTreeMap::new();
    let mut edges = Vec::new();
    for record in records {
        nodes.entry(record.tx_id.clone()).or_insert(GraphNode {
            id: record.tx_id.clone(),
            label: short(&record.tx_id),
            kind: "transaction".to_string(),
        });
        nodes.insert(
            record.id.clone(),
            GraphNode {
                id: record.id.clone(),
                label: short(&record.kind),
                kind: "memory".to_string(),
            },
        );
        edges.push(GraphEdge {
            from: record.tx_id.clone(),
            to: record.id.clone(),
            label: record.kind.clone(),
        });
    }
    MemoryGraph {
        nodes: nodes.into_values().collect(),
        edges,
    }
}
