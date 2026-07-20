use anyhow::{ensure, Result};
use serde::Serialize;
use star::omega_g2_shadow::{OmegaG2ShadowSnapshot, ShadowAuthorityBoundary};
use star::prediction::{
    ConversationContext, Evidence, PredictionCenter, PredictionFilter, PredictionOutcome,
    PredictionStatus,
};

#[derive(Debug, Serialize)]
struct ProbeReport {
    generated_predictions: usize,
    live_confirmed_predictions: usize,
    shadow: OmegaG2ShadowSnapshot,
    authority_exact: bool,
    terminal_classification: &'static str,
    claim_boundary: &'static str,
}

fn main() -> Result<()> {
    let mut center = PredictionCenter::new();
    let mut context = ConversationContext::new(
        "typed-shadow-probe".to_string(),
        1,
        Some(vec![0.1, 0.2, 0.3, 0.4]),
        Some(0.5),
    );
    context.discussed_entities = vec![
        "prediction".to_string(),
        "evidence".to_string(),
        "composition".to_string(),
    ];

    let generated = center.generate(&context);
    ensure!(
        !generated.is_empty(),
        "prediction center produced no traceable predictions"
    );

    let evidence = Evidence {
        outcome: PredictionOutcome::Confirmed,
        prediction_id: generated[0].id,
    };
    center.update_with_evidence(&evidence);

    let live_confirmed = center
        .query(PredictionFilter {
            status: Some(PredictionStatus::Confirmed),
            ..PredictionFilter::default()
        })
        .len();
    ensure!(live_confirmed == 1, "live evidence transition changed");

    let snapshot = center.omega_g2_shadow_snapshot();
    ensure!(snapshot.observed_batches == 1, "shadow batch was not observed");
    ensure!(snapshot.settled_witnesses == 1, "shadow evidence was not settled");
    ensure!(
        snapshot.pending_traces == generated.len().saturating_sub(1),
        "shadow pending count mismatch"
    );
    ensure!(
        snapshot.authority == ShadowAuthorityBoundary::default(),
        "shadow authority boundary opened"
    );

    let report = ProbeReport {
        generated_predictions: generated.len(),
        live_confirmed_predictions: live_confirmed,
        shadow: snapshot,
        authority_exact: true,
        terminal_classification: "PASS",
        claim_boundary: "Feature-gated, in-memory observation of typed prediction rankings and independently settled outcomes only; no prediction, response, routing, persistence, promotion, tool, external-effect, or autonomous-action authority.",
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
