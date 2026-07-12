//! Task-aware verifier scoring for CHARGE diagnostics.
//!
//! This module keeps verifier semantics reusable instead of baking diagnostic
//! scoring into individual experiment examples.

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifierProfile {
    SurfaceCoverage,
    ProfileBlind,
    TaskProfiled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifierTaskClass {
    KnowledgeGap,
    PredictionContradiction,
    CausalMechanism,
}

pub fn score_resolution(
    output: &str,
    target: &str,
    class: VerifierTaskClass,
    profile: VerifierProfile,
) -> f64 {
    match profile {
        VerifierProfile::SurfaceCoverage => surface_resolution_score(output, target),
        VerifierProfile::ProfileBlind => profile_blind_resolution_score(output, target),
        VerifierProfile::TaskProfiled => task_profiled_resolution_score(output, target, class),
    }
}

/// Scores the same normalized output/target evidence as the task-profiled
/// verifier, but deliberately cannot see the task profile. It gives surface
/// semantic credit and penalizes unresolved question-shaped answers, but it
/// does not add contradiction-correction or causal-mechanism credit.
pub fn profile_blind_resolution_score(output: &str, target: &str) -> f64 {
    let answer_shape = if is_question_like(output) { 0.25 } else { 1.0 };
    (surface_resolution_score(output, target) * answer_shape).clamp(0.0, 1.0)
}

pub fn surface_resolution_score(output: &str, target: &str) -> f64 {
    let output_lower = output.to_ascii_lowercase();
    let target_lower = target.to_ascii_lowercase();
    if output_lower.contains(target_lower.trim_end_matches('.')) {
        return 1.0;
    }

    let output_tokens = token_set(output);
    let target_tokens = token_set(target);
    if output_tokens.is_empty() || target_tokens.is_empty() {
        return 0.0;
    }

    let overlap = output_tokens.intersection(&target_tokens).count() as f64;
    let precision = overlap / output_tokens.len() as f64;
    let recall = overlap / target_tokens.len() as f64;
    let mut score = if precision + recall <= f64::EPSILON {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    };

    let target_negative = contains_negation(&target_tokens);
    let output_negative = contains_negation(&output_tokens);
    if target_negative != output_negative {
        score *= 0.2;
    }

    score.clamp(0.0, 1.0)
}

pub fn task_profiled_resolution_score(output: &str, target: &str, class: VerifierTaskClass) -> f64 {
    let surface = surface_resolution_score(output, target);
    let output_tokens = token_set(output);
    let target_tokens = token_set(target);
    if output_tokens.is_empty() || target_tokens.is_empty() {
        return surface;
    }

    // Frozen H5 task-profiled contract:
    // - contradiction-correction earns reasoning credit for resolving the
    //   asserted correction with the target polarity, without requiring a
    //   causal mechanism exposition;
    // - causal-mechanism tasks earn causal credit for identifying a cause/effect
    //   mechanism, without requiring contradiction-correction behavior;
    // - task completion requires answer-shaped output, not restating a question;
    // - generic verbosity and unrelated term spraying are not rewarded beyond
    //   bounded semantic overlap;
    // - surface coverage remains a fallback signal, but cannot replace the
    //   task-specific evidence above when the task profile is visible.
    match class {
        VerifierTaskClass::KnowledgeGap => surface,
        VerifierTaskClass::PredictionContradiction => {
            let answer_shape = if is_question_like(output) { 0.25 } else { 1.0 };
            let negation_match =
                contains_negation(&output_tokens) == contains_negation(&target_tokens);
            let correction_overlap = output_tokens.intersection(&target_tokens).count() as f64
                / target_tokens.len() as f64;
            let contradiction_score = if negation_match {
                correction_overlap
            } else {
                correction_overlap * 0.2
            };
            (surface.max(contradiction_score) * answer_shape).clamp(0.0, 1.0)
        }
        VerifierTaskClass::CausalMechanism => {
            let mechanism_tokens = [
                "cause",
                "causes",
                "caused",
                "causing",
                "increase",
                "increased",
                "reduce",
                "reduced",
                "higher",
                "slower",
                "heat",
            ];
            let mechanism_signal = mechanism_tokens
                .iter()
                .filter(|token| output_tokens.contains(**token))
                .count() as f64
                / 2.0;
            let effect_overlap = output_tokens.intersection(&target_tokens).count() as f64
                / target_tokens.len() as f64;
            let causal_score =
                (0.70 * effect_overlap + 0.30 * mechanism_signal.min(1.0)).clamp(0.0, 1.0);
            surface.max(causal_score).clamp(0.0, 1.0)
        }
    }
}

fn token_set(text: &str) -> HashSet<String> {
    const STOPWORDS: [&str; 25] = [
        "a", "an", "and", "are", "as", "at", "be", "by", "do", "does", "for", "from", "in", "is",
        "it", "of", "on", "only", "the", "to", "used", "what", "which", "why", "with",
    ];

    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '\'')
        .map(|token| token.trim_matches('\'').to_ascii_lowercase())
        .filter(|token| token.len() > 1 && !STOPWORDS.contains(&token.as_str()))
        .collect()
}

