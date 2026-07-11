use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Increasing levels of side-effect authority.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum ActionAuthority {
    Observe,
    ReversibleSandbox,
    PersistentBounded,
    ExternalSideEffect,
}

#[derive(Debug, Clone)]
pub struct ActionEnvelope<A> {
    pub action: A,
    pub authority: ActionAuthority,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("requested authority {requested:?} exceeds autonomous ceiling {ceiling:?}")]
pub struct ActionDenied {
    pub requested: ActionAuthority,
    pub ceiling: ActionAuthority,
}

/// Central authority boundary for autonomous actions.
#[derive(Debug, Clone)]
pub struct ActionBroker {
    max_autonomous_authority: ActionAuthority,
    denied_attempts: u64,
}

impl ActionBroker {
    pub fn new(max_autonomous_authority: ActionAuthority) -> Self {
        Self {
            max_autonomous_authority,
            denied_attempts: 0,
        }
    }

    pub fn sandbox_only() -> Self {
        Self::new(ActionAuthority::ReversibleSandbox)
    }

    pub fn authorize<A>(&mut self, action: &ActionEnvelope<A>) -> Result<(), ActionDenied> {
        if action.authority > self.max_autonomous_authority {
            self.denied_attempts = self.denied_attempts.saturating_add(1);
            return Err(ActionDenied {
                requested: action.authority,
                ceiling: self.max_autonomous_authority,
            });
        }
        Ok(())
    }

    pub fn denied_attempts(&self) -> u64 {
        self.denied_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_broker_rejects_external_side_effects() {
        let mut broker = ActionBroker::sandbox_only();
        let request = ActionEnvelope {
            action: "publish",
            authority: ActionAuthority::ExternalSideEffect,
            rationale: "test".into(),
        };

        assert!(broker.authorize(&request).is_err());
        assert_eq!(broker.denied_attempts(), 1);
    }
}
