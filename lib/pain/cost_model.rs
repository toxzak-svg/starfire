//! Pain cost model — expected pain as a cost term in reasoning strategy selection

use super::{PainSource, ReasoningPattern};

/// Cost model for reasoning strategies
#[derive(Debug, Clone)]
pub struct PainCostModel {
    /// Base cost per pattern type
    pattern_costs: [(ReasoningPattern, f64); 6],
    /// Cost multiplier per pain source
    source_multipliers: [(PainSource, f64); 5],
    /// Historical pain decay rate
    decay_rate: f64,
}

impl PainCostModel {
    pub fn new() -> Self {
        Self {
            pattern_costs: [
                (ReasoningPattern::RepeatedDerivation, 0.1),
                (ReasoningPattern::ExcessiveChaining, 0.15),
                (ReasoningPattern::WrongAbstraction, 0.2),
                (ReasoningPattern::FailedCausalSearch, 0.25),
                (ReasoningPattern::FailedAnalogy, 0.2),
                (ReasoningPattern::AbandonedHypothesis, 0.15),
            ],
            source_multipliers: [
                (PainSource::RedundantComputation, 1.0),
                (PainSource::Contradiction, 1.5),
                (PainSource::WastedEffort, 0.8),
                (PainSource::ConceptFailure, 2.0),
                (PainSource::PoorStrategy, 1.2),
            ],
            decay_rate: 0.95,
        }
    }

    /// Compute expected pain cost for a potential reasoning path
    pub fn expected_pain_cost(
        &self,
        pattern: ReasoningPattern,
        source: PainSource,
        historical_pain: f64,
    ) -> f64 {
        let pattern_cost = self.pattern_costs.iter()
            .find(|(p, _)| *p == pattern)
            .map(|(_, c)| *c)
            .unwrap_or(0.1);

        let source_mult = self.source_multipliers.iter()
            .find(|(s, _)| *s == source)
            .map(|(_, m)| *m)
            .unwrap_or(1.0);

        pattern_cost * source_mult * (1.0 + historical_pain * self.decay_rate)
    }

    /// Get the pain-adjusted score for a strategy
    /// Lower pain = higher score
    pub fn adjust_strategy_score(
        &self,
        base_score: f64,
        expected_pain: f64,
    ) -> f64 {
        base_score * (1.0 - expected_pain * 0.5)
    }

    /// Should we avoid a concept based on its pain history?
    pub fn should_avoid_concept(&self, concept_pain: f64) -> bool {
        concept_pain > 0.7
    }

    /// Select the lowest-pain strategy from candidates
    pub fn select_lowest_pain<'a>(
        &self,
        candidates: &[(&'a str, f64, f64)],
    ) -> Option<&'a str>
    where
        f64: PartialOrd,
    {
        let mut best: Option<(&'a str, f64)> = None;

        for (name, base_score, pain) in candidates {
            let adjusted = self.adjust_strategy_score(*base_score, *pain);
            match best {
                None => best = Some((name, adjusted)),
                Some((_, current_adjusted)) if adjusted > current_adjusted => {
                    best = Some((name, adjusted));
                }
                _ => {}
            }
        }

        best.map(|(name, _)| name)
    }
}

impl Default for PainCostModel {
    fn default() -> Self {
        Self::new()
    }
}