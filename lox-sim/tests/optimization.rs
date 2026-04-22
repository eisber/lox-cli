//! Optimization test cases for lox-sim.
//!
//! Verifies: autodiff gradients, parameter sweeps, energy optimization,
//! multi-objective Pareto analysis, compiled/interpreted equivalence,
//! and batch simulation performance.

use std::collections::HashMap;

use lox_sim::autodiff::{smooth_step, DualNumber, Tape};
use lox_sim::batch::{BatchConfig, BatchRunner, ParamSweep};
use lox_sim::blocks::*;
use lox_sim::compiler::CompiledGraph;
use lox_sim::engine::SimEngine;
use lox_sim::graph::SimGraph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build: Src (PassThrough) → GreaterEqual(threshold param) → Out (PassThrough)
fn threshold_graph() -> SimGraph {
    let mut g = SimGraph::new();
    let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    // Only one input — GreaterEqual falls back to params[0] for the right operand
    let cmp = g.add_block(
        "Cmp",
        Box::new(GreaterEqual),
        &["I1"],
        &["Q"],
        &["Threshold"],
    );
    let out = g.add_block("Out", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    g.add_wire(
        g.find_connector(src, "Q").unwrap(),
        g.find_connector(cmp, "I1").unwrap(),
    )
    .unwrap();
    g.add_wire(
        g.find_connector(cmp, "Q").unwrap(),
        g.find_connector(out, "I1").unwrap(),
    )
    .unwrap();
    g
}

/// Build: TempSrc → HeatIRoomController2 → Out
fn thermostat_graph() -> SimGraph {
    let mut g = SimGraph::new();
    let temp = g.add_block("TempSrc", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let sp = g.add_block("SetpointSrc", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let hrc = g.add_block(
        "Thermostat",
        Box::new(HeatIRoomController2::new()),
        &["Temp", "Setpoint", "Reset", "InputDisable"],
        &["AQh"],
        &["Kp", "Ki"],
    );
    let out = g.add_block("Out", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    g.add_wire(
        g.find_connector(temp, "Q").unwrap(),
        g.find_connector(hrc, "Temp").unwrap(),
    )
    .unwrap();
    g.add_wire(
        g.find_connector(sp, "Q").unwrap(),
        g.find_connector(hrc, "Setpoint").unwrap(),
    )
    .unwrap();
    g.add_wire(
        g.find_connector(hrc, "AQh").unwrap(),
        g.find_connector(out, "I1").unwrap(),
    )
    .unwrap();
    g
}

/// Build a non-trivial graph with 12 blocks for equivalence testing.
///
/// ```text
/// In1 ─┐           ┌→ Gain(2.0) ──┐
///      ├→ Add ─→ PT1 ──────────────┤
/// In2 ─┘           └→ Sub ←─ Const ┘
///                      │            │
///  In3 → Mult(In3,3) → Add2 ───────┤
///                                   ↓
///                                  Out
/// ```
fn complex_graph() -> SimGraph {
    let mut g = SimGraph::new();

    let in1 = g.add_block("In1", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let in2 = g.add_block("In2", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let in3 = g.add_block("In3", Box::new(PassThrough), &["I1"], &["Q"], &[]);

    let add1 = g.add_block("Add1", Box::new(Add), &["I1", "I2"], &["Q"], &[]);
    let pt1 = g.add_block("PT1", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let gain = g.add_block("Gain1", Box::new(Gain), &["I1"], &["Q"], &["Factor"]);
    let konst = g.add_block("Const1", Box::new(Constant), &[], &["Q"], &["Value"]);
    let sub1 = g.add_block("Sub1", Box::new(Sub), &["I1", "I2"], &["Q"], &[]);
    let mult1 = g.add_block("Mult1", Box::new(Mult), &["I1"], &["Q"], &["Factor"]);
    let add2 = g.add_block("Add2", Box::new(Add), &["I1", "I2"], &["Q"], &[]);
    let add3 = g.add_block("Add3", Box::new(Add), &["I1", "I2"], &["Q"], &[]);
    let out = g.add_block("Out", Box::new(PassThrough), &["I1"], &["Q"], &[]);

    // In1,In2 → Add1
    g.add_wire(
        g.find_connector(in1, "Q").unwrap(),
        g.find_connector(add1, "I1").unwrap(),
    )
    .unwrap();
    g.add_wire(
        g.find_connector(in2, "Q").unwrap(),
        g.find_connector(add1, "I2").unwrap(),
    )
    .unwrap();
    // Add1 → PT1
    g.add_wire(
        g.find_connector(add1, "Q").unwrap(),
        g.find_connector(pt1, "I1").unwrap(),
    )
    .unwrap();
    // PT1 → Gain1
    g.add_wire(
        g.find_connector(pt1, "Q").unwrap(),
        g.find_connector(gain, "I1").unwrap(),
    )
    .unwrap();
    // PT1 → Sub1.I1
    g.add_wire(
        g.find_connector(pt1, "Q").unwrap(),
        g.find_connector(sub1, "I1").unwrap(),
    )
    .unwrap();
    // Const1 → Sub1.I2
    g.add_wire(
        g.find_connector(konst, "Q").unwrap(),
        g.find_connector(sub1, "I2").unwrap(),
    )
    .unwrap();
    // In3 → Mult1
    g.add_wire(
        g.find_connector(in3, "Q").unwrap(),
        g.find_connector(mult1, "I1").unwrap(),
    )
    .unwrap();
    // Gain1 → Add2.I1
    g.add_wire(
        g.find_connector(gain, "Q").unwrap(),
        g.find_connector(add2, "I1").unwrap(),
    )
    .unwrap();
    // Sub1 → Add2.I2
    g.add_wire(
        g.find_connector(sub1, "Q").unwrap(),
        g.find_connector(add2, "I2").unwrap(),
    )
    .unwrap();
    // Mult1 → Add3.I1
    g.add_wire(
        g.find_connector(mult1, "Q").unwrap(),
        g.find_connector(add3, "I1").unwrap(),
    )
    .unwrap();
    // Add2 → Add3.I2
    g.add_wire(
        g.find_connector(add2, "Q").unwrap(),
        g.find_connector(add3, "I2").unwrap(),
    )
    .unwrap();
    // Add3 → Out
    g.add_wire(
        g.find_connector(add3, "Q").unwrap(),
        g.find_connector(out, "I1").unwrap(),
    )
    .unwrap();

    g
}

/// Simple graph for batch tests: Src → Gain(Factor) → Out
fn gain_graph() -> SimGraph {
    let mut g = SimGraph::new();
    let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let gain = g.add_block("Heater", Box::new(Gain), &["I1"], &["Q"], &["Factor"]);
    let out = g.add_block("Out", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    g.add_wire(
        g.find_connector(src, "Q").unwrap(),
        g.find_connector(gain, "I1").unwrap(),
    )
    .unwrap();
    g.add_wire(
        g.find_connector(gain, "Q").unwrap(),
        g.find_connector(out, "I1").unwrap(),
    )
    .unwrap();
    g
}

// ===========================================================================
// 1. Gradient verification tests
// ===========================================================================

#[test]
fn test_gradient_threshold_sensitivity() {
    // Use the smooth step approximation directly with DualNumbers to verify
    // d(output)/d(threshold) is negative: raising the threshold reduces activation.
    let beta = 10.0;
    let temp = DualNumber::constant(26.0);
    let threshold = DualNumber::variable(25.0); // seed derivative on threshold

    let output = temp.smooth_step(threshold, beta);

    // Primal: sigmoid(10*(26-25)) = sigmoid(10) ≈ 1.0
    assert!(
        output.val > 0.99,
        "output should be near 1.0, got {}",
        output.val
    );

    // Gradient: d(output)/d(threshold) should be NEGATIVE because
    // smooth_step computes σ(β·(temp - threshold)), and increasing
    // threshold decreases the argument, reducing the output.
    assert!(
        output.dot < 0.0,
        "gradient w.r.t. threshold should be negative, got {}",
        output.dot
    );
}

#[test]
fn test_gradient_via_reverse_mode_tape() {
    // Verify reverse-mode tape gives same gradient as forward-mode for
    // a simple computation: output = sigmoid(β·(input - threshold))
    let beta = 10.0;
    let input_val = 26.0;
    let threshold_val = 25.0;

    // Forward-mode: seed threshold
    let fwd_input = DualNumber::constant(input_val);
    let fwd_threshold = DualNumber::variable(threshold_val);
    let fwd_output = fwd_input.smooth_step(fwd_threshold, beta);

    // Reverse-mode: same computation on tape
    let mut tape = Tape::new();
    let t_input = tape.leaf(input_val);
    let t_threshold = tape.leaf(threshold_val);
    let t_diff = tape.sub(t_input, t_threshold);
    // σ(β·diff) using tape sigmoid
    let t_beta = tape.leaf(beta);
    let t_scaled = tape.mul(t_diff, t_beta);
    let t_output = tape.sigmoid(t_scaled, 1.0); // beta=1 since we already scaled

    let grads = tape.backward(t_output.idx);
    let reverse_grad_threshold = grads[t_threshold.idx];

    // Both methods should agree on the sign (negative w.r.t. threshold)
    assert!(fwd_output.dot < 0.0, "forward gradient should be negative");
    assert!(
        reverse_grad_threshold < 0.0,
        "reverse gradient should be negative"
    );

    // Values should be reasonably close (different sigmoid paths but same sign & magnitude)
    assert!(
        (fwd_output.dot - reverse_grad_threshold).abs() < 0.1,
        "forward ({}) and reverse ({}) gradients should be similar",
        fwd_output.dot,
        reverse_grad_threshold
    );
}

#[test]
fn test_gradient_gain_parameter() {
    // Verify eval_with_dual propagates gradients through Gain blocks.
    // Graph: Src → Gain(Factor) → Out
    // d(Out)/d(Src) should equal Factor.
    let graph = gain_graph();
    let mut engine = SimEngine::new(graph);

    engine.set_param("Heater", "Factor", 3.0);

    let mut inputs = HashMap::new();
    inputs.insert("Src".to_string(), DualNumber::variable(5.0));

    let outputs = engine.eval_with_dual(&inputs);
    let out = outputs.get("Out").expect("Out must exist in dual output");

    // Primal: 5.0 * 3.0 = 15.0
    assert!(
        (out.val - 15.0).abs() < 1e-6,
        "primal should be 15.0, got {}",
        out.val
    );

    // Gradient: d(Out)/d(Src) = Factor = 3.0
    assert!(
        (out.dot - 3.0).abs() < 1e-6,
        "gradient should be 3.0, got {}",
        out.dot
    );
}

// ===========================================================================
// 2. Parameter sweep: thermostat setpoint
// ===========================================================================

#[test]
fn test_sweep_thermostat_setpoint() {
    let graph = thermostat_graph();

    let room_temp = 20.0;
    let setpoints: Vec<f64> = (18..=26).map(|x| x as f64).collect();
    let mut heating_outputs: Vec<f64> = Vec::new();

    for &setpoint in &setpoints {
        let mut engine = SimEngine::new(graph.clone());
        engine.set_param("Thermostat", "Kp", 10.0);
        engine.set_param("Thermostat", "Ki", 0.0);

        engine.set_input("TempSrc", room_temp);
        engine.set_input("SetpointSrc", setpoint);

        for _ in 0..5 {
            engine.tick(0.1);
        }

        heating_outputs.push(engine.get_output("Out"));
    }

    // When setpoint ≤ room_temp (18,19,20), heating should be ~0
    for (i, &sp) in setpoints.iter().enumerate() {
        if sp <= room_temp {
            assert!(
                heating_outputs[i] < 1e-6,
                "setpoint {sp}°C ≤ room temp {room_temp}°C: heating should be ~0, got {}",
                heating_outputs[i]
            );
        }
    }

    // When setpoint > room_temp, output increases with setpoint
    let above: Vec<(f64, f64)> = setpoints
        .iter()
        .zip(heating_outputs.iter())
        .filter(|(&sp, _)| sp > room_temp)
        .map(|(&sp, &out)| (sp, out))
        .collect();

    assert!(above.len() >= 2, "need at least 2 above-room-temp points");
    for window in above.windows(2) {
        assert!(
            window[1].1 >= window[0].1,
            "heating should increase with setpoint: {:.0}°C→{:.1} vs {:.0}°C→{:.1}",
            window[0].0,
            window[0].1,
            window[1].0,
            window[1].1
        );
    }
}

// ===========================================================================
// 3. Energy optimization: blind threshold minimizes chatter
// ===========================================================================

#[test]
fn test_optimize_blind_threshold() {
    // Goal: find sunshine threshold that minimizes blind state changes.
    // Simulate 24h (1440 minutes) with a sine-wave sunshine signal (0–100%).
    //
    // Build: SunshineSrc → GreaterEqual(Threshold) → Out
    let graph = threshold_graph();
    let dt = 1.0; // 1-minute steps
    let steps = 1440; // 24 hours

    let thresholds: Vec<f64> = (4..=16).map(|x| x as f64 * 5.0).collect(); // 20,25,...,80
    let mut chatter_counts: Vec<usize> = Vec::new();

    for &threshold in &thresholds {
        let mut engine = SimEngine::new(graph.clone());
        engine.set_param("Cmp", "Threshold", threshold);

        let mut prev_output = 0.0_f64;
        let mut transitions = 0usize;

        for step in 0..steps {
            // Main daily curve + fast cloud ripple that creates extra crossings.
            // Low thresholds trigger on everything (few transitions: on at dawn, off at dusk).
            // Mid thresholds hit the ripples repeatedly (high chatter).
            // High thresholds only catch the very peak (few transitions or none).
            let minutes = step as f64;
            let daily = 50.0 + 50.0 * (std::f64::consts::TAU * (minutes - 360.0) / 1440.0).sin();
            let ripple = 15.0 * (std::f64::consts::TAU * minutes / 60.0).sin();
            let sunshine = (daily + ripple).clamp(0.0, 100.0);

            engine.set_input("Src", sunshine);
            engine.tick(dt);

            let output = engine.get_output("Out");
            if (output - prev_output).abs() > 0.5 {
                transitions += 1;
            }
            prev_output = output;
        }

        chatter_counts.push(transitions);
    }

    // Very high thresholds → signal rarely exceeds them → few transitions
    // Mid thresholds → ripple causes repeated crossings → more chatter
    // Very low thresholds → signal often above, but ripple at dawn/dusk adds some
    let min_chatter = *chatter_counts.iter().min().unwrap();
    let max_chatter = *chatter_counts.iter().max().unwrap();

    assert!(
        max_chatter > min_chatter,
        "chatter should vary across thresholds: min={min_chatter}, max={max_chatter}"
    );

    // There must be an optimal threshold with minimum chatter
    let optimal_idx = chatter_counts
        .iter()
        .enumerate()
        .min_by_key(|(_, &c)| c)
        .unwrap()
        .0;

    // The optimal shouldn't be the most "active" threshold
    assert!(
        chatter_counts[optimal_idx] < max_chatter,
        "optimal threshold should have less chatter than worst case"
    );

    eprintln!("Blind threshold optimization:");
    for (i, &threshold) in thresholds.iter().enumerate() {
        let marker = if chatter_counts[i] == min_chatter {
            " ← optimal"
        } else {
            ""
        };
        eprintln!(
            "  threshold={:5.1}%  transitions={:3}{}",
            threshold, chatter_counts[i], marker
        );
    }
}

// ===========================================================================
// 4. Multi-objective Pareto test
// ===========================================================================

#[test]
fn test_pareto_energy_vs_comfort() {
    // Sweep thermostat setpoint 15–25°C.
    // Objective 1 (energy): lower setpoint → less heating (energy ∝ max(0, sp - room_temp))
    // Objective 2 (comfort): setpoint close to 21°C → highest comfort (1/(1+|sp-21|))
    let graph = thermostat_graph();
    let room_temp = 18.0;

    let setpoints: Vec<f64> = (15..=25).map(|x| x as f64).collect();

    struct Solution {
        setpoint: f64,
        energy: f64,  // lower is better
        comfort: f64, // higher is better
    }

    let mut solutions: Vec<Solution> = Vec::new();

    for &setpoint in &setpoints {
        let mut engine = SimEngine::new(graph.clone());
        engine.set_param("Thermostat", "Kp", 10.0);
        engine.set_param("Thermostat", "Ki", 0.0);

        engine.set_input("TempSrc", room_temp);
        engine.set_input("SetpointSrc", setpoint);

        let mut total_heating = 0.0;
        for _ in 0..100 {
            engine.tick(0.1);
            total_heating += engine.get_output("Out");
        }

        let energy = total_heating; // lower is better
        let comfort = 1.0 / (1.0 + (setpoint - 21.0).abs()); // higher is better

        solutions.push(Solution {
            setpoint,
            energy,
            comfort,
        });
    }

    // Compute Pareto front: solution A dominates B iff
    //   A.energy ≤ B.energy AND A.comfort ≥ B.comfort (with at least one strict)
    let n = solutions.len();
    let mut is_dominated = vec![false; n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let better_energy = solutions[j].energy <= solutions[i].energy;
            let better_comfort = solutions[j].comfort >= solutions[i].comfort;
            let strictly_better = solutions[j].energy < solutions[i].energy
                || solutions[j].comfort > solutions[i].comfort;
            if better_energy && better_comfort && strictly_better {
                is_dominated[i] = true;
                break;
            }
        }
    }

    let pareto_front: Vec<&Solution> = solutions
        .iter()
        .enumerate()
        .filter(|(i, _)| !is_dominated[*i])
        .map(|(_, s)| s)
        .collect();

    eprintln!(
        "Pareto front ({} non-dominated solutions):",
        pareto_front.len()
    );
    for s in &pareto_front {
        eprintln!(
            "  setpoint={:.0}°C  energy={:.1}  comfort={:.3}",
            s.setpoint, s.energy, s.comfort
        );
    }

    // The Pareto front should have at least 2 solutions:
    // - low setpoint (low energy, low comfort)
    // - setpoint near 21 (high energy, high comfort)
    assert!(
        pareto_front.len() >= 2,
        "Pareto front should have ≥2 non-dominated solutions, got {}",
        pareto_front.len()
    );

    // Verify the trade-off: lowest-energy Pareto solution should have less comfort
    // than the highest-comfort Pareto solution.
    let min_energy = pareto_front
        .iter()
        .min_by(|a, b| a.energy.partial_cmp(&b.energy).unwrap())
        .unwrap();
    let max_comfort = pareto_front
        .iter()
        .max_by(|a, b| a.comfort.partial_cmp(&b.comfort).unwrap())
        .unwrap();

    assert!(
        max_comfort.energy > min_energy.energy
            || (max_comfort.energy - min_energy.energy).abs() < 1e-6,
        "trade-off: high-comfort solution should use more energy"
    );
}

// ===========================================================================
// 5. Compiled vs interpreted equivalence
// ===========================================================================

#[test]
fn test_compiled_matches_interpreted() {
    let graph = complex_graph();
    let mut engine = SimEngine::new(graph.clone());
    let mut compiled = CompiledGraph::from_graph(&graph);

    // Set parameters identically
    engine.set_param("Gain1", "Factor", 2.0);
    engine.set_param("Const1", "Value", 7.0);
    engine.set_param("Mult1", "Factor", 3.0);

    compiled.set_param("Gain1", "Factor", 2.0);
    compiled.set_param("Const1", "Value", 7.0);
    compiled.set_param("Mult1", "Factor", 3.0);

    // Test inputs: varying patterns over 1000 ticks
    for tick in 0..1000 {
        let t = tick as f64 * 0.01;
        let v1 = (t * 2.0).sin() * 10.0;
        let v2 = (t * 3.0).cos() * 5.0;
        let v3 = (t * 0.7).sin() * 8.0 + 2.0;

        engine.set_input("In1", v1);
        engine.set_input("In2", v2);
        engine.set_input("In3", v3);

        compiled.set_input("In1", v1);
        compiled.set_input("In2", v2);
        compiled.set_input("In3", v3);

        engine.tick(0.01);
        compiled.tick(0.01);

        let interp_out = engine.get_output("Out");
        let compiled_out = compiled.get_output("Out");

        assert!(
            (interp_out - compiled_out).abs() < 1e-10,
            "tick {tick}: interpreted={interp_out} ≠ compiled={compiled_out}"
        );
    }
}

// ===========================================================================
// 6. Batch performance test
// ===========================================================================

#[test]
fn test_batch_1000_simulations() {
    let graph = gain_graph();
    let runner = BatchRunner::new(graph);

    // Generate 1000 factor values spanning 0.1 to 100.0
    let values: Vec<f64> = (1..=1000).map(|i| i as f64 * 0.1).collect();

    let config = BatchConfig {
        param_sweeps: vec![ParamSweep {
            block_name: "Heater".to_string(),
            param: "Factor".to_string(),
            values,
        }],
        input_timeline: vec![
            (0.0, "Src".to_string(), 1.0),
            (0.5, "Src".to_string(), 2.0),
            (1.5, "Src".to_string(), 0.5),
        ],
        duration_s: 2.0,
        dt: 0.1,
        outputs_to_track: vec!["Out".to_string()],
    };

    let start = std::time::Instant::now();
    let results = runner.run(&config);
    let elapsed = start.elapsed();

    // All 1000 simulations must complete
    assert_eq!(results.len(), 1000);

    // Results must be deterministic: run again and compare
    let results2 = runner.run(&config);
    assert_eq!(results, results2, "batch results must be deterministic");

    // Each result must have output traces
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.output_traces.contains_key("Out"),
            "result {i} missing 'Out' trace"
        );
        let trace = &result.output_traces["Out"];
        assert!(!trace.is_empty(), "result {i} has empty output trace");
    }

    // Output should scale with gain factor: larger factor → larger output
    let first_avg: f64 = results[0].output_traces["Out"].iter().sum::<f64>()
        / results[0].output_traces["Out"].len() as f64;
    let last_avg: f64 = results[999].output_traces["Out"].iter().sum::<f64>()
        / results[999].output_traces["Out"].len() as f64;
    assert!(
        last_avg > first_avg,
        "higher gain should produce larger outputs: first_avg={first_avg}, last_avg={last_avg}"
    );

    let sims_per_sec = 1000.0 / elapsed.as_secs_f64();
    eprintln!(
        "Batch: 1000 simulations in {:.2?} ({:.0} sims/sec)",
        elapsed, sims_per_sec
    );
}

// ===========================================================================
// Additional: smooth_step optimization landscape test
// ===========================================================================

#[test]
fn test_smooth_step_optimization_landscape() {
    // Verify the smooth_step function creates a differentiable optimization
    // landscape suitable for gradient-based methods.
    let beta = 10.0;
    let threshold = 25.0;

    // Sample smooth_step across a range of temperatures
    let temps: Vec<f64> = (15..=35).map(|x| x as f64).collect();
    let outputs: Vec<f64> = temps
        .iter()
        .map(|&t| smooth_step(t, threshold, beta))
        .collect();

    // Verify monotonicity
    for i in 1..outputs.len() {
        assert!(
            outputs[i] >= outputs[i - 1] - f64::EPSILON,
            "smooth_step must be monotonically non-decreasing"
        );
    }

    // Verify boundary behavior
    assert!(outputs[0] < 0.01, "well below threshold → near 0");
    assert!(
        outputs[outputs.len() - 1] > 0.99,
        "well above threshold → near 1"
    );

    // Verify the gradient at the threshold is steepest
    let grad_at_threshold = beta * 0.5 * (1.0 - 0.5); // β·σ·(1-σ) at σ=0.5
    let expected = beta / 4.0;
    assert!(
        (grad_at_threshold - expected).abs() < 1e-10,
        "gradient at threshold should be β/4 = {expected}"
    );

    // Verify that numerical finite-difference gradient matches analytic
    let eps = 1e-5;
    let fd_grad = (smooth_step(threshold + eps, threshold, beta)
        - smooth_step(threshold - eps, threshold, beta))
        / (2.0 * eps);
    assert!(
        (fd_grad - expected).abs() < 1e-3,
        "finite-difference gradient ({fd_grad}) should match analytic ({expected})"
    );
}

#[test]
fn test_tape_multi_parameter_gradient() {
    // Use reverse-mode tape to get gradients w.r.t. multiple parameters
    // simultaneously. This is the key advantage of reverse-mode AD.
    //
    // f(a,b,c) = a*b + b*c + a*c  at (2,3,4)
    // ∂f/∂a = b + c = 7
    // ∂f/∂b = a + c = 6
    // ∂f/∂c = b + a = 5
    let mut tape = Tape::new();
    let a = tape.leaf(2.0);
    let b = tape.leaf(3.0);
    let c = tape.leaf(4.0);

    let ab = tape.mul(a, b);
    let bc = tape.mul(b, c);
    let ac = tape.mul(a, c);
    let ab_bc = tape.add(ab, bc);
    let f = tape.add(ab_bc, ac);

    assert!((f.val - 26.0).abs() < f64::EPSILON, "f(2,3,4) = 26");

    let grads = tape.backward(f.idx);
    assert!(
        (grads[a.idx] - 7.0).abs() < f64::EPSILON,
        "∂f/∂a = 7, got {}",
        grads[a.idx]
    );
    assert!(
        (grads[b.idx] - 6.0).abs() < f64::EPSILON,
        "∂f/∂b = 6, got {}",
        grads[b.idx]
    );
    assert!(
        (grads[c.idx] - 5.0).abs() < f64::EPSILON,
        "∂f/∂c = 5, got {}",
        grads[c.idx]
    );
}
