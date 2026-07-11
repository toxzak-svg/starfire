//! Witnessed congruence splitting for unresolved CHARGE histories.
//!
//! The primitive in this module is diagnostic-only. It treats the current
//! behavioral partition of unresolved histories as a conjectured congruence and
//! allows an actually executed resolver continuation to refute that identity.
//! No CHARGE residual coordinate, kind, scope, hidden task class, target answer,
//! or resolver-leader label is accepted by the fitting API.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use super::judge::{ImprovementDirection, OutcomeWitness};

const SCORE_EPSILON: f64 = 1e-12;

#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WitnessBin {
    StrongWorse = -2,
    Worse = -1,
    Flat = 0,
    Better = 1,
    StrongBetter = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TerminalDisposition {
    Persisted,
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TerminalWitness {
    pub movement: WitnessBin,
    pub disposition: TerminalDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolverWord {
    pub first: u8,
    pub second: u8,
}

impl ResolverWord {
    pub fn reverse(self) -> Self {
        Self {
            first: self.second,
            second: self.first,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContinuationObservation {
    pub anchor_id: u64,
    pub word: ResolverWord,
    pub terminal: TerminalWitness,
    pub compute_cost: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CongruenceSplitConfig {
    pub witness_deadzone: f64,
    pub witness_strong_boundary: f64,
    pub proposal_budget: usize,
    pub max_signature_classes: usize,
    pub min_train_signature_support: usize,
    pub min_holdout_signature_support: usize,
    pub complexity_penalty: f64,
    pub min_train_defect_gain_after_penalty: f64,
    pub min_holdout_defect_gain: f64,
    pub max_holdout_defect_ratio: f64,
}

impl Default for CongruenceSplitConfig {
    fn default() -> Self {
        Self {
            witness_deadzone: 0.05,
            witness_strong_boundary: 0.25,
            proposal_budget: 20,
            max_signature_classes: 16,
            min_train_signature_support: 8,
            min_holdout_signature_support: 4,
            complexity_penalty: 0.02,
            min_train_defect_gain_after_penalty: 0.15,
            min_holdout_defect_gain: 0.10,
            max_holdout_defect_ratio: 0.75,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObserverSignature {
    pub parent_class: u16,
    pub terminal: TerminalWitness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessedCongruenceObserver {
    pub word: ResolverWord,
    pub parent_signature_classes: BTreeMap<Vec<TerminalWitness>, u16>,
    pub signature_classes: BTreeMap<ObserverSignature, u16>,
    pub parent_fallback_class: u16,
}

impl WitnessedCongruenceObserver {
    pub fn parent_class(&self, one_step_signature: &[TerminalWitness]) -> u16 {
        self.parent_signature_classes
            .get(one_step_signature)
            .copied()
            .unwrap_or(self.parent_fallback_class)
    }

    pub fn class(
        &self,
        one_step_signature: &[TerminalWitness],
        terminal: Option<TerminalWitness>,
    ) -> u16 {
        let parent_class = self.parent_class(one_step_signature);
        let Some(terminal) = terminal else {
            return parent_class;
        };
        self.signature_classes
            .get(&ObserverSignature {
                parent_class,
                terminal,
            })
            .copied()
            .unwrap_or(parent_class)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CongruenceSplitFit {
    pub candidate_word: Option<ResolverWord>,
    pub observer: Option<WitnessedCongruenceObserver>,
    pub training_parent_defect: f64,
    pub training_split_defect: f64,
    pub training_gain_after_penalty: f64,
    pub holdout_parent_defect: f64,
    pub holdout_split_defect: f64,
    pub holdout_defect_gain: f64,
    pub holdout_defect_ratio: f64,
    pub proposal_evaluations: usize,
}

impl CongruenceSplitFit {
    pub fn applied(&self) -> bool {
        self.observer.is_some()
    }
}

#[derive(Debug, Error)]
pub enum CongruenceSplitError {
    #[error("training continuation observations are empty")]
    EmptyTraining,
    #[error("holdout continuation observations are empty")]
    EmptyHoldout,
    #[error("training one-step signatures are empty")]
    EmptyTrainingSignatures,
    #[error("holdout one-step signatures are empty")]
    EmptyHoldoutSignatures,
    #[error("congruence split configuration is invalid")]
    InvalidConfig,
    #[error("continuation observation declares zero compute cost")]
    ZeroComputeCost,
    #[error("only non-repeating length-two resolver words are admissible")]
    InvalidResolverWord,
    #[error("candidate table contains only {available} unique words; exact proposal budget requires {required}")]
    InsufficientProposals { available: usize, required: usize },
}

pub fn directed_normalized_motion(witness: &OutcomeWitness) -> Option<f64> {
    if !witness.before.is_finite() || !witness.after.is_finite() {
        return None;
    }
    let raw = match witness.direction {
        ImprovementDirection::HigherIsBetter => witness.after - witness.before,
        ImprovementDirection::LowerIsBetter => witness.before - witness.after,
    };
    let scale = witness.before.abs().max(witness.after.abs()).max(1.0);
    Some((raw / scale).clamp(-1.0, 1.0))
}

pub fn quantize_terminal_witness(
    motion: f64,
    disposition: TerminalDisposition,
    config: CongruenceSplitConfig,
) -> Option<TerminalWitness> {
    if !motion.is_finite() {
        return None;
    }
    let movement = if motion < -config.witness_strong_boundary {
        WitnessBin::StrongWorse
    } else if motion < -config.witness_deadzone {
        WitnessBin::Worse
    } else if motion <= config.witness_deadzone {
        WitnessBin::Flat
    } else if motion <= config.witness_strong_boundary {
        WitnessBin::Better
    } else {
        WitnessBin::StrongBetter
    };
    Some(TerminalWitness {
        movement,
        disposition,
    })
}

pub fn congruence_defect(
    classes: &BTreeMap<u64, u16>,
    observations: &[ContinuationObservation],
    audit_words: &[ResolverWord],
) -> Option<f64> {
    let audit_words: BTreeSet<_> = audit_words.iter().copied().collect();
    let mut counts = BTreeMap::<(u16, ResolverWord, TerminalWitness), u64>::new();
    let mut totals = BTreeMap::<(u16, ResolverWord), u64>::new();

    for observation in observations {
        if !audit_words.contains(&observation.word) {
            continue;
        }
        let Some(class) = classes.get(&observation.anchor_id).copied() else {
            continue;
        };
        *counts
            .entry((class, observation.word, observation.terminal))
            .or_default() += 1;
        *totals.entry((class, observation.word)).or_default() += 1;
    }

    let mut mismatches = 0_u64;
    let mut comparable_pairs = 0_u64;
    for ((class, word), total) in totals {
        let total_pairs = choose_two(total);
        if total_pairs == 0 {
            continue;
        }
        let agreeing_pairs = counts
            .range(
                (class, word, min_terminal())..=(class, word, max_terminal()),
            )
            .map(|(_, count)| choose_two(*count))
            .sum::<u64>();
        mismatches = mismatches.saturating_add(total_pairs.saturating_sub(agreeing_pairs));
        comparable_pairs = comparable_pairs.saturating_add(total_pairs);
    }

    if comparable_pairs == 0 {
        None
    } else {
        Some(mismatches as f64 / comparable_pairs as f64)
    }
}

pub fn fit_witnessed_congruence_split(
    train: &[ContinuationObservation],
    train_one_step: &BTreeMap<u64, Vec<TerminalWitness>>,
    holdout: &[ContinuationObservation],
    holdout_one_step: &BTreeMap<u64, Vec<TerminalWitness>>,
    config: CongruenceSplitConfig,
) -> Result<CongruenceSplitFit, CongruenceSplitError> {
    validate(train, train_one_step, holdout, holdout_one_step, config)?;

    let words: Vec<_> = train
        .iter()
        .map(|observation| observation.word)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if words.len() < config.proposal_budget {
        return Err(CongruenceSplitError::InsufficientProposals {
            available: words.len(),
            required: config.proposal_budget,
        });
    }
    let proposal_words = &words[..config.proposal_budget];

    let (parent_signature_classes, train_parent_classes) =
        fit_parent_classes(train_one_step, config.max_signature_classes);
    let parent_fallback_class = 0_u16;
    let holdout_parent_classes = apply_parent_classes(
        holdout_one_step,
        &parent_signature_classes,
        parent_fallback_class,
    );

    let train_lookup = observation_lookup(train);
    let mut best: Option<Candidate> = None;

    for word in proposal_words.iter().copied() {
        let audit_words: Vec<_> = proposal_words
            .iter()
            .copied()
            .filter(|candidate| *candidate != word && *candidate != word.reverse())
            .collect();
        let Some(parent_defect) =
            congruence_defect(&train_parent_classes, train, &audit_words)
        else {
            continue;
        };
        let signature_classes = fit_supported_split_classes(
            &train_parent_classes,
            &train_lookup,
            word,
            parent_signature_classes.len() as u16,
            config.min_train_signature_support,
            config.max_signature_classes,
        );
        if !actually_splits_parent(&signature_classes) {
            continue;
        }
        let split_classes = apply_split_classes(
            train_one_step,
            &train_lookup,
            word,
            &parent_signature_classes,
            parent_fallback_class,
            &signature_classes,
        );
        let Some(split_defect) = congruence_defect(&split_classes, train, &audit_words) else {
            continue;
        };
        let gain_after_penalty = parent_defect - split_defect - config.complexity_penalty;
        let candidate = Candidate {
            word,
            parent_defect,
            split_defect,
            gain_after_penalty,
            signature_classes,
            audit_words,
        };
        if best
            .as_ref()
            .is_none_or(|current| candidate_is_better(&candidate, current))
        {
            best = Some(candidate);
        }
    }

    let proposal_evaluations = proposal_words.len();
    let Some(best) = best else {
        return Ok(empty_fit(proposal_evaluations));
    };

    if best.gain_after_penalty + SCORE_EPSILON
        < config.min_train_defect_gain_after_penalty
    {
        return Ok(CongruenceSplitFit {
            candidate_word: Some(best.word),
            observer: None,
            training_parent_defect: best.parent_defect,
            training_split_defect: best.split_defect,
            training_gain_after_penalty: best.gain_after_penalty,
            holdout_parent_defect: 0.0,
            holdout_split_defect: 0.0,
            holdout_defect_gain: 0.0,
            holdout_defect_ratio: 1.0,
            proposal_evaluations,
        });
    }

    let holdout_lookup = observation_lookup(holdout);
    let holdout_split_classes = apply_split_classes(
        holdout_one_step,
        &holdout_lookup,
        best.word,
        &parent_signature_classes,
        parent_fallback_class,
        &best.signature_classes,
    );
    let holdout_supported_children = supported_child_count(
        &holdout_parent_classes,
        &holdout_split_classes,
        config.min_holdout_signature_support,
    );
    let holdout_parent_defect = congruence_defect(
        &holdout_parent_classes,
        holdout,
        &best.audit_words,
    )
    .unwrap_or(0.0);
    let holdout_split_defect = congruence_defect(
        &holdout_split_classes,
        holdout,
        &best.audit_words,
    )
    .unwrap_or(holdout_parent_defect);
    let holdout_defect_gain = holdout_parent_defect - holdout_split_defect;
    let holdout_defect_ratio = ratio(holdout_split_defect, holdout_parent_defect);

    let holdout_pass = holdout_supported_children >= 2
        && holdout_defect_gain + SCORE_EPSILON >= config.min_holdout_defect_gain
        && holdout_defect_ratio <= config.max_holdout_defect_ratio + SCORE_EPSILON;

    let observer = holdout_pass.then_some(WitnessedCongruenceObserver {
        word: best.word,
        parent_signature_classes,
        signature_classes: best.signature_classes,
        parent_fallback_class,
    });

    Ok(CongruenceSplitFit {
        candidate_word: Some(best.word),
        observer,
        training_parent_defect: best.parent_defect,
        training_split_defect: best.split_defect,
        training_gain_after_penalty: best.gain_after_penalty,
        holdout_parent_defect,
        holdout_split_defect,
        holdout_defect_gain,
        holdout_defect_ratio,
        proposal_evaluations,
    })
}

#[derive(Debug, Clone)]
struct Candidate {
    word: ResolverWord,
    parent_defect: f64,
    split_defect: f64,
    gain_after_penalty: f64,
    signature_classes: BTreeMap<ObserverSignature, u16>,
    audit_words: Vec<ResolverWord>,
}

fn validate(
    train: &[ContinuationObservation],
    train_one_step: &BTreeMap<u64, Vec<TerminalWitness>>,
    holdout: &[ContinuationObservation],
    holdout_one_step: &BTreeMap<u64, Vec<TerminalWitness>>,
    config: CongruenceSplitConfig,
) -> Result<(), CongruenceSplitError> {
    if train.is_empty() {
        return Err(CongruenceSplitError::EmptyTraining);
    }
    if holdout.is_empty() {
        return Err(CongruenceSplitError::EmptyHoldout);
    }
    if train_one_step.is_empty() {
        return Err(CongruenceSplitError::EmptyTrainingSignatures);
    }
    if holdout_one_step.is_empty() {
        return Err(CongruenceSplitError::EmptyHoldoutSignatures);
    }
    if config.proposal_budget == 0
        || config.max_signature_classes < 2
        || config.min_train_signature_support == 0
        || config.min_holdout_signature_support == 0
        || !config.witness_deadzone.is_finite()
        || !config.witness_strong_boundary.is_finite()
        || config.witness_deadzone < 0.0
        || config.witness_strong_boundary <= config.witness_deadzone
        || !config.complexity_penalty.is_finite()
        || config.complexity_penalty < 0.0
        || !config.min_train_defect_gain_after_penalty.is_finite()
        || config.min_train_defect_gain_after_penalty < 0.0
        || !config.min_holdout_defect_gain.is_finite()
        || config.min_holdout_defect_gain < 0.0
        || !config.max_holdout_defect_ratio.is_finite()
        || config.max_holdout_defect_ratio < 0.0
    {
        return Err(CongruenceSplitError::InvalidConfig);
    }
    for observation in train.iter().chain(holdout.iter()) {
        if observation.compute_cost == 0 {
            return Err(CongruenceSplitError::ZeroComputeCost);
        }
        if observation.word.first == observation.word.second {
            return Err(CongruenceSplitError::InvalidResolverWord);
        }
    }
    Ok(())
}

fn fit_parent_classes(
    signatures: &BTreeMap<u64, Vec<TerminalWitness>>,
    max_classes: usize,
) -> (
    BTreeMap<Vec<TerminalWitness>, u16>,
    BTreeMap<u64, u16>,
) {
    let mut counts = BTreeMap::<Vec<TerminalWitness>, usize>::new();
    for signature in signatures.values() {
        *counts.entry(signature.clone()).or_default() += 1;
    }
    let mut ranked: Vec<_> = counts.into_iter().collect();
    ranked.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked.truncate(max_classes.max(1));

    let signature_classes: BTreeMap<_, _> = ranked
        .into_iter()
        .enumerate()
        .map(|(index, (signature, _))| (signature, index as u16))
        .collect();
    let classes = apply_parent_classes(signatures, &signature_classes, 0);
    (signature_classes, classes)
}

fn apply_parent_classes(
    signatures: &BTreeMap<u64, Vec<TerminalWitness>>,
    signature_classes: &BTreeMap<Vec<TerminalWitness>, u16>,
    fallback: u16,
) -> BTreeMap<u64, u16> {
    signatures
        .iter()
        .map(|(anchor_id, signature)| {
            (
                *anchor_id,
                signature_classes
                    .get(signature)
                    .copied()
                    .unwrap_or(fallback),
            )
        })
        .collect()
}

fn observation_lookup(
    observations: &[ContinuationObservation],
) -> BTreeMap<(u64, ResolverWord), TerminalWitness> {
    observations
        .iter()
        .map(|observation| {
            (
                (observation.anchor_id, observation.word),
                observation.terminal,
            )
        })
        .collect()
}

fn fit_supported_split_classes(
    parent_classes: &BTreeMap<u64, u16>,
    lookup: &BTreeMap<(u64, ResolverWord), TerminalWitness>,
    word: ResolverWord,
    first_child_class: u16,
    min_support: usize,
    max_signature_classes: usize,
) -> BTreeMap<ObserverSignature, u16> {
    let mut counts = BTreeMap::<ObserverSignature, usize>::new();
    for (anchor_id, parent_class) in parent_classes {
        let Some(terminal) = lookup.get(&(*anchor_id, word)).copied() else {
            continue;
        };
        *counts
            .entry(ObserverSignature {
                parent_class: *parent_class,
                terminal,
            })
            .or_default() += 1;
    }

    let mut supported: Vec<_> = counts
        .into_iter()
        .filter(|(_, support)| *support >= min_support)
        .collect();
    supported.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    supported.truncate(max_signature_classes);
    supported.sort_by_key(|(signature, _)| *signature);

    supported
        .into_iter()
        .enumerate()
        .map(|(index, (signature, _))| {
            (
                signature,
                first_child_class.saturating_add(index as u16),
            )
        })
        .collect()
}

fn actually_splits_parent(signature_classes: &BTreeMap<ObserverSignature, u16>) -> bool {
    let mut terminals = BTreeMap::<u16, BTreeSet<TerminalWitness>>::new();
    for signature in signature_classes.keys() {
        terminals
            .entry(signature.parent_class)
            .or_default()
            .insert(signature.terminal);
    }
    terminals.values().any(|values| values.len() >= 2)
}

fn apply_split_classes(
    one_step: &BTreeMap<u64, Vec<TerminalWitness>>,
    lookup: &BTreeMap<(u64, ResolverWord), TerminalWitness>,
    word: ResolverWord,
    parent_signature_classes: &BTreeMap<Vec<TerminalWitness>, u16>,
    parent_fallback_class: u16,
    signature_classes: &BTreeMap<ObserverSignature, u16>,
) -> BTreeMap<u64, u16> {
    one_step
        .iter()
        .map(|(anchor_id, signature)| {
            let parent_class = parent_signature_classes
                .get(signature)
                .copied()
                .unwrap_or(parent_fallback_class);
            let class = lookup
                .get(&(*anchor_id, word))
                .and_then(|terminal| {
                    signature_classes.get(&ObserverSignature {
                        parent_class,
                        terminal: *terminal,
                    })
                })
                .copied()
                .unwrap_or(parent_class);
            (*anchor_id, class)
        })
        .collect()
}

fn supported_child_count(
    parent_classes: &BTreeMap<u64, u16>,
    split_classes: &BTreeMap<u64, u16>,
    min_support: usize,
) -> usize {
    let mut counts = BTreeMap::<u16, usize>::new();
    for (anchor_id, split_class) in split_classes {
        let Some(parent_class) = parent_classes.get(anchor_id) else {
            continue;
        };
        if split_class != parent_class {
            *counts.entry(*split_class).or_default() += 1;
        }
    }
    counts
        .values()
        .filter(|support| **support >= min_support)
        .count()
}

fn candidate_is_better(candidate: &Candidate, current: &Candidate) -> bool {
    candidate
        .gain_after_penalty
        .partial_cmp(&current.gain_after_penalty)
        .unwrap_or(Ordering::Less)
        .then_with(|| current.word.cmp(&candidate.word))
        == Ordering::Greater
}

fn empty_fit(proposal_evaluations: usize) -> CongruenceSplitFit {
    CongruenceSplitFit {
        candidate_word: None,
        observer: None,
        training_parent_defect: 0.0,
        training_split_defect: 0.0,
        training_gain_after_penalty: 0.0,
        holdout_parent_defect: 0.0,
        holdout_split_defect: 0.0,
        holdout_defect_gain: 0.0,
        holdout_defect_ratio: 1.0,
        proposal_evaluations,
    }
}

fn choose_two(value: u64) -> u64 {
    value.saturating_mul(value.saturating_sub(1)) / 2
}

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator <= SCORE_EPSILON {
        if numerator <= SCORE_EPSILON {
            1.0
        } else {
            f64::INFINITY
        }
    } else {
        numerator / denominator
    }
}

fn min_terminal() -> TerminalWitness {
    TerminalWitness {
        movement: WitnessBin::StrongWorse,
        disposition: TerminalDisposition::Persisted,
    }
}

fn max_terminal() -> TerminalWitness {
    TerminalWitness {
        movement: WitnessBin::StrongBetter,
        disposition: TerminalDisposition::Resolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAT: TerminalWitness = TerminalWitness {
        movement: WitnessBin::Flat,
        disposition: TerminalDisposition::Persisted,
    };
    const SOLVED: TerminalWitness = TerminalWitness {
        movement: WitnessBin::StrongBetter,
        disposition: TerminalDisposition::Resolved,
    };

    fn toy_table(anchor_ids: &[u64]) -> Vec<ContinuationObservation> {
        let mut observations = Vec::new();
        let words = [
            ResolverWord { first: 0, second: 1 },
            ResolverWord { first: 1, second: 0 },
            ResolverWord { first: 0, second: 2 },
            ResolverWord { first: 2, second: 0 },
        ];
        for (index, anchor_id) in anchor_ids.iter().enumerate() {
            let alpha = index < anchor_ids.len() / 2;
            for word in words {
                let terminal = match (word.first, word.second, alpha) {
                    (0, 1, true) | (1, 0, false) | (0, 2, true) | (2, 0, false) => SOLVED,
                    _ => FLAT,
                };
                observations.push(ContinuationObservation {
                    anchor_id: *anchor_id,
                    word,
                    terminal,
                    compute_cost: 2,
                });
            }
        }
        observations
    }

    fn signatures(anchor_ids: &[u64]) -> BTreeMap<u64, Vec<TerminalWitness>> {
        anchor_ids
            .iter()
            .copied()
            .map(|anchor_id| (anchor_id, vec![FLAT, FLAT, FLAT]))
            .collect()
    }

    #[test]
    fn directed_motion_preserves_worsening_signal() {
        let witness = OutcomeWitness::new(
            "objective",
            0.8,
            0.2,
            ImprovementDirection::HigherIsBetter,
            vec![],
        );
        assert!((directed_normalized_motion(&witness).unwrap() + 0.6).abs() < 1e-12);
    }

    #[test]
    fn identical_one_step_behavior_can_be_split_by_continuation() {
        let train_ids: Vec<u64> = (1..=16).collect();
        let holdout_ids: Vec<u64> = (101..=108).collect();
        let config = CongruenceSplitConfig {
            proposal_budget: 4,
            min_train_signature_support: 8,
            min_holdout_signature_support: 4,
            complexity_penalty: 0.0,
            min_train_defect_gain_after_penalty: 0.1,
            min_holdout_defect_gain: 0.1,
            max_holdout_defect_ratio: 0.9,
            ..CongruenceSplitConfig::default()
        };
        let fit = fit_witnessed_congruence_split(
            &toy_table(&train_ids),
            &signatures(&train_ids),
            &toy_table(&holdout_ids),
            &signatures(&holdout_ids),
            config,
        )
        .unwrap();

        assert!(fit.applied());
        assert_eq!(fit.proposal_evaluations, 4);
        assert!(fit.training_gain_after_penalty > 0.1);
        assert!(fit.holdout_defect_gain > 0.1);
    }

    #[test]
    fn observer_abstains_to_parent_on_unknown_terminal_signature() {
        let observer = WitnessedCongruenceObserver {
            word: ResolverWord { first: 0, second: 1 },
            parent_signature_classes: BTreeMap::from([(vec![FLAT], 3)]),
            signature_classes: BTreeMap::from([(
                ObserverSignature {
                    parent_class: 3,
                    terminal: SOLVED,
                },
                4,
            )]),
            parent_fallback_class: 0,
        };
        assert_eq!(observer.class(&[FLAT], Some(FLAT)), 3);
        assert_eq!(observer.class(&[FLAT], None), 3);
    }

    #[test]
    fn no_behavioral_noncongruence_is_not_promoted() {
        let train_ids: Vec<u64> = (1..=16).collect();
        let holdout_ids: Vec<u64> = (101..=108).collect();
        let words = [
            ResolverWord { first: 0, second: 1 },
            ResolverWord { first: 1, second: 0 },
        ];
        let flat_table = |ids: &[u64]| {
            ids.iter()
                .flat_map(|anchor_id| {
                    words.map(|word| ContinuationObservation {
                        anchor_id: *anchor_id,
                        word,
                        terminal: FLAT,
                        compute_cost: 2,
                    })
                })
                .collect::<Vec<_>>()
        };
        let config = CongruenceSplitConfig {
            proposal_budget: 2,
            min_train_signature_support: 8,
            min_holdout_signature_support: 4,
            ..CongruenceSplitConfig::default()
        };
        let fit = fit_witnessed_congruence_split(
            &flat_table(&train_ids),
            &signatures(&train_ids),
            &flat_table(&holdout_ids),
            &signatures(&holdout_ids),
            config,
        )
        .unwrap();
        assert!(!fit.applied());
        assert!(fit.candidate_word.is_none());
    }

    #[test]
    fn candidate_ties_are_deterministic() {
        let train_ids: Vec<u64> = (1..=16).collect();
        let holdout_ids: Vec<u64> = (101..=108).collect();
        let config = CongruenceSplitConfig {
            proposal_budget: 4,
            min_train_signature_support: 8,
            min_holdout_signature_support: 4,
            complexity_penalty: 0.0,
            min_train_defect_gain_after_penalty: 0.1,
            min_holdout_defect_gain: 0.1,
            max_holdout_defect_ratio: 0.9,
            ..CongruenceSplitConfig::default()
        };
        let first = fit_witnessed_congruence_split(
            &toy_table(&train_ids),
            &signatures(&train_ids),
            &toy_table(&holdout_ids),
            &signatures(&holdout_ids),
            config,
        )
        .unwrap();
        let second = fit_witnessed_congruence_split(
            &toy_table(&train_ids),
            &signatures(&train_ids),
            &toy_table(&holdout_ids),
            &signatures(&holdout_ids),
            config,
        )
        .unwrap();
        assert_eq!(first.candidate_word, second.candidate_word);
        assert_eq!(first.observer, second.observer);
    }
}
