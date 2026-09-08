use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::blocks::Block;
use crate::types::*;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors during graph construction.
#[derive(Debug)]
pub enum GraphError {
    /// Connector ID does not exist.
    InvalidConnector(ConnectorId),
    /// Attempted to wire from a non-output connector.
    ConnectorNotOutput(ConnectorId),
    /// Attempted to wire to a non-input connector.
    ConnectorNotInput(ConnectorId),
    /// Input connector already has a wire connected.
    InputAlreadyWired(ConnectorId),
    /// Block ID does not exist.
    InvalidBlock(BlockId),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConnector(id) => write!(f, "invalid connector ID: {id}"),
            Self::ConnectorNotOutput(id) => write!(f, "connector {id} is not an output"),
            Self::ConnectorNotInput(id) => write!(f, "connector {id} is not an input"),
            Self::InputAlreadyWired(id) => write!(f, "input connector {id} is already wired"),
            Self::InvalidBlock(id) => write!(f, "invalid block ID: {id}"),
        }
    }
}

impl std::error::Error for GraphError {}

// ---------------------------------------------------------------------------
// Structural metadata
// ---------------------------------------------------------------------------

/// Structural info about a block in the graph.
#[derive(Clone)]
pub struct BlockInfo {
    pub id: BlockId,
    pub name: String,
    pub block_type: String,
    pub room: Option<String>,
    pub inputs: Vec<ConnectorId>,
    pub outputs: Vec<ConnectorId>,
    pub params: Vec<ConnectorId>,
}

/// Result of topological sort.
pub struct TopologicalResult {
    /// Block evaluation order (valid even when feedback is present).
    pub order: Vec<BlockId>,
    /// Wire-level feedback edges that should use previous-tick values.
    pub feedback_wires: HashSet<(ConnectorId, ConnectorId)>,
}

impl TopologicalResult {
    /// Returns `true` if the graph contains feedback loops.
    pub fn has_feedback(&self) -> bool {
        !self.feedback_wires.is_empty()
    }
}

// ---------------------------------------------------------------------------
// SimGraph
// ---------------------------------------------------------------------------

/// Simulation graph: blocks, connectors, wiring, topological sort.
#[derive(Clone)]
pub struct SimGraph {
    pub(crate) blocks: Vec<BlockInfo>,
    pub(crate) block_impls: Vec<Box<dyn Block>>,
    pub(crate) connectors: Vec<ConnectorInfo>,
    pub(crate) wires: Vec<(ConnectorId, ConnectorId)>,
    pub(crate) input_source: HashMap<ConnectorId, ConnectorId>,
    block_by_name: HashMap<String, BlockId>,
    blocks_by_name: HashMap<String, Vec<BlockId>>,
    /// Non-fatal diagnostics collected during graph construction (e.g. wires
    /// that could not be honored because the source was not an output).
    warnings: Vec<String>,
}

impl SimGraph {
    pub fn new() -> Self {
        SimGraph {
            blocks: Vec::new(),
            block_impls: Vec::new(),
            connectors: Vec::new(),
            wires: Vec::new(),
            input_source: HashMap::new(),
            block_by_name: HashMap::new(),
            blocks_by_name: HashMap::new(),
            warnings: Vec::new(),
        }
    }

    // --- Construction -------------------------------------------------------

    /// Add a block with named input / output / parameter connectors.
    /// Returns the `BlockId` assigned to the new block.
    pub fn add_block(
        &mut self,
        name: impl Into<String>,
        block: Box<dyn Block>,
        input_keys: &[&str],
        output_keys: &[&str],
        param_keys: &[&str],
    ) -> BlockId {
        let block_type = block.block_type().to_string();
        self.add_block_with_type(name, block, input_keys, output_keys, param_keys, block_type)
    }

