//! Temporal Types — Validity windows, decay functions, and staleness scoring
//!
//! This module provides the core temporal data structures for tracking
//! when facts are valid vs stale.

use serde::{Deserialize, Serialize};

/// A property value with temporal validity (when it became true and when it stopped)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalProperty {
    /// The value
    pub value: PropertyValue,
    /// When this value became valid (Unix timestamp in seconds)
    pub valid_from: i64,
    /// When this value stopped being valid (None = still valid)
    pub valid_until: Option<i64>,
    /// Confidence at time of recording
    pub confidence: f64,
    /// Decay function for staleness scoring
    pub decay_fn: DecayFunction,
}

impl TemporalProperty {
    /// Check if this property was valid at the given time
    pub fn was_valid_at(&self, time: i64) -> bool {
        self.valid_from <= time && self.valid_until.unwrap_or(i64::MAX) > time
    }

    /// Check if this property is currently valid (at the current time)
    pub fn is_currently_valid(&self, now: i64) -> bool {
        self.was_valid_at(now)
    }
}

/// A property value — can be various types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<String>),
}

impl From<String> for PropertyValue {
    fn from(s: String) -> Self {
        PropertyValue::String(s)
    }
}

impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self {
        PropertyValue::String(s.to_string())
    }
}

impl From<f64> for PropertyValue {
    fn from(n: f64) -> Self {
        PropertyValue::Number(n)
    }
}

impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Boolean(b)
    }
}

/// Decay function for staleness scoring
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DecayFunction {
    /// Exponential decay with a domain-specific half-life
    /// Staleness = 1 - 0.5^(age_hours / half_life_hours)
    DomainHalfLife { half_life_hours: f64 },
    /// Linear decay over a fixed window
    /// Staleness = clamp(age_hours / window_hours, 0.0, 1.0)
    Linear { window_hours: f64 },
    /// No decay — validity determined entirely by validity window
    /// Returns 0.0 if currently valid, 1.0 if expired
    None,
    /// Custom exponential decay with arbitrary rate
    /// Staleness = 1 - 2^(-age_hours / rate)
    Custom { rate: f64 },
}

impl Default for DecayFunction {
    fn default() -> Self {
        DecayFunction::None
    }
}

/// Parameters for staleness scoring
#[derive(Debug, Clone, Copy)]
pub struct DecayParams {
    /// Half-life in hours for exponential decay (used in DomainHalfLife variant)
    pub half_life_hours: f64,
    /// How much to weight age vs update frequency (0.0 to 1.0)
    pub frequency_weight: f64,
}

impl Default for DecayParams {
    fn default() -> Self {
        Self {
            half_life_hours: 24.0,
            frequency_weight: 0.5,
        }
    }
}

impl DecayParams {
    /// Domain-adaptive decay parameters — different domains decay at different rates
    ///
    /// - "fast": High-flux properties that change often (e.g., weather, mood)
    /// - "medium": Normal properties with moderate change rates
    /// - "slow": Static facts that rarely change (e.g., species, physics laws)
    pub fn for_domain(domain: &str) -> Self {
        match domain {
            "fast" => Self {
                half_life_hours: 1.0,
                frequency_weight: 0.7,
            },
            "medium" => Self {
                half_life_hours: 24.0,
                frequency_weight: 0.5,
            },
            "slow" => Self {
                half_life_hours: 168.0, // ~1 week
                frequency_weight: 0.3,
            },
            _ => Self {
                half_life_hours: 24.0,
                frequency_weight: 0.5,
            },
        }
    }
}

/// Compute staleness score for a temporal property
///
/// Returns [0.0, 1.0] where:
/// - 0.0 = completely fresh (just recorded)
/// - 1.0 = maximally stale (expired or decayed to minimum relevance)
///
/// The computation uses the property's decay function to determine how
/// quickly it loses relevance over time.
pub fn compute_staleness(
    property: &TemporalProperty,
    now: i64,
    _params: &DecayParams,
) -> f64 {
    let age_hours = (now - property.valid_from) as f64 / 3600.0;

    match property.decay_fn {
        DecayFunction::DomainHalfLife { half_life_hours } => {
            // Exponential decay: staleness = 1 - 0.5^(age / half_life)
            let half_lives = age_hours / half_life_hours;
            1.0 - 0.5_f64.powf(half_lives)
        }
        DecayFunction::Linear { window_hours } => {
            // Linear decay over fixed window, clamped to [0, 1]
            (age_hours / window_hours).clamp(0.0, 1.0)
        }
        DecayFunction::None => {
            // Step function: 0 if currently valid, 1 if expired
            if property.valid_until.is_none() {
                0.0
            } else {
                1.0
            }
        }
        DecayFunction::Custom { rate } => {
            // Custom exponential: staleness = 1 - 2^(-age_hours / rate)
            // This is equivalent to: 1 - e^(-age_hours / (rate / ln(2)))
            // Using the identity: 2^(-x) = e^(-x * ln(2))
            let exponent = age_hours / rate;
            1.0 - (-exponent * std::f64::consts::LN_2).exp()
        }
    }
}

/// Query with temporal awareness — extends EntityQuery with time-based filters
#[derive(Debug, Clone)]
pub struct TemporalQuery {
    /// Base entity query filters
    pub base: super::state::EntityQuery,
    /// Filter: only return facts valid at this time (None = now)
    pub valid_at: Option<i64>,
    /// Filter: minimum recency score [0.0, 1.0] (alias for max_staleness inverted)
    pub min_recency: Option<f64>,
    /// Filter: maximum staleness score [0.0, 1.0]
    pub max_staleness: Option<f64>,
    /// Include stale (superseded) properties in results
    pub include_stale: bool,
    /// Decay parameters override for this query
    pub decay_override: Option<DecayParams>,
}

