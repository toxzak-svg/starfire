use anyhow::{ensure, Result};
use star::arise_edge::{
    authority_boundary, AriseEngine, AriseRequest, AriseTerminalClassification, ObligationId,
    SemanticObligation,
};

fn obligation(id: u16, dependencies: &[u16], semantic_key: &str, witness: &str) -> SemanticObligation {
    SemanticObligation {
        id: ObligationId(id),
        semantic_key: semantic_key.to_string(),
        dependencies: dependencies.iter().copied().map(ObligationId).collect(),
        witness: witness.to_string(),
    }
}

fn main() -> Result<()> {
    let request = AriseRequest {
        trace_id: 1,
        intent_label: "explanation".to_string(),
        terminal_obligations: vec![ObligationId(4)],
        initially_satisfied: Vec::new(),
        obligations: vec![
            obligation(1, &[], "air.rises", "Moist air rises"),
            obligation(2, &[1], "air.cools", "Rising air cools"),
            obligation(
                3,
                &[2],
                "vapor.condenses",
                "Cooling water vapor condenses into droplets",
            ),
            obligation(
                4,
                &[3],
                "rain.forms",
                "Growing droplets eventually fall as rain",
            ),
        ],
        prohibited_fragments: vec!["unsupported certainty".to_string()],
    };

    let trace = AriseEngine::default().execute(&request)?;
    let authority = authority_boundary();

    ensure!(
        trace.terminal_classification == AriseTerminalClassification::Pass,
        "ARISE terminal classification was not PASS"
    );
    ensure!(trace.final_residual == 0, "semantic residual was not discharged");
    ensure!(
        trace
            .accepted_spans
            .iter()
            .all(|span| span.residual_after < span.residual_before),
        "an accepted span failed to reduce semantic residual"
    );
    ensure!(
        !authority.generated_text_influence
            && !authority.persistence_authority
            && !authority.routing_authority
            && !authority.tool_selection_authority
            && !authority.charge_discharge_authority
            && !authority.autonomous_action_authority,
        "ARISE-A0 authority boundary opened unexpectedly"
    );

    println!("{}", serde_json::to_string_pretty(&trace)?);
    Ok(())
}