    /// Add a block with an explicit type string (used by the parser when the
    /// XML Type attribute differs from the runtime block_type, e.g. unknown blocks
    /// mapped to PassThrough).
    pub fn add_block_with_type(
        &mut self,
        name: impl Into<String>,
        block: Box<dyn Block>,
        input_keys: &[&str],
        output_keys: &[&str],
        param_keys: &[&str],
        block_type: impl Into<String>,
    ) -> BlockId {
        let block_id = self.blocks.len();
        let name = name.into();

        let inputs = self.create_connectors(input_keys, ConnectorDir::Input, block_id);
        let outputs = self.create_connectors(output_keys, ConnectorDir::Output, block_id);
        let params = self.create_connectors(param_keys, ConnectorDir::Parameter, block_id);

        self.block_by_name.insert(name.clone(), block_id);
        self.blocks_by_name
            .entry(name.clone())
            .or_default()
            .push(block_id);
        self.blocks.push(BlockInfo {
            id: block_id,
            name,
            block_type: block_type.into(),
            room: None,
            inputs,
            outputs,
            params,
        });
        self.block_impls.push(block);
        block_id
    }

    /// Wire an output connector to an input connector.
    pub fn add_wire(&mut self, from: ConnectorId, to: ConnectorId) -> Result<(), GraphError> {
        if from >= self.connectors.len() {
            return Err(GraphError::InvalidConnector(from));
        }
        if to >= self.connectors.len() {
            return Err(GraphError::InvalidConnector(to));
        }
        if self.connectors[from].dir != ConnectorDir::Output {
            return Err(GraphError::ConnectorNotOutput(from));
        }
        // Parameter connectors accept wires too: Loxone lets any parameter
        // (e.g. Formula Input1-Input4, comparator Input2) be driven by a
        // wire instead of a fixed value.
        if self.connectors[to].dir == ConnectorDir::Output {
            return Err(GraphError::ConnectorNotInput(to));
        }
        if self.input_source.contains_key(&to) {
            return Err(GraphError::InputAlreadyWired(to));
        }

        self.input_source.insert(to, from);
        self.wires.push((from, to));
        Ok(())
    }

    // --- Topological sort ---------------------------------------------------

    /// Compute topological block order with back-edge detection for feedback loops.
    ///
    /// Self-wires (output -> input of the same block) are **not** treated as
    /// feedback — they are ignored in the dependency graph.
    pub fn topological_order(&self) -> TopologicalResult {
        let n = self.blocks.len();
        if n == 0 {
            return TopologicalResult {
                order: Vec::new(),
                feedback_wires: HashSet::new(),
            };
        }

        // Build unique block-level edges, remembering the underlying wires.
        let mut block_edge_wires: HashMap<(BlockId, BlockId), Vec<(ConnectorId, ConnectorId)>> =
            HashMap::new();
        for &(from_cid, to_cid) in &self.wires {
            let src = self.connectors[from_cid].block_id;
            let dst = self.connectors[to_cid].block_id;
            if src != dst {
                block_edge_wires
                    .entry((src, dst))
                    .or_default()
                    .push((from_cid, to_cid));
            }
        }

        // Deduplicated adjacency list.
        let mut adj: Vec<Vec<BlockId>> = vec![Vec::new(); n];
        for &(src, dst) in block_edge_wires.keys() {
            adj[src].push(dst);
        }
        for list in &mut adj {
            list.sort_unstable();
            list.dedup();
        }

        // DFS topological sort.
        let mut visited = vec![0u8; n]; // 0 white, 1 gray, 2 black
        let mut order = Vec::with_capacity(n);
        let mut back_edges: Vec<(BlockId, BlockId)> = Vec::new();

        for i in 0..n {
            if visited[i] == 0 {
                Self::dfs(i, &adj, &mut visited, &mut order, &mut back_edges);
            }
        }
        order.reverse();

        // Map block-level back edges to wire-level feedback.
        let mut feedback_wires = HashSet::new();
        for (src, dst) in &back_edges {
            if let Some(wires) = block_edge_wires.get(&(*src, *dst)) {
                for &w in wires {
                    feedback_wires.insert(w);
                }
            }
        }

        TopologicalResult {
            order,
            feedback_wires,
        }
    }

    fn dfs(
        node: BlockId,
        adj: &[Vec<BlockId>],
        visited: &mut [u8],
        order: &mut Vec<BlockId>,
        back_edges: &mut Vec<(BlockId, BlockId)>,
    ) {
        visited[node] = 1;
        for &neighbor in &adj[node] {
            match visited[neighbor] {
                0 => Self::dfs(neighbor, adj, visited, order, back_edges),
                1 => back_edges.push((node, neighbor)),
                _ => {}
            }
        }
        visited[node] = 2;
        order.push(node);
    }

