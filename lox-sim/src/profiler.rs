use std::cmp::Reverse;

#[derive(Debug, Clone)]
pub struct SimProfiler {
    tick_count: u64,
    total_wall_ns: u64,
    block_eval_count: Vec<u64>,
    block_time_ns: Vec<u64>,
    dirty_skip_count: u64,
    signals_changed: u64,
    total_block_evals: u64,
    block_types: Vec<String>,
}

impl SimProfiler {
    pub fn new(block_count: usize) -> Self {
        Self {
            tick_count: 0,
            total_wall_ns: 0,
            block_eval_count: vec![0; block_count],
            block_time_ns: vec![0; block_count],
            dirty_skip_count: 0,
            signals_changed: 0,
            total_block_evals: 0,
            block_types: (0..block_count)
                .map(|block_id| format!("Block#{block_id}"))
                .collect(),
        }
    }

    pub fn record_tick(&mut self, wall_ns: u64, blocks_evaluated: usize, blocks_skipped: usize) {
        self.tick_count += 1;
        self.total_wall_ns = self.total_wall_ns.saturating_add(wall_ns);
        self.total_block_evals = self
            .total_block_evals
            .saturating_add(blocks_evaluated as u64);
        self.dirty_skip_count = self.dirty_skip_count.saturating_add(blocks_skipped as u64);
    }

    pub fn record_block(&mut self, block_id: usize, time_ns: u64) {
        if let Some(count) = self.block_eval_count.get_mut(block_id) {
            *count = count.saturating_add(1);
        }
        if let Some(total_time) = self.block_time_ns.get_mut(block_id) {
            *total_time = total_time.saturating_add(time_ns);
        }
    }

    pub fn report(&self) -> ProfileReport {
        let total_wall_time_ms = self.total_wall_ns as f64 / 1_000_000.0;
        let total_wall_secs = self.total_wall_ns as f64 / 1_000_000_000.0;
        let total_block_time_ns: u64 = self.block_time_ns.iter().sum();
        let total_considered = self.total_block_evals + self.dirty_skip_count;

        let mut hottest_blocks = self
            .block_time_ns
            .iter()
            .enumerate()
            .filter(|(_, time_ns)| **time_ns > 0)
            .map(|(block_id, &time_ns)| {
                (
                    block_id,
                    self.block_types
                        .get(block_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Block#{block_id}")),
                    self.block_eval_count[block_id],
                    if total_block_time_ns > 0 {
                        (time_ns as f64 / total_block_time_ns as f64) * 100.0
                    } else {
                        0.0
                    },
                )
            })
            .collect::<Vec<_>>();
        hottest_blocks.sort_by_key(|(block_id, _, _, pct_time)| {
            (Reverse((pct_time * 1_000_000.0) as u64), *block_id)
        });
        hottest_blocks.truncate(10);

        ProfileReport {
            total_ticks: self.tick_count,
            total_wall_time_ms,
            ticks_per_second: if total_wall_secs > 0.0 {
                self.tick_count as f64 / total_wall_secs
            } else {
                0.0
            },
            block_evals_per_second: if total_wall_secs > 0.0 {
                self.total_block_evals as f64 / total_wall_secs
            } else {
                0.0
            },
            avg_blocks_per_tick: if self.tick_count > 0 {
                self.total_block_evals as f64 / self.tick_count as f64
            } else {
                0.0
            },
            dirty_skip_ratio: if total_considered > 0 {
                self.dirty_skip_count as f64 / total_considered as f64
            } else {
                0.0
            },
            hottest_blocks,
        }
    }

    pub fn record_signal_changes(&mut self, changed: u64) {
        self.signals_changed = self.signals_changed.saturating_add(changed);
    }

    pub(crate) fn set_block_types(&mut self, block_types: Vec<String>) {
        self.block_types = block_types;
    }
}

#[derive(Debug, Clone)]
pub struct ProfileReport {
    pub total_ticks: u64,
    pub total_wall_time_ms: f64,
    pub ticks_per_second: f64,
    pub block_evals_per_second: f64,
    pub avg_blocks_per_tick: f64,
    pub dirty_skip_ratio: f64,
    pub hottest_blocks: Vec<(usize, String, u64, f64)>,
}
