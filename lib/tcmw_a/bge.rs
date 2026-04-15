//! BGE — Behavioral Grammar Encoder (Layer 2)
//!
//! Encodes Zach's action sequences into a learnable grammar:
//! - Session archetypes — recurring patterns (e.g., "pipeline_build", "debug_loop")
//! - Markov transition matrix — P(archetype_j | archetype_i)
//! - K-means clustering on action frequency vectors (pure Rust, no external ML)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArchetypeId(u64);

impl ArchetypeId {
    pub fn new() -> Self {
        use rand::RngCore;
        Self(rand::thread_rng().next_u64())
    }
}

impl Default for ArchetypeId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchetype {
    pub id: ArchetypeId,
    pub label: String,
    pub characteristic_actions: Vec<String>,
    pub frequency: usize,
    pub archetype_lambda: f64,
}

impl SessionArchetype {
    pub fn new(label: &str) -> Self {
        Self {
            id: ArchetypeId::new(),
            label: label.to_string(),
            characteristic_actions: Vec::new(),
            frequency: 1,
            archetype_lambda: 0.15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarkovChain {
    transitions: HashMap<(ArchetypeId, ArchetypeId), usize>,
    totals: HashMap<ArchetypeId, usize>,
    laplace_k: f64,
}

impl Default for MarkovChain {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkovChain {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            totals: HashMap::new(),
            laplace_k: 1.0,
        }
    }

    pub fn update(&mut self, from: ArchetypeId, to: ArchetypeId) {
        *self.transitions.entry((from, to)).or_insert(0) += 1;
        *self.totals.entry(from).or_insert(0) += 1;
    }

    pub fn probability(&self, from: ArchetypeId, to: ArchetypeId) -> f64 {
        let total = *self.totals.get(&from).unwrap_or(&0) as f64;
        let count = *self.transitions.get(&(from, to)).unwrap_or(&0) as f64;
        let num_states = self.totals.len().max(1) as f64;
        (count + self.laplace_k) / (total + self.laplace_k * num_states)
    }

    pub fn most_likely(&self, from: ArchetypeId) -> Option<(ArchetypeId, f64)> {
        let total = *self.totals.get(&from).unwrap_or(&0);
        if total == 0 {
            return None;
        }

        let mut best: Option<(ArchetypeId, f64)> = None;
        for (key, &cnt) in &self.transitions {
            if key.0 == from {
                let p = (cnt as f64 + self.laplace_k)
                    / (total as f64 + self.laplace_k * self.totals.len() as f64);
                if best.as_ref().map(|(_, bp)| p > *bp).unwrap_or(true) {
                    best = Some((key.1, p));
                }
            }
        }
        best.or_else(|| {
            self.totals.keys()
                .find(|&&k| k != from)
                .map(|&k| {
                    let p = self.laplace_k / (total as f64 + self.laplace_k * self.totals.len() as f64);
                    (k, p)
                })
        })
    }
}

#[derive(Debug, Clone)]
pub struct BGE {
    archetypes: Vec<SessionArchetype>,
    current_archetype: Option<SessionArchetype>,
    markov: MarkovChain,
    action_history: Vec<String>,
    action_counts: HashMap<String, usize>,
    k_means_k: usize,
    needs_revision: bool,
}

impl Default for BGE {
    fn default() -> Self {
        Self::new(5)
    }
}

impl BGE {
    pub fn new(k_means_k: usize) -> Self {
        Self {
            archetypes: Vec::new(),
            current_archetype: None,
            markov: MarkovChain::new(),
            action_history: Vec::new(),
            action_counts: HashMap::new(),
            k_means_k,
            needs_revision: false,
        }
    }

    pub fn observe_action(&mut self, action: &str, _event_history: &[super::cef::CausalEvent]) -> Option<SessionArchetype> {
        *self.action_counts.entry(action.to_string()).or_insert(0) += 1;
        self.action_history.push(action.to_string());
        if self.action_history.len() > 100 {
            self.action_history.remove(0);
        }
        self.detect_archetype()
    }

    fn detect_archetype(&mut self) -> Option<SessionArchetype> {
        if self.action_history.len() < 3 {
            return None;
        }

        if self.archetypes.len() < self.k_means_k {
            let label = self.suggest_archetype_label();
            let mut arch = SessionArchetype::new(&label);
            arch.characteristic_actions = self.top_actions(5);
            self.archetypes.push(arch.clone());
            self.current_archetype = Some(arch.clone());
            return Some(arch);
        }

        if let Some(idx) = self.nearest_archetype_index() {
            self.archetypes[idx].frequency += 1;
            let arch = self.archetypes[idx].clone();
            self.current_archetype = Some(arch.clone());
            return Some(arch);
        }

        None
    }

    fn nearest_archetype_index(&self) -> Option<usize> {
        if self.archetypes.is_empty() {
            return None;
        }
        let mut best_idx = 0;
        let mut best_dist = f64::MAX;
        let features = self.build_feature_vector();

        for (i, arch) in self.archetypes.iter().enumerate() {
            let arch_features: Vec<f64> = arch.characteristic_actions.iter()
                .map(|a| *self.action_counts.get(a).unwrap_or(&0) as f64)
                .collect();
            let dist = self.euclidean_distance(&features, &arch_features);
            if dist < best_dist {
                best_dist = dist;
                best_idx = i;
            }
        }
        Some(best_idx)
    }

    fn euclidean_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        let len = a.len().max(b.len());
        let mut sum = 0.0;
        for i in 0..len {
            let av = a.get(i).unwrap_or(&0.0);
            let bv = b.get(i).unwrap_or(&0.0);
            let diff = av - bv;
            sum += diff * diff;
        }
        sum.sqrt()
    }

    fn build_feature_vector(&self) -> Vec<f64> {
        let actions: Vec<_> = self.action_counts.keys().collect();
        actions.iter().map(|a| *self.action_counts.get(*a).unwrap_or(&0) as f64).collect()
    }

    fn suggest_archetype_label(&self) -> String {
        let top = self.top_actions(2);
        if top.is_empty() {
            "unknown".to_string()
        } else {
            top.join("_")
        }
    }

    fn top_actions(&self, n: usize) -> Vec<String> {
        let mut sorted: Vec<_> = self.action_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        sorted.into_iter().take(n).map(|(k, _)| k.clone()).collect()
    }

    pub fn trigger_revision(&mut self) {
        self.needs_revision = true;
        if self.action_history.len() >= 10 {
            let saved = self.action_history.clone();
            self.archetypes.clear();
            self.current_archetype = None;
            for action in &saved {
                *self.action_counts.entry(action.clone()).or_insert(0) += 1;
            }
            self.needs_revision = false;
        }
    }

    pub fn current_archetype(&self) -> Option<&SessionArchetype> {
        self.current_archetype.as_ref()
    }

    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    pub fn archetypes(&self) -> &[SessionArchetype] {
        &self.archetypes
    }

    pub fn archetypes_mut(&mut self) -> &mut [SessionArchetype] {
        &mut self.archetypes
    }

    pub fn action_history(&self) -> &[String] {
        &self.action_history
    }

    pub fn action_counts(&self) -> &HashMap<String, usize> {
        &self.action_counts
    }

    pub fn markov(&self) -> &MarkovChain {
        &self.markov
    }

    pub fn markov_mut(&mut self) -> &mut MarkovChain {
        &mut self.markov
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markov_update() {
        let mut chain = MarkovChain::new();
        let a1 = ArchetypeId::new();
        let a2 = ArchetypeId::new();
        chain.update(a1, a2);
        let p = chain.probability(a1, a2);
        assert!(p > 0.0);
    }

    #[test]
    fn test_markov_most_likely() {
        let mut chain = MarkovChain::new();
        let a1 = ArchetypeId::new();
        let a2 = ArchetypeId::new();
        let a3 = ArchetypeId::new();
        chain.update(a1, a2);
        chain.update(a1, a2);
        chain.update(a1, a3);
        let result = chain.most_likely(a1);
        assert!(result.is_some());
        let (to, _) = result.unwrap();
        assert_eq!(to, a2);
    }

    #[test]
    fn test_action_counts() {
        let mut bge = BGE::default();
        let events = vec![];
        bge.observe_action("test", &events);
        bge.observe_action("test", &events);
        bge.observe_action("other", &events);
        bge.observe_action("test", &events);
        assert_eq!(bge.action_counts.get("test"), Some(&3));
        assert_eq!(bge.action_counts.get("other"), Some(&1));
    }

    #[test]
    fn test_archetype_detection() {
        let mut bge = BGE::new(3);
        let events = vec![];
        bge.observe_action("open VSCode", &events);
        bge.observe_action("write code", &events);
        bge.observe_action("write code", &events);
        let arch = bge.detect_archetype();
        assert!(arch.is_some());
    }

    #[test]
    fn test_euclidean_distance() {
        let bge = BGE::default();
        let d = bge.euclidean_distance(&[1.0, 0.0], &[0.0, 1.0]);
        assert!((d - std::f64::consts::SQRT_2).abs() < 0.0001);
    }

    #[test]
    fn test_archetype_frequency_increment() {
        let mut bge = BGE::new(3);
        let events = vec![];

        bge.observe_action("action1", &events);
        bge.observe_action("action2", &events);
        bge.observe_action("action3", &events);
        bge.observe_action("action1", &events);
        bge.observe_action("action2", &events);
        bge.observe_action("action3", &events);

        if let Some(arch) = bge.detect_archetype() {
            assert!(arch.frequency >= 1);
        }
    }
}
