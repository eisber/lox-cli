//! Signal flow tracer — BFS reachability through the wiring graph.
//!
//! Given a [`SimGraph`], determines whether a signal can flow from one block
//! to another by following wires and UUID-shared connectors.

use std::collections::{HashSet, VecDeque};

use crate::graph::SimGraph;
use crate::types::BlockId;

/// Result of a signal trace.
#[derive(Debug, Clone)]
pub struct TraceResult {
    /// Whether a path was found from source to destination.
    pub found: bool,
    /// Block IDs along the path (source first, destination last).
    /// Empty if no path was found.
    pub path: Vec<BlockId>,
    /// Number of block-to-block hops. 0 if not found.
    pub hops: usize,
}

/// Trace whether a signal can flow from `source_name` to `dest_name` via
/// `dest_connector` (e.g. `"I1"`) through the wiring graph.
///
/// Uses BFS over the block-level adjacency implied by wires:
/// an edge exists from block A to block B if any output of A is wired to
/// any input of B.
///
/// Returns `true` if a directed path exists.
pub fn trace_signal(
    graph: &SimGraph,
    source_name: &str,
    dest_name: &str,
    dest_connector: &str,
) -> bool {
    let result = trace(graph, source_name, dest_name, Some(dest_connector));
    result.found
}

/// Full trace with path and hop count.
///
/// If `dest_connector` is `None`, any input on the destination block counts.
/// Handles duplicate block names by trying all matching source/dest blocks.
pub fn trace(
    graph: &SimGraph,
    source_name: &str,
    dest_name: &str,
    dest_connector: Option<&str>,
) -> TraceResult {
    let not_found = TraceResult {
        found: false,
        path: Vec::new(),
        hops: 0,
    };

    let src_ids = graph.find_blocks_by_name(source_name);
    let dst_ids = graph.find_blocks_by_name(dest_name);
    if src_ids.is_empty() || dst_ids.is_empty() {
        return not_found;
    }

    let dst_set: HashSet<BlockId> = dst_ids.iter().copied().collect();

    // If a specific connector is requested, keep only dest blocks that have it.
    let dst_ids: Vec<BlockId> = if let Some(key) = dest_connector {
        dst_ids
            .iter()
            .copied()
            .filter(|&bid| graph.find_connector(bid, key).is_some())
            .collect()
    } else {
        dst_ids.to_vec()
    };
    if dst_ids.is_empty() {
        return not_found;
    }
    let dst_set_filtered: HashSet<BlockId> = dst_ids.iter().copied().collect();

    // Trivial self-trace (any src is also a dst).
    for &sid in src_ids {
        if dst_set_filtered.contains(&sid) {
            return TraceResult {
                found: true,
                path: vec![sid],
                hops: 0,
            };
        }
    }

    // Build block-level forward adjacency from wire list.
    let n = graph.block_count();
    let mut adj: Vec<Vec<BlockId>> = vec![Vec::new(); n];
    for &(from_cid, to_cid) in graph.wires() {
        let src_block = graph.connector(from_cid).block_id;
        let dst_block = graph.connector(to_cid).block_id;
        if src_block != dst_block {
            let include = match dest_connector {
                Some(key) => !dst_set.contains(&dst_block) || graph.connector(to_cid).key == key,
                None => true,
            };
            if include {
                adj[src_block].push(dst_block);
            }
        }
    }

    // Multi-source BFS.
    let mut visited = vec![false; n];
    let mut parent: Vec<Option<BlockId>> = vec![None; n];
    let mut queue = VecDeque::new();

    for &sid in src_ids {
        visited[sid] = true;
        queue.push_back(sid);
    }

    while let Some(cur) = queue.pop_front() {
        if dst_set_filtered.contains(&cur) && !src_ids.contains(&cur) {
            // Reconstruct path.
            let mut path = Vec::new();
            let mut node = cur;
            path.push(node);
            while let Some(p) = parent[node] {
                path.push(p);
                node = p;
            }
            path.reverse();
            let hops = path.len() - 1;
            return TraceResult {
                found: true,
                path,
                hops,
            };
        }
        for &next in &adj[cur] {
            if !visited[next] {
                visited[next] = true;
                parent[next] = Some(cur);
                queue.push_back(next);
            }
        }
    }

    not_found
}