impl Default for TemporalQuery {
    fn default() -> Self {
        Self {
            base: super::state::EntityQuery::default(),
            valid_at: None,
            min_recency: None,
            max_staleness: None,
            include_stale: false,
            decay_override: None,
        }
    }
}

impl TemporalQuery {
    /// Create a temporal query at the current time
    pub fn now() -> Self {
        Self::default()
    }

    /// Create a temporal query valid at a specific time
    pub fn valid_at(mut self, time: i64) -> Self {
        self.valid_at = Some(time);
        self
    }

    /// Set the maximum staleness threshold
    pub fn max_staleness(mut self, threshold: f64) -> Self {
        self.max_staleness = Some(threshold);
        self
    }

    /// Include stale (superseded) properties in results
    pub fn include_stale(mut self) -> Self {
        self.include_stale = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_staleness_domain_half_life() {
        let prop = TemporalProperty {
            value: PropertyValue::from("test"),
            valid_from: 0,
            valid_until: None,
            confidence: 1.0,
            decay_fn: DecayFunction::DomainHalfLife { half_life_hours: 10.0 },
        };

        // At age 0: staleness = 1 - 0.5^0 = 0
        assert!((compute_staleness(&prop, 0, &DecayParams::default()) - 0.0).abs() < 0.001);

        // At age = half_life (10 hours): staleness = 1 - 0.5^1 = 0.5
        let staleness = compute_staleness(&prop, 10 * 3600, &DecayParams::default());
        assert!((staleness - 0.5).abs() < 0.001);

        // At age = 2 * half_life (20 hours): staleness = 1 - 0.5^2 = 0.75
        let staleness = compute_staleness(&prop, 20 * 3600, &DecayParams::default());
        assert!((staleness - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_compute_staleness_linear() {
        let prop = TemporalProperty {
            value: PropertyValue::from("test"),
            valid_from: 0,
            valid_until: None,
            confidence: 1.0,
            decay_fn: DecayFunction::Linear { window_hours: 10.0 },
        };

        // At age 0: staleness = 0
        assert!((compute_staleness(&prop, 0, &DecayParams::default()) - 0.0).abs() < 0.001);

        // At age 5 hours (half window): staleness = 0.5
        let staleness = compute_staleness(&prop, 5 * 3600, &DecayParams::default());
        assert!((staleness - 0.5).abs() < 0.001);

        // At age 10 hours (full window): staleness = 1.0
        let staleness = compute_staleness(&prop, 10 * 3600, &DecayParams::default());
        assert!((staleness - 1.0).abs() < 0.001);

        // Beyond window: still 1.0 (clamped)
        let staleness = compute_staleness(&prop, 20 * 3600, &DecayParams::default());
        assert!((staleness - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_staleness_none() {
        // Currently valid (valid_until = None)
        let prop = TemporalProperty {
            value: PropertyValue::from("test"),
            valid_from: 0,
            valid_until: None,
            confidence: 1.0,
            decay_fn: DecayFunction::None,
        };
        assert!((compute_staleness(&prop, 1000, &DecayParams::default()) - 0.0).abs() < 0.001);

        // Expired (valid_until = Some)
        let prop = TemporalProperty {
            value: PropertyValue::from("test"),
            valid_from: 0,
            valid_until: Some(100),
            confidence: 1.0,
            decay_fn: DecayFunction::None,
        };
        assert!((compute_staleness(&prop, 200, &DecayParams::default()) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_staleness_custom() {
        let prop = TemporalProperty {
            value: PropertyValue::from("test"),
            valid_from: 0,
            valid_until: None,
            confidence: 1.0,
            decay_fn: DecayFunction::Custom { rate: 10.0 },
        };

        // At age = rate: staleness = 1 - 2^(-1) = 1 - 0.5 = 0.5
        let staleness = compute_staleness(&prop, 10 * 3600, &DecayParams::default());
        assert!((staleness - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_temporal_property_was_valid_at() {
        let prop = TemporalProperty {
            value: PropertyValue::from("test"),
            valid_from: 100,
            valid_until: Some(200),
            confidence: 1.0,
            decay_fn: DecayFunction::None,
        };

        assert!(!prop.was_valid_at(50));   // Before valid_from
        assert!(prop.was_valid_at(150));  // Between valid_from and valid_until
        assert!(!prop.was_valid_at(250)); // After valid_until
        assert!(prop.was_valid_at(100));  // At exactly valid_from
        assert!(!prop.was_valid_at(200)); // At exactly valid_until (exclusive)
    }

    #[test]
    fn test_decays_for_domain() {
        let fast = DecayParams::for_domain("fast");
        assert!((fast.half_life_hours - 1.0).abs() < 0.001);

        let medium = DecayParams::for_domain("medium");
        assert!((medium.half_life_hours - 24.0).abs() < 0.001);

        let slow = DecayParams::for_domain("slow");
        assert!((slow.half_life_hours - 168.0).abs() < 0.001);

        let unknown = DecayParams::for_domain("unknown");
        assert!((unknown.half_life_hours - 24.0).abs() < 0.001); // Defaults to medium
    }

    #[test]
    fn test_property_value_conversions() {
        let s: PropertyValue = "hello".into();
        assert_eq!(s, PropertyValue::String("hello".to_string()));

        let n: PropertyValue = 42.0.into();
        assert_eq!(n, PropertyValue::Number(42.0));

        let b: PropertyValue = true.into();
        assert_eq!(b, PropertyValue::Boolean(true));
    }
}