fn contains_negation(tokens: &HashSet<String>) -> bool {
    ["not", "never", "no", "without", "false"]
        .iter()
        .any(|negation| tokens.contains(*negation))
}

fn is_question_like(text: &str) -> bool {
    let trimmed = text.trim_start().to_ascii_lowercase();
    text.contains('?')
        || trimmed.starts_with("what ")
        || trimmed.starts_with("why ")
        || trimmed.starts_with("how ")
        || trimmed.starts_with("does ")
        || trimmed.starts_with("is ")
        || trimmed.starts_with("are ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_profiled_contradiction_requires_matching_negation() {
        let target = "Copper is not ferromagnetic at room temperature.";
        let wrong = "Copper is ferromagnetic at room temperature.";
        let correct = "Copper is not ferromagnetic at room temperature.";

        let wrong_score = score_resolution(
            wrong,
            target,
            VerifierTaskClass::PredictionContradiction,
            VerifierProfile::TaskProfiled,
        );
        let correct_score = score_resolution(
            correct,
            target,
            VerifierTaskClass::PredictionContradiction,
            VerifierProfile::TaskProfiled,
        );

        assert!(wrong_score < 0.30, "wrong_score={wrong_score}");
        assert!(correct_score > 0.95, "correct_score={correct_score}");
    }

    #[test]
    fn task_profiled_contradiction_penalizes_unresolved_questions() {
        let target = "Sound travels faster in steel than in air.";
        let question = "Does sound travel faster in air than in steel?";
        let answer = "Sound travels faster in steel than in air.";

        let question_score = score_resolution(
            question,
            target,
            VerifierTaskClass::PredictionContradiction,
            VerifierProfile::TaskProfiled,
        );
        let answer_score = score_resolution(
            answer,
            target,
            VerifierTaskClass::PredictionContradiction,
            VerifierProfile::TaskProfiled,
        );

        assert!(question_score < 0.30, "question_score={question_score}");
        assert!(answer_score > 0.95, "answer_score={answer_score}");
    }

    #[test]
    fn task_profiled_causal_mechanism_rewards_mechanism_language() {
        let target = "Packet loss causes retransmission and increased latency.";
        let output = "Some packets fail to arrive, so retransmission causes increased latency.";

        let surface = score_resolution(
            output,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::SurfaceCoverage,
        );
        let profiled = score_resolution(
            output,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::TaskProfiled,
        );

        assert!(
            profiled >= surface + 0.05,
            "surface={surface} profiled={profiled}"
        );
        assert!(profiled >= 0.70, "profiled={profiled}");
    }

    #[test]
    fn contradiction_correction_does_not_require_causal_mechanism_exposition() {
        let target =
            "Earth's seasons are caused mainly by axial tilt, not changing distance from the Sun.";
        let output =
            "Earth's seasons are caused mainly by axial tilt, not changing distance from the Sun.";

        let contradiction_score = score_resolution(
            output,
            target,
            VerifierTaskClass::PredictionContradiction,
            VerifierProfile::TaskProfiled,
        );
        let causal_score = score_resolution(
            output,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::TaskProfiled,
        );

        assert!(
            contradiction_score >= 0.95,
            "contradiction_score={contradiction_score}"
        );
        assert!(
            contradiction_score >= causal_score,
            "contradiction_score={contradiction_score} causal_score={causal_score}"
        );
    }

    #[test]
    fn causal_mechanism_does_not_require_contradiction_correction_behavior() {
        let target = "Friction causes mechanical energy to become heat.";
        let output = "Friction causes mechanical energy to become heat.";

        let causal_score = score_resolution(
            output,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::TaskProfiled,
        );
        let contradiction_score = score_resolution(
            output,
            target,
            VerifierTaskClass::PredictionContradiction,
            VerifierProfile::TaskProfiled,
        );

        assert!(causal_score >= 0.95, "causal_score={causal_score}");
        assert!(
            causal_score >= contradiction_score,
            "causal_score={causal_score} contradiction_score={contradiction_score}"
        );
    }

    #[test]
    fn generic_verbosity_cannot_dominate_task_profiles() {
        let target = "Packet loss causes retransmission and increased latency.";
        let verbose = "Packet packet network latency retransmission causal mechanism contradiction correction resolver profile task output surface coverage relevant terms.";

        let contradiction_score = score_resolution(
            verbose,
            target,
            VerifierTaskClass::PredictionContradiction,
            VerifierProfile::TaskProfiled,
        );
        let causal_score = score_resolution(
            verbose,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::TaskProfiled,
        );

        assert!(
            contradiction_score < 0.70,
            "contradiction_score={contradiction_score}"
        );
        assert!(causal_score < 0.70, "causal_score={causal_score}");
    }

    #[test]
    fn wrong_profile_behavior_with_many_terms_does_not_get_positive_credit() {
        let target = "Copper is not ferromagnetic at room temperature.";
        let wrong = "Copper room temperature ferromagnetic magnetic metal correction mechanism cause therefore yes copper is ferromagnetic.";

        let score = score_resolution(
            wrong,
            target,
            VerifierTaskClass::PredictionContradiction,
            VerifierProfile::TaskProfiled,
        );

        assert!(score < 0.30, "score={score}");
    }

    #[test]
    fn profile_blind_scoring_does_not_reward_every_task_profile() {
        let target = "Packet loss causes retransmission and increased latency.";
        let output = "Does packet loss cause retransmission and latency?";

        let causal_score = score_resolution(
            output,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::TaskProfiled,
        );
        let blind_score = score_resolution(
            output,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::ProfileBlind,
        );

        assert!(causal_score >= 0.70, "causal_score={causal_score}");
        assert!(
            blind_score < causal_score - 0.10,
            "blind_score={blind_score} causal_score={causal_score}"
        );
    }

    #[test]
    fn superficial_formatting_does_not_change_task_profiled_score() {
        let target = "Combustion causes heat release.";
        let first = "Combustion causes heat release.";
        let second = "  combustion CAUSES heat release  ";

        let first_score = score_resolution(
            first,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::TaskProfiled,
        );
        let second_score = score_resolution(
            second,
            target,
            VerifierTaskClass::CausalMechanism,
            VerifierProfile::TaskProfiled,
        );

        assert_eq!(first_score, second_score);
    }

    #[test]
    fn task_profiled_knowledge_gap_keeps_surface_semantics() {
        let target = "DNS resolves domain names to IP addresses.";
        let output = "DNS resolves domain names to IP addresses.";

        assert_eq!(
            score_resolution(
                output,
                target,
                VerifierTaskClass::KnowledgeGap,
                VerifierProfile::TaskProfiled,
            ),
            score_resolution(
                output,
                target,
                VerifierTaskClass::KnowledgeGap,
                VerifierProfile::SurfaceCoverage,
            )
        );
    }
}