/// Check whether any block in the graph is reachable from `source_name`.
///
/// Returns a set of all reachable block IDs (excluding the source itself,
/// unless there's a self-loop).  Handles duplicate block names by starting
/// BFS from all matching source blocks.
pub fn reachable_from(graph: &SimGraph, source_name: &str) -> HashSet<BlockId> {
    let mut result = HashSet::new();
    let src_ids = graph.find_blocks_by_name(source_name);
    if src_ids.is_empty() {
        return result;
    }

    let n = graph.block_count();
    let mut adj: Vec<Vec<BlockId>> = vec![Vec::new(); n];
    for &(from_cid, to_cid) in graph.wires() {
        let s = graph.connector(from_cid).block_id;
        let d = graph.connector(to_cid).block_id;
        if s != d {
            adj[s].push(d);
        }
    }

    let src_set: HashSet<BlockId> = src_ids.iter().copied().collect();
    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();
    for &sid in src_ids {
        visited[sid] = true;
        queue.push_back(sid);
    }

    while let Some(cur) = queue.pop_front() {
        if !src_set.contains(&cur) {
            result.insert(cur);
        }
        for &next in &adj[cur] {
            if !visited[next] {
                visited[next] = true;
                queue.push_back(next);
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::PassThrough;
    use crate::graph::SimGraph;

    fn pt() -> Box<dyn crate::blocks::Block> {
        Box::new(PassThrough)
    }

    // -- Linear chain: A → B → C --------------------------------------------

    #[test]
    fn linear_chain_found() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let c = g.add_block("C", pt(), &["I1"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(c, "I1").unwrap(),
        )
        .unwrap();

        let result = trace(&g, "A", "C", Some("I1"));
        assert!(result.found);
        assert_eq!(result.hops, 2);
        assert_eq!(result.path, vec![a, b, c]);
    }

    #[test]
    fn linear_chain_signal_trace() {
        let mut g = SimGraph::new();
        let _a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let _b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let _c = g.add_block("C", pt(), &["I1"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(0, "Q").unwrap(),
            g.find_connector(1, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(1, "Q").unwrap(),
            g.find_connector(2, "I1").unwrap(),
        )
        .unwrap();

        assert!(trace_signal(&g, "A", "C", "I1"));
    }

    // -- Fan-out: A → B, A → C ----------------------------------------------

    #[test]
    fn fan_out_both_reachable() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &[], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let c = g.add_block("C", pt(), &["I1"], &["Q"], &[]);

        let a_q = g.find_connector(a, "Q").unwrap();
        g.add_wire(a_q, g.find_connector(b, "I1").unwrap()).unwrap();
        g.add_wire(a_q, g.find_connector(c, "I1").unwrap()).unwrap();

        let rb = trace(&g, "A", "B", Some("I1"));
        assert!(rb.found);
        assert_eq!(rb.hops, 1);

        let rc = trace(&g, "A", "C", Some("I1"));
        assert!(rc.found);
        assert_eq!(rc.hops, 1);
    }

    #[test]
    fn fan_out_reachable_set() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &[], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let c = g.add_block("C", pt(), &["I1"], &["Q"], &[]);

        let a_q = g.find_connector(a, "Q").unwrap();
        g.add_wire(a_q, g.find_connector(b, "I1").unwrap()).unwrap();
        g.add_wire(a_q, g.find_connector(c, "I1").unwrap()).unwrap();

        let reached = reachable_from(&g, "A");
        assert!(reached.contains(&b));
        assert!(reached.contains(&c));
        assert_eq!(reached.len(), 2);
    }

    // -- No path: A is disconnected from D -----------------------------------

    #[test]
    fn no_path() {
        let mut g = SimGraph::new();
        let _a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let _b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let _d = g.add_block("D", pt(), &["I1"], &["Q"], &[]);

        // Wire A → B only, D is disconnected.
        g.add_wire(
            g.find_connector(0, "Q").unwrap(),
            g.find_connector(1, "I1").unwrap(),
        )
        .unwrap();

        let result = trace(&g, "A", "D", Some("I1"));
        assert!(!result.found);
        assert!(result.path.is_empty());
        assert_eq!(result.hops, 0);
    }

    #[test]
    fn no_path_signal_trace() {
        let mut g = SimGraph::new();
        let _a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let _d = g.add_block("D", pt(), &["I1"], &["Q"], &[]);

        assert!(!trace_signal(&g, "A", "D", "I1"));
    }

    // -- Cycle: feedback loop doesn't infinite-loop --------------------------

    #[test]
    fn cycle_no_infinite_loop() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let c = g.add_block("C", pt(), &["I1"], &["Q"], &[]);

        // A → B → C → A (cycle)
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(c, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(c, "Q").unwrap(),
            g.find_connector(a, "I1").unwrap(),
        )
        .unwrap();

        // A → C is reachable in 2 hops despite cycle.
        let result = trace(&g, "A", "C", Some("I1"));
        assert!(result.found);
        assert_eq!(result.hops, 2);

        // B → A is also reachable through the cycle.
        let result2 = trace(&g, "B", "A", Some("I1"));
        assert!(result2.found);
    }

    #[test]
    fn cycle_reachable_from() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);

        // A ↔ B (bidirectional)
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(a, "I1").unwrap(),
        )
        .unwrap();

        let from_a = reachable_from(&g, "A");
        assert!(from_a.contains(&b));
        let from_b = reachable_from(&g, "B");
        assert!(from_b.contains(&a));
    }

    // -- Self-trace ----------------------------------------------------------

    #[test]
    fn self_trace() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);

        let result = trace(&g, "A", "A", None);
        assert!(result.found);
        assert_eq!(result.hops, 0);
        assert_eq!(result.path, vec![a]);
    }

    // -- Non-existent blocks -------------------------------------------------

    #[test]
    fn nonexistent_source() {
        let mut g = SimGraph::new();
        let _a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);

        let result = trace(&g, "MISSING", "A", None);
        assert!(!result.found);
    }

    #[test]
    fn nonexistent_dest() {
        let mut g = SimGraph::new();
        let _a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);

        let result = trace(&g, "A", "MISSING", None);
        assert!(!result.found);
    }

    // -- Fixture-based test (parsed .Loxone XML) -----------------------------

    #[test]
    fn fixture_trace_if_available() {
        // Use the first available .Loxone fixture file.
        let fixture_paths = [
            "../../sps_0267_20260416093258.Loxone",
            "../../sps_0267_20260414190409.Loxone",
        ];

        let graph = fixture_paths.iter().find_map(|path| {
            let abs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
            crate::parser::parse_file(&abs).ok()
        });

        let graph = match graph {
            Some(g) => g,
            None => {
                eprintln!("No fixture .Loxone file found — skipping fixture trace test");
                return;
            }
        };

        // Verify the graph loaded with blocks.
        assert!(graph.block_count() > 0, "fixture loaded but has no blocks");

        // Verify tracer doesn't panic on a real-world graph.
        // Try tracing from every block to itself (trivial).
        for bid in 0..graph.block_count().min(10) {
            let name = &graph.block_info(bid).name;
            let result = trace(&graph, name, name, None);
            assert!(result.found, "self-trace failed for block '{name}'");
        }

        // Verify reachable_from works on the first block without panicking.
        let first_name = &graph.block_info(0).name;
        let _ = reachable_from(&graph, first_name);
    }
}
