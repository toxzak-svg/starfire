use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoalId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalSource {
    External,
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    Pending,
    Active,
    Achieved,
    Failed,
    Blocked,
    BudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    pub id: GoalId,
    pub description: String,
    pub source: GoalSource,
    pub priority: f32,
    pub status: GoalStatus,
}

impl Goal {
    pub fn external(id: u64, description: impl Into<String>) -> Self {
        Self {
            id: GoalId(id),
            description: description.into(),
            source: GoalSource::External,
            priority: 1.0,
            status: GoalStatus::Pending,
        }
    }

    pub fn activate(&mut self) {
        self.status = GoalStatus::Active;
    }
}
