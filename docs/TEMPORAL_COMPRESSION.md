# Temporal Compression Module — Starfire Integration Spec

## Problem Statement

Phase 1 of the Compression Intelligence research (temporalbench v1-v4) found:

> **Temporal indexing matters more than noise filtering.** The variation in model accuracy comes not from how much context to ignore, but from whether the system can identify which facts are *current* vs *stale*.

System A (Plain RAG) achieved perfect scores on ChangeDetection and CausalTrace but failed on AsOfQA (temporal "what is true now" questions). Systems B/C/D passed because they tracked validity windows and/or applied decay.

Starfire's current world_model has `Entity.last_updated` but:
- ❌ No per-property validity windows (`valid_from`/`valid_until`)
- ❌ No decay functions
- ❌ No staleness scoring in queries
- ❌ No temporal filtering on `EntityQuery`

## Design

### 1. TemporalPropertyValue

Replace `PropertyValue` with a temporally-aware version that tracks when a value was true:

```rust
/// A property value with temporal validity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalProperty {
    /// The value
    pub value: PropertyValue,
    /// When this value became valid (Unix timestamp)
    pub valid_from: i64,
    /// When this value stopped being valid (None = still valid)
    pub valid_until: Option<i64>,
    /// Confidence at time of recording
    pub confidence: f64,
    /// Decay function for this property
    pub decay_fn: DecayFunction,
}

/// Decay function for staleness scoring
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DecayFunction {
    /// Exponential decay with domain-specific half-life
    DomainHalfLife {
        half_life_hours: f64,
    },
    /// Linear decay over a fixed window
    Linear { window_hours: f64 },
    /// No decay — validity window only
    None,
    /// Custom decay rate
    Custom { rate: f64 },
}
```

### 2. Entity Upgrade

```rust
/// Entity with temporal property tracking
pub struct TemporalEntity {
    pub id: EntityId,
    pub name: String,
    /// Map of property_name → Vec of temporal values (history)
    /// Sorted by valid_from descending (most recent first)
    pub temporal_properties: HashMap<String, Vec<TemporalProperty>>,
    pub relations: Vec<Relation>,
    pub last_updated: i64,
    pub confidence: f64,
}
```

**Key behavior:** When a property is updated, the *old* value is NOT overwritten — it's kept with `valid_until` set to the current time. This creates a complete history that enables:
- "What was true at time T?"
- "When did X change?"
- "How many times did X change?"

### 3. TemporalQuery

Extend `EntityQuery` with temporal filters:

```rust
/// Query with temporal awareness
pub struct TemporalQuery {
    pub base: EntityQuery,
    /// Filter: only return facts valid at this time (None = now)
    pub valid_at: Option<i64>,
    /// Filter: minimum recency score [0.0, 1.0]
    pub min_recency: Option<f64>,
    /// Filter: maximum staleness score [0.0, 1.0]  
    pub max_staleness: Option<f64>,
    /// Include stale (superseded) properties in results
    pub include_stale: bool,
    /// Decay parameters override (for this query)
    pub decay_override: Option<DecayParams>,
}
```

### 4. StalenessScorer

Computes how stale a fact is given current time and update frequency:

```rust
/// Parameters for staleness scoring
pub struct DecayParams {
    /// Half-life in hours for exponential decay
    pub half_life_hours: f64,
    /// How much to weight age vs update frequency
    pub frequency_weight: f64,
}

impl DecayParams {
    /// Domain-adaptive decay — different domains decay at different rates
    pub fn for_domain(domain: &str) -> Self {
        match domain {
            "fast" => Self { half_life_hours: 1.0, frequency_weight: 0.7 },
            "medium" => Self { half_life_hours: 24.0, frequency_weight: 0.5 },
            "slow" => Self { half_life_hours: 168.0, frequency_weight: 0.3 },
            _ => Self { half_life_hours: 24.0, frequency_weight: 0.5 },
        }
    }
}

/// Compute staleness score for a temporal property
/// Returns [0.0, 1.0] where 0 = fresh, 1 = maximally stale
pub fn compute_staleness(
    property: &TemporalProperty,
    now: i64,
    params: &DecayParams,
) -> f64 {
    let age_hours = (now - property.valid_from) as f64 / 3600.0;
    
    match property.decay_fn {
        DecayFunction::DomainHalfLife { half_life_hours } => {
            // Exponential decay: staleness = 1 - 0.5^(age / half_life)
            let half_lives = age_hours / half_life_hours;
            1.0 - 0.5_f64.powf(half_lives)
        }
        DecayFunction::Linear { window_hours } => {
            // Linear decay over fixed window
            (age_hours / window_hours).clamp(0.0, 1.0)
        }
        DecayFunction::None => {
            // Step function: 0 if valid, 1.0 if not
            if property.valid_until.is_none() { 0.0 } else { 1.0 }
        }
        DecayFunction::Custom { rate } => {
            let exponent = age_hours / rate;
            1.0 - (-exponent / std::f64::consts::LN_2).exp()
        }
    }
}
```

