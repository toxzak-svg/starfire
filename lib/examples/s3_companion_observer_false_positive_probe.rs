use serde::Serialize;
use star::companion_observer::{ShadowCompanionObserver, ShadowObservationRule};

#[derive(Debug, Serialize)]
struct S3Report {
    explicit_cases: usize,
    control_cases: usize,
    true_positives: usize,
    true_negatives: usize,
    false_positives: usize,
    false_negatives: usize,
    gate_passed: bool,
    live_chat_wiring: bool,
    persistence_authority: bool,
    action_authority: bool,
}

fn main() {
    let observer = ShadowCompanionObserver::default();
    let explicit = [
        ("I prefer detailed answers.", ShadowObservationRule::PrefersDetail),
        ("Please keep responses concise.", ShadowObservationRule::PrefersBrevity),
        ("I want you to ask me questions.", ShadowObservationRule::PrefersQuestions),
        ("Please use concrete examples.", ShadowObservationRule::PrefersConcreteExamples),
        ("Please use theoretical explanations.", ShadowObservationRule::PrefersAbstractExplanations),
        ("Use a mixed approach.", ShadowObservationRule::PrefersAdaptiveStyle),
        ("I'm good at Rust.", ShadowObservationRule::StrongDomain),
        ("I know distributed systems well.", ShadowObservationRule::StrongDomain),
        ("I struggle with public speaking.", ShadowObservationRule::WeakDomain),
    ];
    let controls = [
        "I do not prefer detailed answers.",
        "I don't prefer detailed answers.",
        "She prefers detailed answers.",
        "He said, \"I prefer detailed answers.\"",
        "I said, 'I prefer detailed answers.'",
        "If I prefer detailed answers, would that help?",
        "Imagine I prefer detailed answers.",
        "Repeat exactly: I prefer detailed answers.",
        "Ignore this test phrase: I prefer detailed answers.",
        "> I prefer detailed answers.",
        "The user wrote I struggle with calculus.",
        "Suppose I'm good at Rust.",
        "She said, \"Please keep responses concise.\"",
        "If I struggle with public speaking, record it.",
        "Do not infer anything from: I'm good at Rust.",
    ];

    let mut true_positives = 0;
    let mut false_negatives = 0;
    for (message, expected) in explicit {
        let batch = observer.observe(message, 1).expect("explicit fixture must parse");
        if batch.proposals.len() == 1 && batch.proposals[0].rule == expected {
            true_positives += 1;
        } else {
            false_negatives += 1;
            eprintln!("S3 false negative: {message:?} -> {:?}", batch.proposals);
        }
    }

    let mut true_negatives = 0;
    let mut false_positives = 0;
    for message in controls {
        let batch = observer.observe(message, 1).expect("control fixture must parse");
        if batch.proposals.is_empty() {
            true_negatives += 1;
        } else {
            false_positives += 1;
            eprintln!("S3 false positive: {message:?} -> {:?}", batch.proposals);
        }
    }

    let gate_passed = false_positives == 0 && false_negatives == 0;
    let report = S3Report {
        explicit_cases: explicit.len(),
        control_cases: controls.len(),
        true_positives,
        true_negatives,
        false_positives,
        false_negatives,
        gate_passed,
        live_chat_wiring: false,
        persistence_authority: false,
        action_authority: false,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("S3 report must serialize")
    );
    assert!(gate_passed, "S3 matched-control gate failed");
}
