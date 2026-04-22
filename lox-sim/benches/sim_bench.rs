use std::path::{Path, PathBuf};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use lox_sim::batch::{BatchConfig, BatchRunner, ParamSweep};
use lox_sim::blocks::{Gain, PassThrough};
use lox_sim::engine::SimEngine;
use lox_sim::graph::SimGraph;
use lox_sim::parser::parse_file;

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("lox-sim has repo parent")
        .join("tests/eval/fixture.Loxone")
}

fn bench_tick_performance(c: &mut Criterion) {
    let graph = parse_file(fixture_path()).expect("fixture parses");

    c.bench_function("bench_tick_performance", |b| {
        b.iter(|| {
            let mut engine = SimEngine::new(graph.clone());
            engine.enable_profiling();
            for _ in 0..100_000 {
                engine.tick(0.01);
            }
            let report = engine.profile_report().expect("profile available");
            criterion::black_box((report.ticks_per_second, report.block_evals_per_second));
        });
    });
}

fn bench_batch_sweep(c: &mut Criterion) {
    let mut graph = SimGraph::new();
    let src = graph.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let gain = graph.add_block("Heater", Box::new(Gain), &["I1"], &["Q"], &["Factor"]);
    graph
        .add_wire(
            graph.find_connector(src, "Q").unwrap(),
            graph.find_connector(gain, "I1").unwrap(),
        )
        .unwrap();
    let runner = BatchRunner::new(graph);
    let config = BatchConfig {
        param_sweeps: vec![ParamSweep {
            block_name: "Heater".to_string(),
            param: "Factor".to_string(),
            values: (0..=40).map(|value| value as f64).collect(),
        }],
        input_timeline: vec![
            (0.0, "Src".to_string(), 0.0),
            (10.0, "Src".to_string(), 40.0),
        ],
        duration_s: 100.0,
        dt: 0.1,
        outputs_to_track: vec!["Heater".to_string()],
    };

    c.bench_with_input(
        BenchmarkId::new("bench_batch_sweep", config.param_sweeps[0].values.len()),
        &config,
        |b, config| {
            b.iter(|| {
                let results = runner.run(config);
                criterion::black_box(results.len());
            });
        },
    );
}

criterion_group!(sim_bench, bench_tick_performance, bench_batch_sweep);
criterion_main!(sim_bench);
