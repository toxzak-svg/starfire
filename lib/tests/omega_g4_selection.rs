use star::commitment_state::Atom;
use star::intervention_guided_abstraction_selection::{
    diagnostic_schema_prediction, SelectionSchemaKind, SelectionTransferTask, TransferCase,
};

fn atom(value: &str) -> Atom {
    Atom::new(value.to_string()).expect("valid atom")
}

fn task() -> SelectionTransferTask {
    let chain = vec![atom("x0"), atom("x1"), atom("x2"), atom("x3"), atom("x4")];
    let proxy = atom("proxy");
    let distractors = vec![atom("d0"), atom("d1")];
    SelectionTransferTask {
        root_id: 1,
        family: "integration".to_string(),
        chain,
        proxy,
        distractors,
        cases: Vec::<TransferCase>::new(),
    }
}

#[test]
fn proxy_break_preserves_recursive_chain() {
    let task = task();
    let mut events = task.chain.clone();
    events.extend(task.distractors.clone());
    events.push(task.proxy.clone());

    assert!(diagnostic_schema_prediction(
        SelectionSchemaKind::RecursiveAppendAdjacent,
        &task,
        &events,
    )
    .expect("recursive prediction"));
    assert!(!diagnostic_schema_prediction(
        SelectionSchemaKind::ProxyAnchorAdjacent,
        &task,
        &events,
    )
    .expect("proxy prediction"));
}

#[test]
fn chain_break_preserves_proxy_anchor() {
    let task = task();
    let mut events = vec![task.proxy.clone(), task.chain[0].clone()];
    events.push(task.chain[2].clone());
    events.push(task.chain[1].clone());
    events.extend(task.chain.iter().skip(3).cloned());
    events.extend(task.distractors.clone());

    assert!(!diagnostic_schema_prediction(
        SelectionSchemaKind::RecursiveAppendAdjacent,
        &task,
        &events,
    )
    .expect("recursive prediction"));
    assert!(diagnostic_schema_prediction(
        SelectionSchemaKind::ProxyAnchorAdjacent,
        &task,
        &events,
    )
    .expect("proxy prediction"));
}
