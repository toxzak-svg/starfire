use starfire_stlm_l1d_preflight::identity_genome::{
    authority_boundary as identity_authority_boundary, IdentityClaim, IdentityClaimType,
    IdentityGenome, IdentityPersistence, IdentityQuery,
};
use starfire_stlm_l1d_preflight::phrase_critic::{
    authority_boundary as critic_authority_boundary, PhraseCritic, PhraseCriticCandidate,
    PhraseCriticContext, PhraseCriticModel, PHRASE_CRITIC_CONTEXT_SIZE,
    PHRASE_CRITIC_SCHEMA_VERSION, PHRASE_CRITIC_VOCABULARY_SIZE,
};
use std::collections::BTreeSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let critic = PhraseCritic::new(toy_model())?;
    let candidates = vec![
        PhraseCriticCandidate {
            candidate_id: 1,
            text: "The evidence supports that conclusion.".to_string(),
            semantic_verified: true,
            slots_preserved: true,
            identity_conflicts: 0,
            rule_score: 100,
        },
        PhraseCriticCandidate {
            candidate_id: 2,
            text: "The evidence supports that conclusion!".to_string(),
            semantic_verified: true,
            slots_preserved: true,
            identity_conflicts: 0,
            rule_score: 90,
        },
        PhraseCriticCandidate {
            candidate_id: 3,
            text: "A different unsupported conclusion!".to_string(),
            semantic_verified: false,
            slots_preserved: false,
            identity_conflicts: 0,
            rule_score: 10_000,
        },
    ];
    let context = PhraseCriticContext::default();
    let selection_a = critic.select(&context, &candidates)?;
    let selection_b = critic.select(&context, &candidates)?;

    let mut genome = IdentityGenome::default();
    genome.insert_claim(identity_claim(
        "value-evidential-honesty",
        IdentityClaimType::Value,
        8_700,
        8_800,
        &["technical", "uncertainty", "truth"],
    ))?;
    genome.insert_claim(identity_claim(
        "tendency-direct-explanation",
        IdentityClaimType::BehavioralTendency,
        8_100,
        7_900,
        &["technical", "direct"],
    ))?;
    genome.insert_claim(identity_claim(
        "relationship-shared-project",
        IdentityClaimType::RelationshipFact,
        8_000,
        9_000,
        &["relationship", "project"],
    ))?;
    let identity_query = IdentityQuery {
        tags: string_set(&["technical", "truth"]),
        max_claims: 2,
    };
    let identity_slice_a = genome.retrieve_slice(&identity_query)?;
    let identity_slice_b = genome.retrieve_slice(&identity_query)?;

    let critic_boundary = critic_authority_boundary();
    let identity_boundary = identity_authority_boundary();
    let report = serde_json::json!({
        "experiment": "stlm-l1d-phrase-critic-identity-genome-preflight-v1",
        "critic_exact_replay": selection_a == selection_b,
        "critic_selected_candidate_id": selection_a.selected_candidate_id,
        "critic_candidates_scored": selection_a.complete_candidates_scored,
        "critic_candidates_rejected_by_hard_gate": selection_a.candidates_rejected_by_hard_gate,
        "semantic_drift_candidate_rejected": selection_a.selected_candidate_id != 3,
        "identity_slice_exact_replay": identity_slice_a == identity_slice_b,
        "identity_slice_claim_ids": identity_slice_a.claim_ids,
        "critic_authority_closed": !critic_boundary.hard_semantic_gate_override
            && !critic_boundary.identity_conflict_override
            && !critic_boundary.selected_text_return
            && !critic_boundary.runtime_chat_influence
            && !critic_boundary.http_response_influence,
        "identity_authority_closed": !identity_boundary.automatic_invariant_promotion
            && !identity_boundary.automatic_belief_promotion
            && !identity_boundary.automatic_ontology_promotion
            && !identity_boundary.runtime_chat_influence,
        "candidate_text_persisted": false,
        "live_text_influence": false,
        "gate_passed": selection_a == selection_b
            && selection_a.selected_candidate_id == 2
            && selection_a.candidates_rejected_by_hard_gate == 1
            && identity_slice_a == identity_slice_b
            && !critic_boundary.runtime_chat_influence
            && !identity_boundary.runtime_chat_influence,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    if report["gate_passed"] != serde_json::Value::Bool(true) {
        return Err("STLM L1-D preflight failed".into());
    }
    Ok(())
}

fn toy_model() -> PhraseCriticModel {
    let hidden_size = 2;
    let mut embeddings = vec![vec![0.0; hidden_size]; PHRASE_CRITIC_VOCABULARY_SIZE];
    embeddings[usize::from(b'!')][0] = 2.0;
    embeddings[usize::from(b'.')][0] = -2.0;
    PhraseCriticModel {
        schema_version: PHRASE_CRITIC_SCHEMA_VERSION,
        vocabulary_size: PHRASE_CRITIC_VOCABULARY_SIZE,
        hidden_size,
        context_size: PHRASE_CRITIC_CONTEXT_SIZE,
        embeddings,
        recurrent_weights: vec![vec![0.0; hidden_size]; hidden_size],
        context_weights: vec![vec![0.0; hidden_size]; PHRASE_CRITIC_CONTEXT_SIZE],
        hidden_bias: vec![0.0; hidden_size],
        output_weights: vec![2.0, 0.0],
        output_bias: 0.0,
    }
}

fn identity_claim(
    id: &str,
    claim_type: IdentityClaimType,
    confidence_bps: u16,
    expression_weight_bps: u16,
    tags: &[&str],
) -> IdentityClaim {
    IdentityClaim {
        id: id.to_string(),
        claim_type,
        statement: format!("fixture statement for {id}"),
        confidence_bps,
        provenance: "stlm-l1d-preflight-fixture".to_string(),
        evidence_refs: Vec::new(),
        contradiction_refs: Vec::new(),
        persistence: IdentityPersistence::Revisable,
        expression_weight_bps,
        tags: string_set(tags),
        quarantined: false,
    }
}

fn string_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
