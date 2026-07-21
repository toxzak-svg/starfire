//! Bounded shadow autonomy for Starfire.
//!
//! This module deliberately sits outside `Runtime::chat()`. It establishes a
//! falsifiable observe/pressure/operator/action/judge loop without granting live
//! production authority, automatic ontology promotion, or self-edit-to-main.

pub mod broker;
pub mod budget;
pub mod goals;
pub mod kernel;
pub mod operators;

pub use broker::{ActionAuthority, ActionBroker, ActionDenied, ActionEnvelope};
pub use budget::{BudgetError, BudgetUsage, ResourceBudget};
pub use goals::{Goal, GoalId, GoalSource, GoalStatus};
pub use kernel::{AutonomousKernel, EpisodeReport, EpisodeStepRecord, EpisodeTermination};
pub use operators::{
    CognitiveOperator, OperatorContext, OperatorProposal, OperatorRegistry, RegistryError,
    SelectedOperator,
};