    // --- Accessors ----------------------------------------------------------

    pub fn connector_count(&self) -> usize {
        self.connectors.len()
    }
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
    pub fn block_info(&self, id: BlockId) -> &BlockInfo {
        &self.blocks[id]
    }
    pub fn connector(&self, id: ConnectorId) -> &ConnectorInfo {
        &self.connectors[id]
    }
    pub fn set_connector_default(
        &mut self,
        id: ConnectorId,
        value: Signal,
    ) -> Result<(), GraphError> {
        if id >= self.connectors.len() {
            return Err(GraphError::InvalidConnector(id));
        }
        self.connectors[id].default_value = value;
        Ok(())
    }
    pub fn wires(&self) -> &[(ConnectorId, ConnectorId)] {
        &self.wires
    }

    /// Non-fatal diagnostics collected during graph construction.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Record a non-fatal diagnostic (e.g. a wire that could not be honored).
    pub fn push_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// Describe a connector as `"<block title> (<Type>).<key> [uuid:<uuid>]"`
    /// for human-readable diagnostics. `uuid` may be empty if unknown.
    pub fn describe_connector(&self, cid: ConnectorId, uuid: &str) -> String {
        if cid >= self.connectors.len() {
            return format!("connector#{cid}");
        }
        let conn = &self.connectors[cid];
        let block = &self.blocks[conn.block_id];
        if uuid.is_empty() {
            format!("'{}' ({}).{}", block.name, block.block_type, conn.key)
        } else {
            format!(
                "'{}' ({}).{} [uuid:{}]",
                block.name, block.block_type, conn.key, uuid
            )
        }
    }

    pub fn input_source_of(&self, input_cid: ConnectorId) -> Option<ConnectorId> {
        self.input_source.get(&input_cid).copied()
    }

    pub fn find_block_by_name(&self, name: &str) -> Option<BlockId> {
        self.block_by_name.get(name).copied()
    }

    /// Find all blocks with a given name (handles duplicates in parsed configs).
    pub fn find_blocks_by_name(&self, name: &str) -> &[BlockId] {
        self.blocks_by_name
            .get(name)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Find a connector on a block by its key (e.g. `"I1"`, `"Q"`).
    pub fn find_connector(&self, block_id: BlockId, key: &str) -> Option<ConnectorId> {
        let info = &self.blocks[block_id];
        info.inputs
            .iter()
            .chain(info.outputs.iter())
            .chain(info.params.iter())
            .find(|&&cid| self.connectors[cid].key == key)
            .copied()
    }

    /// Move block implementations out (consumed by `SimEngine`).
    pub(crate) fn take_block_impls(&mut self) -> Vec<Box<dyn Block>> {
        std::mem::take(&mut self.block_impls)
    }

    // --- Type/room query methods --------------------------------------------

    /// Find all blocks of a given type.
    pub fn blocks_by_type(&self, block_type: &str) -> Vec<BlockId> {
        self.blocks
            .iter()
            .filter(|b| b.block_type == block_type)
            .map(|b| b.id)
            .collect()
    }

    /// Find all blocks of a given type in a specific room.
    pub fn blocks_by_type_in_room(&self, block_type: &str, room: &str) -> Vec<BlockId> {
        self.blocks
            .iter()
            .filter(|b| b.block_type == block_type && b.room.as_deref() == Some(room))
            .map(|b| b.id)
            .collect()
    }

    /// Find all output connectors of a block type that match a key pattern.
    pub fn outputs_matching(
        &self,
        block_type: &str,
        room: Option<&str>,
        connector_key: &str,
    ) -> Vec<ConnectorId> {
        self.blocks
            .iter()
            .filter(|b| {
                b.block_type == block_type && room.is_none_or(|r| b.room.as_deref() == Some(r))
            })
            .flat_map(|b| {
                b.outputs
                    .iter()
                    .filter(|&&cid| self.connectors[cid].key == connector_key)
                    .copied()
            })
            .collect()
    }

    // --- Internal helpers ---------------------------------------------------

    fn create_connectors(
        &mut self,
        keys: &[&str],
        dir: ConnectorDir,
        block_id: BlockId,
    ) -> Vec<ConnectorId> {
        keys.iter()
            .map(|key| {
                let cid = self.connectors.len();
                self.connectors.push(ConnectorInfo {
                    id: cid,
                    key: key.to_string(),
                    dir,
                    block_id,
                    default_value: 0.0,
                });
                cid
            })
            .collect()
    }
}

impl Default for SimGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::PassThrough;

