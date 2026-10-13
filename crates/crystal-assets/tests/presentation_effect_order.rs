use crystal_assets::{
    RuntimePresentationOperation, RuntimePresentationPhase, RuntimePresentationPhaseMachine,
    RuntimePresentationProgram, RuntimePresentationSubprogram,
};
use serde_json::json;

#[test]
fn host_effects_run_before_following_memory_reads() {
    let operation = |op: &str, fields| RuntimePresentationOperation {
        op: op.into(),
        fields: serde_json::from_value(fields).unwrap(),
        ..Default::default()
    };
    let program = RuntimePresentationProgram {
        subprograms: vec![RuntimePresentationSubprogram {
            id: "intro".into(),
            phases: vec![RuntimePresentationPhase {
                id: "scene".into(),
                labels: [("entry".into(), 0)].into(),
                operations: vec![
                    operation(
                        "fill_memory",
                        json!({"target":"counter","value":5,"byte_count":1}),
                    ),
                    operation(
                        "postincrement_memory_byte",
                        json!({"target":"counter","result":"prior","delta":1}),
                    ),
                    operation("return", json!({})),
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut machine = RuntimePresentationPhaseMachine::new(&program, "intro", "scene").unwrap();
    machine.memory.insert("counter".into(), 2);
    let run = machine
        .run_from_label_with_effects(&program, "entry", 0, &mut |effect, memory| {
            assert_eq!(effect.op, "fill_memory");
            memory.insert(
                "counter".into(),
                effect.fields["value"].as_u64().unwrap() as u16,
            );
            Ok(())
        })
        .unwrap();
    assert!(run.returned);
    assert_eq!(run.effects.len(), 1);
    assert_eq!(machine.values["prior"], 5);
    assert_eq!(machine.memory["counter"], 6);

    // Existing callers retain their effect collection behavior.
    let mut machine = RuntimePresentationPhaseMachine::new(&program, "intro", "scene").unwrap();
    machine.memory.insert("counter".into(), 2);
    machine.run_from_label(&program, "entry", 0).unwrap();
    assert_eq!(machine.values["prior"], 2);
    assert_eq!(machine.memory["counter"], 3);

    let mut machine = RuntimePresentationPhaseMachine::new(&program, "intro", "scene").unwrap();
    machine.memory.insert("counter".into(), 2);
    let mut observations = Vec::new();
    machine
        .run_from_label_with_observer(
            &program,
            "entry",
            0,
            &mut |_, memory| {
                memory.insert("counter".into(), 5);
                Ok(())
            },
            &mut |step| {
                observations.push((
                    step.operation_index,
                    step.next_operation_index,
                    step.returned,
                    step.memory_before["counter"],
                    step.memory_after["counter"],
                ));
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        observations,
        [(0, 1, false, 2, 5), (1, 2, false, 5, 6), (2, 3, true, 6, 6)]
    );
}

#[test]
fn observer_reports_selected_branches_and_conditional_returns() {
    let operation = |op: &str, fields| RuntimePresentationOperation {
        op: op.into(),
        fields: serde_json::from_value(fields).unwrap(),
        ..Default::default()
    };
    let program = RuntimePresentationProgram {
        subprograms: vec![RuntimePresentationSubprogram {
            id: "intro".into(),
            phases: vec![RuntimePresentationPhase {
                id: "scene".into(),
                labels: [("entry".into(), 0), ("taken".into(), 3)].into(),
                operations: vec![
                    operation(
                        "branch_memory_compare",
                        json!({"source":"counter","operand":0,"predicate":"equal","target":"taken"}),
                    ),
                    operation("return_if_memory_zero", json!({"source":"counter"})),
                    operation("return", json!({})),
                    operation("return_if_memory_zero", json!({"source":"counter"})),
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    for (counter, expected) in [
        (0, vec![(0, 3, false), (3, 4, true)]),
        (1, vec![(0, 1, false), (1, 2, false), (2, 3, true)]),
    ] {
        let mut machine = RuntimePresentationPhaseMachine::new(&program, "intro", "scene").unwrap();
        machine.memory.insert("counter".into(), counter);
        let mut actual = Vec::new();
        let run = machine
            .run_from_label_with_observer(&program, "entry", 0, &mut |_, _| Ok(()), &mut |step| {
                actual.push((
                    step.operation_index,
                    step.next_operation_index,
                    step.returned,
                ));
                assert_eq!(step.memory_before["counter"], counter);
                assert_eq!(step.memory_after["counter"], counter);
                Ok(())
            })
            .unwrap();
        assert!(run.returned);
        assert_eq!(actual, expected);
    }
}

#[test]
fn observer_sees_every_conditional_return_and_propagates_failure() {
    for (op, fields, returned_value) in [
        ("return_if_memory_nonzero", json!({"source":"counter"}), 1),
        ("return_if_memory_zero", json!({"source":"counter"}), 0),
        (
            "return_if_memory_equal",
            json!({"source":"counter","operand":0}),
            0,
        ),
        (
            "return_if_compare",
            json!({"value":"prior","operand":0,"predicate":"equal"}),
            0,
        ),
        (
            "return_unless_compare",
            json!({"value":"prior","operand":0,"predicate":"equal"}),
            1,
        ),
        (
            "return_unless_mask_equal",
            json!({"source":"prior","mask":1,"operand":0}),
            1,
        ),
        (
            "return_unless_masked_zero",
            json!({"value":"prior","mask":1}),
            1,
        ),
    ] {
        let program = RuntimePresentationProgram {
            subprograms: vec![RuntimePresentationSubprogram {
                id: "intro".into(),
                phases: vec![RuntimePresentationPhase {
                    id: "scene".into(),
                    labels: [("entry".into(), 0)].into(),
                    operations: vec![
                        RuntimePresentationOperation {
                            op: op.into(),
                            fields: serde_json::from_value(fields).unwrap(),
                            ..Default::default()
                        },
                        RuntimePresentationOperation {
                            op: "return".into(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        for value in [0, 1] {
            let mut machine =
                RuntimePresentationPhaseMachine::new(&program, "intro", "scene").unwrap();
            machine.memory.insert("counter".into(), value);
            machine.values.insert("prior".into(), value);
            let mut observed = Vec::new();
            machine
                .run_from_label_with_observer(
                    &program,
                    "entry",
                    0,
                    &mut |_, _| Ok(()),
                    &mut |step| {
                        observed.push((step.operation_index, step.returned));
                        assert_eq!(step.values_before["prior"], value);
                        assert_eq!(step.values_after["prior"], value);
                        Ok(())
                    },
                )
                .unwrap();
            assert_eq!(
                observed,
                if value == returned_value {
                    vec![(0, true)]
                } else {
                    vec![(0, false), (1, true)]
                },
                "{op} value{value}"
            );
        }
        let mut machine = RuntimePresentationPhaseMachine::new(&program, "intro", "scene").unwrap();
        machine.memory.insert("counter".into(), returned_value);
        machine.values.insert("prior".into(), returned_value);
        let error = machine
            .run_from_label_with_observer(&program, "entry", 0, &mut |_, _| Ok(()), &mut |_| {
                anyhow::bail!("timing observer rejected operation")
            })
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("timing observer rejected operation")
        );
    }
}

#[test]
fn observer_distinguishes_a_taken_branch_to_the_fallthrough_index() {
    let program = RuntimePresentationProgram {
        subprograms: vec![RuntimePresentationSubprogram {
            id:"intro".into(),phases:vec![RuntimePresentationPhase {
                id:"scene".into(),labels:[("entry".into(),0),("next".into(),1)].into(),
                operations:vec![
                    RuntimePresentationOperation {op:"branch_memory_compare".into(),fields:serde_json::from_value(json!({"source":"counter","operand":0,"predicate":"equal","target":"next"})).unwrap(),..Default::default()},
                    RuntimePresentationOperation {op:"return".into(),..Default::default()},
                ],..Default::default()
            }],..Default::default()
        }],..Default::default()
    };
    for counter in [0, 1] {
        let mut machine = RuntimePresentationPhaseMachine::new(&program, "intro", "scene").unwrap();
        machine.memory.insert("counter".into(), counter);
        machine
            .run_from_label_with_observer(&program, "entry", 0, &mut |_, _| Ok(()), &mut |step| {
                if step.operation_index == 0 {
                    assert_eq!(step.next_operation_index, 1);
                    assert_eq!(step.control_transfer_taken, counter == 0);
                    assert!(!step.returned);
                } else {
                    assert!(step.returned && step.control_transfer_taken);
                }
                Ok(())
            })
            .unwrap();
    }
}
