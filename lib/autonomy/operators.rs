use std::cmp::Ordering;

use thiserror::Error;

use crate::charge::Charge;
use crate::environment::{Environment, ObjectiveFeedback};

use super::broker::ActionAuthority;

pub struct OperatorContext<'a, E: Environment> {
    pub observation: &'a E::Observation,
    pub available_actions: &'a [E::Action],
    pub objective: &'a ObjectiveFeedback,
    pub charge: &'a Charge,
    pub step_index: u64,
}

#[derive(Debug, Clone)]
pub struct OperatorProposal<A> {
    pub action: A,
    pub authority: ActionAuthority,
    pub rationale: String,
    pub predicted_effect: String,
    pub expected_utility: f32,
    pub requested_discharge: f32,
    pub compute_cost: u64,
    pub declared_action_cost: u64,
}

pub trait CognitiveOperator<E: Environment>: 'static {
    fn id(&self) -> &str;

    fn propose(&mut self, context: &OperatorContext<'_, E>) -> Option<OperatorProposal<E::Action>>;
}

#[derive(Debug, Clone)]
pub struct SelectedOperator<A> {
    pub operator_id: String,
    pub proposal: OperatorProposal<A>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegistryError {
    #[error("operator id cannot be empty")]
    EmptyId,
    #[error("duplicate operator id: {0}")]
    DuplicateId(String),
}

/// Deterministic registry that lets operators compete under one common contract.
pub struct OperatorRegistry<E: Environment> {
    operators: Vec<Box<dyn CognitiveOperator<E>>>,
}

impl<E: Environment + 'static> Default for OperatorRegistry<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Environment + 'static> OperatorRegistry<E> {
    pub fn new() -> Self {
        Self {
            operators: Vec::new(),
        }
    }

    pub fn register<O>(&mut self, operator: O) -> Result<(), RegistryError>
    where
        O: CognitiveOperator<E>,
    {
        let id = operator.id().trim();
        if id.is_empty() {
            return Err(RegistryError::EmptyId);
        }
        if self.operators.iter().any(|existing| existing.id() == id) {
            return Err(RegistryError::DuplicateId(id.to_string()));
        }
        self.operators.push(Box::new(operator));
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.operators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operators.is_empty()
    }

    /// Select the highest-utility applicable proposal. Ties are broken by stable
    /// operator id so replay does not depend on registration order.
    pub fn select(
        &mut self,
        context: &OperatorContext<'_, E>,
    ) -> Option<SelectedOperator<E::Action>> {
        let mut best: Option<SelectedOperator<E::Action>> = None;

        for operator in &mut self.operators {
            let Some(proposal) = operator.propose(context) else {
                continue;
            };
            if !proposal.expected_utility.is_finite()
                || !proposal.requested_discharge.is_finite()
                || proposal.expected_utility < 0.0
                || proposal.requested_discharge < 0.0
                || proposal.compute_cost == 0
            {
                continue;
            }

            let candidate = SelectedOperator {
                operator_id: operator.id().to_string(),
                proposal,
            };

            let replace = match &best {
                None => true,
                Some(current) => match candidate
                    .proposal
                    .expected_utility
                    .total_cmp(&current.proposal.expected_utility)
                {
                    Ordering::Greater => true,
                    Ordering::Equal => candidate.operator_id < current.operator_id,
                    Ordering::Less => false,
                },
            };

            if replace {
                best = Some(candidate);
            }
        }

        best
    }
}