    fn pt() -> Box<dyn Block> {
        Box::new(PassThrough)
    }

    #[test]
    fn empty_graph() {
        let g = SimGraph::new();
        assert_eq!(g.block_count(), 0);
        assert_eq!(g.connector_count(), 0);
        let r = g.topological_order();
        assert!(r.order.is_empty());
        assert!(!r.has_feedback());
    }

    #[test]
    fn single_block() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        assert_eq!(g.block_count(), 1);
        assert_eq!(g.connector_count(), 2);

        let r = g.topological_order();
        assert_eq!(r.order, vec![a]);
        assert!(!r.has_feedback());
    }

    #[test]
    fn linear_chain_topo_order() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let c = g.add_block("C", pt(), &["I1"], &["Q"], &[]);

        let a_q = g.find_connector(a, "Q").unwrap();
        let b_i = g.find_connector(b, "I1").unwrap();
        let b_q = g.find_connector(b, "Q").unwrap();
        let c_i = g.find_connector(c, "I1").unwrap();

        g.add_wire(a_q, b_i).unwrap();
        g.add_wire(b_q, c_i).unwrap();

        let r = g.topological_order();
        assert!(!r.has_feedback());

        let pos = |id: BlockId| r.order.iter().position(|&x| x == id).unwrap();
        assert!(pos(a) < pos(b));
        assert!(pos(b) < pos(c));
    }

    #[test]
    fn diamond_topo_order() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q1", "Q2"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let c = g.add_block("C", pt(), &["I1"], &["Q"], &[]);
        let d = g.add_block("D", pt(), &["I1", "I2"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(a, "Q1").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(a, "Q2").unwrap(),
            g.find_connector(c, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(d, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(c, "Q").unwrap(),
            g.find_connector(d, "I2").unwrap(),
        )
        .unwrap();

        let r = g.topological_order();
        assert!(!r.has_feedback());

        let pos = |id: BlockId| r.order.iter().position(|&x| x == id).unwrap();
        assert!(pos(a) < pos(b));
        assert!(pos(a) < pos(c));
        assert!(pos(b) < pos(d));
        assert!(pos(c) < pos(d));
    }

    #[test]
    fn independent_blocks_all_present() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let c = g.add_block("C", pt(), &["I1"], &["Q"], &[]);

        let r = g.topological_order();
        assert!(!r.has_feedback());
        assert_eq!(r.order.len(), 3);
        assert!(r.order.contains(&a));
        assert!(r.order.contains(&b));
        assert!(r.order.contains(&c));
    }

    #[test]
    fn feedback_loop_detected() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);

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

        let r = g.topological_order();
        assert!(r.has_feedback());
        assert_eq!(r.order.len(), 2);
        assert!(r.order.contains(&a));
        assert!(r.order.contains(&b));
        assert_eq!(r.feedback_wires.len(), 1);
    }

    #[test]
    fn three_block_cycle() {
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
        g.add_wire(
            g.find_connector(c, "Q").unwrap(),
            g.find_connector(a, "I1").unwrap(),
        )
        .unwrap();

        let r = g.topological_order();
        assert!(r.has_feedback());
        assert_eq!(r.order.len(), 3);
        assert_eq!(r.feedback_wires.len(), 1);
    }

    #[test]
    fn self_wire_not_feedback() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(a, "I1").unwrap(),
        )
        .unwrap();

        let r = g.topological_order();
        assert!(!r.has_feedback());
        assert_eq!(r.order, vec![a]);
    }

    #[test]
    fn wire_invalid_connector() {
        let mut g = SimGraph::new();
        assert!(matches!(
            g.add_wire(0, 1),
            Err(GraphError::InvalidConnector(0))
        ));
    }

    #[test]
    fn wire_from_non_output() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);

        let a_i = g.find_connector(a, "I1").unwrap();
        let b_i = g.find_connector(b, "I1").unwrap();
        assert!(matches!(
            g.add_wire(a_i, b_i),
            Err(GraphError::ConnectorNotOutput(_))
        ));
    }

    #[test]
    fn wire_to_non_input() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);

        let a_q = g.find_connector(a, "Q").unwrap();
        let b_q = g.find_connector(b, "Q").unwrap();
        assert!(matches!(
            g.add_wire(a_q, b_q),
            Err(GraphError::ConnectorNotInput(_))
        ));
    }

    #[test]
    fn wire_input_already_wired() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &[], &["Q"], &[]);
        let b = g.add_block("B", pt(), &[], &["Q"], &[]);
        let c = g.add_block("C", pt(), &["I1"], &[], &[]);

        let a_q = g.find_connector(a, "Q").unwrap();
        let b_q = g.find_connector(b, "Q").unwrap();
        let c_i = g.find_connector(c, "I1").unwrap();

        g.add_wire(a_q, c_i).unwrap();
        assert!(matches!(
            g.add_wire(b_q, c_i),
            Err(GraphError::InputAlreadyWired(_))
        ));
    }

    #[test]
    fn find_connector_by_key() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1", "I2"], &["Q", "AQ"], &["P1"]);

        assert!(g.find_connector(a, "I1").is_some());
        assert!(g.find_connector(a, "I2").is_some());
        assert!(g.find_connector(a, "Q").is_some());
        assert!(g.find_connector(a, "AQ").is_some());
        assert!(g.find_connector(a, "P1").is_some());
        assert!(g.find_connector(a, "NOPE").is_none());
    }

    #[test]
    fn find_block_by_name() {
        let mut g = SimGraph::new();
        g.add_block("Light", pt(), &[], &["Q"], &[]);
        g.add_block("Blind", pt(), &[], &["Q"], &[]);

        assert_eq!(g.find_block_by_name("Light"), Some(0));
        assert_eq!(g.find_block_by_name("Blind"), Some(1));
        assert_eq!(g.find_block_by_name("Missing"), None);
    }

    #[test]
    fn connector_count_accumulates() {
        let mut g = SimGraph::new();
        assert_eq!(g.connector_count(), 0);
        g.add_block("A", pt(), &["I1"], &["Q"], &["P1"]);
        assert_eq!(g.connector_count(), 3);
        g.add_block("B", pt(), &["I1", "I2"], &["Q"], &[]);
        assert_eq!(g.connector_count(), 6);
    }

    #[test]
    fn block_with_params() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &[], &["Q"], &["P1", "P2"]);
        let info = g.block_info(a);
        assert_eq!(info.params.len(), 2);
        assert_eq!(g.connector(info.params[0]).key, "P1");
        assert_eq!(g.connector(info.params[1]).key, "P2");
        assert_eq!(g.connector(info.params[0]).dir, ConnectorDir::Parameter);
    }

    #[test]
    fn fan_out_topo_order() {
        let mut g = SimGraph::new();
        let src = g.add_block("Src", pt(), &[], &["Q"], &[]);
        let d1 = g.add_block("D1", pt(), &["I1"], &["Q"], &[]);
        let d2 = g.add_block("D2", pt(), &["I1"], &["Q"], &[]);
        let d3 = g.add_block("D3", pt(), &["I1"], &["Q"], &[]);

        let src_q = g.find_connector(src, "Q").unwrap();
        g.add_wire(src_q, g.find_connector(d1, "I1").unwrap())
            .unwrap();
        g.add_wire(src_q, g.find_connector(d2, "I1").unwrap())
            .unwrap();
        g.add_wire(src_q, g.find_connector(d3, "I1").unwrap())
            .unwrap();

        let r = g.topological_order();
        assert!(!r.has_feedback());
        let pos = |id: BlockId| r.order.iter().position(|&x| x == id).unwrap();
        assert!(pos(src) < pos(d1));
        assert!(pos(src) < pos(d2));
        assert!(pos(src) < pos(d3));
    }

    #[test]
    fn error_display() {
        let e = GraphError::InvalidConnector(42);
        assert_eq!(e.to_string(), "invalid connector ID: 42");

        let e = GraphError::InputAlreadyWired(7);
        assert_eq!(e.to_string(), "input connector 7 is already wired");
    }

    #[test]
    fn set_connector_default_updates_value() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let i1 = g.find_connector(a, "I1").unwrap();
        g.set_connector_default(i1, 42.0).unwrap();
        assert_eq!(g.connector(i1).default_value, 42.0);
    }
}
