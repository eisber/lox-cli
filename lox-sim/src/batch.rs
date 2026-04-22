use std::collections::HashMap;

use rayon::prelude::*;

use crate::engine::SimEngine;
use crate::graph::SimGraph;

pub struct BatchRunner {
    base_graph: SimGraph,
}

pub struct BatchConfig {
    pub param_sweeps: Vec<ParamSweep>,
    pub input_timeline: Vec<(f64, String, f64)>,
    pub duration_s: f64,
    pub dt: f64,
    pub outputs_to_track: Vec<String>,
}

pub struct ParamSweep {
    pub block_name: String,
    pub param: String,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchResult {
    pub params: Vec<f64>,
    pub output_traces: HashMap<String, Vec<f64>>,
    pub energy_kwh: f64,
    pub comfort_score: f64,
}

impl BatchRunner {
    pub fn new(graph: SimGraph) -> Self {
        Self { base_graph: graph }
    }

    /// Run all parameter combinations in parallel using rayon
    pub fn run(&self, config: &BatchConfig) -> Vec<BatchResult> {
        let combinations = self.param_combinations(&config.param_sweeps);
        combinations
            .par_iter()
            .map(|params| {
                self.run_single(
                    params,
                    &config.input_timeline,
                    config.dt,
                    config.duration_s,
                    &config.outputs_to_track,
                )
            })
            .collect()
    }

    /// Run a single simulation, return traces
    pub fn run_single(
        &self,
        params: &[(String, String, f64)],
        inputs: &[(f64, String, f64)],
        dt: f64,
        duration: f64,
        outputs: &[String],
    ) -> BatchResult {
        let mut engine = SimEngine::new(self.base_graph.clone());

        for (block_name, param, value) in params {
            assert!(
                engine.set_param(block_name, param, *value),
                "unknown batch parameter {block_name}.{param}"
            );
        }

        let mut timeline = inputs.to_vec();
        timeline.sort_by(|left, right| left.0.total_cmp(&right.0));
        let steps = if dt > 0.0 {
            (duration / dt).ceil().max(0.0) as usize
        } else {
            0
        };
        let mut input_index = 0usize;
        let mut elapsed = 0.0f64;
        let mut output_traces: HashMap<String, Vec<f64>> = outputs
            .iter()
            .cloned()
            .map(|name| (name, Vec::with_capacity(steps)))
            .collect();

        let mut accumulated_output = 0.0;
        let mut sample_count = 0usize;

        for _ in 0..steps {
            while input_index < timeline.len() && timeline[input_index].0 <= elapsed + f64::EPSILON
            {
                let (_, ref input_name, value) = timeline[input_index];
                assert!(
                    engine.set_input(input_name, value),
                    "unknown batch input {input_name}"
                );
                input_index += 1;
            }

            engine.tick(dt);

            for output_name in outputs {
                let value = engine.get_output(output_name);
                output_traces
                    .entry(output_name.clone())
                    .or_default()
                    .push(value);
                accumulated_output += value.max(0.0);
                sample_count += 1;
            }

            elapsed += dt;
        }

        let average_output = if sample_count > 0 {
            accumulated_output / sample_count as f64
        } else {
            0.0
        };

        BatchResult {
            params: params.iter().map(|(_, _, value)| *value).collect(),
            output_traces,
            energy_kwh: accumulated_output * dt / 3_600_000.0,
            comfort_score: if sample_count > 0 {
                1.0 / (1.0 + (average_output - 21.0).abs())
            } else {
                1.0
            },
        }
    }

    fn param_combinations(&self, sweeps: &[ParamSweep]) -> Vec<Vec<(String, String, f64)>> {
        if sweeps.is_empty() {
            return vec![Vec::new()];
        }

        let mut combinations = vec![Vec::new()];
        for sweep in sweeps {
            let mut next = Vec::new();
            for combination in &combinations {
                for value in &sweep.values {
                    let mut expanded = combination.clone();
                    expanded.push((sweep.block_name.clone(), sweep.param.clone(), *value));
                    next.push(expanded);
                }
            }
            combinations = next;
        }
        combinations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{Gain, PassThrough};

    fn batch_graph() -> SimGraph {
        let mut graph = SimGraph::new();
        let src = graph.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let gain = graph.add_block("Heater", Box::new(Gain), &["I1"], &["Q"], &["Factor"]);
        graph
            .add_wire(
                graph.find_connector(src, "Q").unwrap(),
                graph.find_connector(gain, "I1").unwrap(),
            )
            .unwrap();
        graph
    }

    #[test]
    fn batch_sweep_varies_results() {
        let runner = BatchRunner::new(batch_graph());
        let config = BatchConfig {
            param_sweeps: vec![ParamSweep {
                block_name: "Heater".to_string(),
                param: "Factor".to_string(),
                values: vec![1.0, 2.0, 3.0],
            }],
            input_timeline: vec![(0.0, "Src".to_string(), 10.0)],
            duration_s: 1.0,
            dt: 0.1,
            outputs_to_track: vec!["Heater".to_string()],
        };

        let results = runner.run(&config);
        assert_eq!(results.len(), 3);
        let traces: Vec<Vec<f64>> = results
            .iter()
            .map(|result| result.output_traces["Heater"].clone())
            .collect();
        assert_ne!(traces[0], traces[1]);
        assert_ne!(traces[1], traces[2]);
    }

    #[test]
    fn parallel_batch_matches_serial_execution() {
        let runner = BatchRunner::new(batch_graph());
        let config = BatchConfig {
            param_sweeps: vec![ParamSweep {
                block_name: "Heater".to_string(),
                param: "Factor".to_string(),
                values: vec![0.5, 1.0, 1.5, 2.0],
            }],
            input_timeline: vec![(0.0, "Src".to_string(), 4.0), (0.3, "Src".to_string(), 8.0)],
            duration_s: 1.0,
            dt: 0.1,
            outputs_to_track: vec!["Heater".to_string()],
        };

        let parallel = runner.run(&config);
        let serial: Vec<BatchResult> = config.param_sweeps[0]
            .values
            .iter()
            .map(|value| {
                runner.run_single(
                    &[("Heater".to_string(), "Factor".to_string(), *value)],
                    &config.input_timeline,
                    config.dt,
                    config.duration_s,
                    &config.outputs_to_track,
                )
            })
            .collect();

        assert_eq!(parallel, serial);
    }
}