### 5. TemporalIndex Trait

Implement on `WorldModel` for efficient temporal queries:

```rust
impl WorldModel {
    /// Get the current (non-stale) value for a property at a given time
    pub fn get_current_value(
        &self,
        entity_id: &EntityId,
        property: &str,
        valid_at: Option<i64>,
    ) -> Option<&TemporalProperty> {
        let now = valid_at.unwrap_or_else(crate::now_timestamp);
        self.entities
            .get(entity_id)?
            .temporal_properties
            .get(property)?
            .iter()
            .find(|p| p.valid_from <= now && p.valid_until.unwrap_or(i64::MAX) > now)
    }
    
    /// Get all values for a property (including historical), filtered by staleness
    pub fn get_property_history(
        &self,
        entity_id: &EntityId,
        property: &str,
        max_staleness: f64,
    ) -> Vec<&TemporalProperty> {
        let now = crate::now_timestamp();
        self.entities
            .get(entity_id)
            .and_then(|e| e.temporal_properties.get(property))
            .map(|props| {
                props.iter()
                    .filter(|p| {
                        let staleness = compute_staleness(p, now, &DecayParams::default());
                        staleness <= max_staleness
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Query entities with temporal filtering
    pub fn query_temporal(&self, query: TemporalQuery) -> Vec<&TemporalEntity> {
        let now = query.valid_at.unwrap_or_else(crate::now_timestamp);
        
        self.entities.values()
            .filter(|e| {
                // Base filters
                if let Some(ref pattern) = query.base.name_pattern {
                    if !e.name.to_lowercase().contains(&pattern.to_lowercase()) {
                        return false;
                    }
                }
                
                // Temporal filter: entity must have at least one valid property
                let has_valid = e.temporal_properties.values().any(|props| {
                    props.iter().any(|p| {
                        p.valid_from <= now && 
                        p.valid_until.unwrap_or(i64::MAX) > now
                    })
                });
                if !has_valid && !query.include_stale {
                    return false;
                }
                
                // Staleness filter
                if let Some(max_stale) = query.max_staleness {
                    let min_staleness = e.temporal_properties.values()
                        .flatten()
                        .map(|p| compute_staleness(p, now, &DecayParams::default()))
                        .fold(f64::MAX, f64::min);
                    if min_staleness > max_stale {
                        return false;
                    }
                }
                
                true
            })
            .take(query.base.limit)
            .collect()
    }
}
```

## Integration Points

1. **`Entity.id` → `TemporalEntity`** — Replace `HashMap<String, PropertyValue>` with `HashMap<String, Vec<TemporalProperty>>`
2. **`EntityQuery` → `TemporalQuery`** — Add temporal parameters to existing queries
3. **Perception pipeline** — When new facts arrive from Quanot, wrap them in `TemporalProperty` with `valid_from = now()`
4. **KG reasoning** — `TemporalQuery` flows through the existing KG query path
5. **LLM polisher** — Pass `TemporalQuery::valid_at` when the user asks "what is X now?"

## Why This Fixes System A's Failure

System A failed because it returned the most *recently added* fact rather than the most *currently valid* fact. With temporal validity windows:

- Query: "What is X's value as of day 59?"
- System checks: which value of X has `valid_from ≤ 59 < valid_until`?
- Returns the correct value (or marks as unknown if no value covers that time)

The staleness scorer additionally enables decayed retrieval: even if a value is "stale" (superseded), it can still be retrieved with a penalty, which is useful when no current value exists.

## Implementation Order

1. `TemporalProperty` + `DecayFunction` types (pure data, no logic)
2. `TemporalEntity` (replace `Entity.properties`)
3. `DecayParams` + `compute_staleness()`
4. `TemporalQuery` (extend `EntityQuery`)
5. `WorldModel::get_current_value()` + `get_property_history()`
6. `WorldModel::query_temporal()`
7. Perception pipeline update
8. Integration with KG reasoning

## References

- `C:\Users\Zwmar\dev\temporal-attention\hybrid_store.py` — Python reference implementation (TemporalAttentionStore, HybridStore)
- `C:\Users\Zwmar\dev\compression_intelligence\analyze_compression.py` — Phase 1 analysis
- `C:\Users\Zwmar\.openclaw\workspace\projects\starfire\lib\world_model\state.rs` — existing EntityQuery
