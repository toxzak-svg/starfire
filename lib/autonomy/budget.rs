use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Hard limits for one autonomous episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub max_steps: u64,
    pub max_action_cost: u64,
    pub max_compute_cost: u64,
}

impl ResourceBudget {
    pub const fn new(max_steps: u64, max_action_cost: u64, max_compute_cost: u64) -> Self {
        Self {
            max_steps,
            max_action_cost,
            max_compute_cost,
        }
    }
}

/// Conservatively accounted resource use for one episode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetUsage {
    pub steps: u64,
    pub action_cost: u64,
    pub compute_cost: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BudgetError {
    #[error("step budget exhausted")]
    Steps,
    #[error("action-cost budget exhausted")]
    ActionCost,
    #[error("compute-cost budget exhausted")]
    ComputeCost,
    #[error("resource accounting overflow")]
    Overflow,
}

impl BudgetUsage {
    /// Reserve one action before it is executed. Declared costs are charged even
    /// when the environment later reports a lower actual cost, keeping the
    /// accounting conservative and deterministic.
    pub fn reserve(
        &mut self,
        budget: ResourceBudget,
        declared_action_cost: u64,
        compute_cost: u64,
    ) -> Result<(), BudgetError> {
        let next_steps = self.steps.checked_add(1).ok_or(BudgetError::Overflow)?;
        let next_action = self
            .action_cost
            .checked_add(declared_action_cost)
            .ok_or(BudgetError::Overflow)?;
        let next_compute = self
            .compute_cost
            .checked_add(compute_cost)
            .ok_or(BudgetError::Overflow)?;

        if next_steps > budget.max_steps {
            return Err(BudgetError::Steps);
        }
        if next_action > budget.max_action_cost {
            return Err(BudgetError::ActionCost);
        }
        if next_compute > budget.max_compute_cost {
            return Err(BudgetError::ComputeCost);
        }

        self.steps = next_steps;
        self.action_cost = next_action;
        self.compute_cost = next_compute;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservation_is_atomic_when_a_limit_would_be_exceeded() {
        let budget = ResourceBudget::new(1, 2, 3);
        let mut usage = BudgetUsage::default();
        usage.reserve(budget, 2, 3).unwrap();
        let before = usage;

        assert_eq!(usage.reserve(budget, 1, 1), Err(BudgetError::Steps));
        assert_eq!(usage, before);
    }
}
