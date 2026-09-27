use super::*;

fn isolated(dt: f32, tau: f32, exact: bool) -> Brain {
    let mut graph = b"FLYGON01".to_vec();
    for word in [1u32, 0, 0, 0] {
        graph.extend(word.to_le_bytes());
    }
    let metadata = serde_json::json!({
        "dataset":"synthetic passive-cell test", "contacts":0,
        "cells":[{"id":"test", "kind":"test", "class":"test", "side":"",
            "transmitter":"acetylcholine", "position":null}]
    });
    let mut config: Config =
        serde_json::from_str(include_str!("../../../modpacks/flygon/operant.json")).unwrap();
    config.operant = None;
    config.learning = false;
    config.dt_ms = dt;
    config.tau_synapse_ms = tau;
    config.exponential_current_integration = exact;
    let mut brain = Brain::new(
        &graph,
        &metadata.to_string(),
        &serde_json::to_string(&config).unwrap(),
    )
    .unwrap();
    brain.inject_currents("[[0,0.4]]").unwrap();
    brain.voltage[0] += 0.2;
    brain.current[0] = 1.0;
    brain.adaptation[0] = 0.3;
    brain
}

#[test]
fn passive_response_matches_analytic_solution_across_steps_and_equal_time_constants() {
    for tau in [5.0, 20.0, 20.000002] {
        for dt in [0.1, 0.5, 1.0] {
            let mut brain = isolated(dt, tau, true);
            brain.advance(10.0).unwrap();
            let leak = (-10.0f64 / 20.0).exp();
            let coupling = |tc: f64| {
                if tc == 20.0 {
                    0.5 * leak
                } else {
                    tc / (20.0 - tc) * (leak - (-10.0 / tc).exp())
                }
            };
            let expected = -52.0 + 0.2 * leak + coupling(tau as f64) + 0.4 * (1.0 - leak)
                - 0.3 * coupling(200.0);
            assert!(
                (brain.voltage[0] as f64 - expected).abs() < 0.00015,
                "dt={dt}, tau={tau}: {} versus {expected}",
                brain.voltage[0]
            );
            assert_eq!(brain.counts[0], 0);
        }
    }
    let mut exact = isolated(1.0, 5.0, true);
    let mut legacy = isolated(1.0, 5.0, false);
    exact.advance(10.0).unwrap();
    legacy.advance(10.0).unwrap();
    assert!(
        exact.voltage[0] - legacy.voltage[0] > 0.01,
        "the endpoint approximation undercounts a decaying excitatory current"
    );
}

#[test]
fn integration_mode_cannot_silently_change_live_or_on_restore() {
    let mut exact = isolated(0.5, 5.0, true);
    let legacy = isolated(0.5, 5.0, false);
    let before = exact.checkpoint().unwrap();
    assert!(exact.configure(&legacy.configuration().unwrap()).is_err());
    assert!(exact.restore(&legacy.checkpoint().unwrap()).is_err());
    assert_eq!(before, exact.checkpoint().unwrap());
}

#[test]
fn imported_clock_near_overflow_is_rejected_before_mutating_dynamics() {
    let mut brain = isolated(0.5, 5.0, true);
    let mut checkpoint: serde_json::Value = serde_json::from_str(&brain.checkpoint().unwrap()).unwrap();
    checkpoint["tick"] = serde_json::json!(u64::MAX - 1);
    brain.restore(&checkpoint.to_string()).unwrap();
    let before = brain.checkpoint().unwrap();
    assert!(brain.advance(10.0).unwrap_err().contains("clock limit"));
    assert_eq!(before, brain.checkpoint().unwrap());
}
