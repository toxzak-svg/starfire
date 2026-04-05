//! Causal Graph — Directed graph of causal relationships
//!
//! Manages the causal knowledge graph with nodes as entities and
//! directed edges as causal relationships.

use super::{CausalEdge, CausalEdgeId};
use std::collections::{HashMap, HashSet};

/// Node in the causal graph
#[derive(Debug, Clone)]
pub struct CausalNode {
    pub id: String,
    pub properties: HashMap<String, String>,
    pub in_degree: usize,
    pub out_degree: usize,
}

impl CausalNode {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            properties: HashMap::new(),
            in_degree: 0,
            out_degree: 0,
        }
    }
}

/// Causal graph
#[derive(Debug, Clone)]
pub struct CausalGraph {
    nodes: HashMap<String, CausalNode>,
    edges: HashMap<CausalEdgeId, CausalEdge>,
    adjacency: HashMap<String, Vec<(String, CausalEdgeId)>>, // cause -> [(effect, edge_id)]
    reverse_adjacency: HashMap<String, Vec<(String, CausalEdgeId)>>, // effect -> [(cause, edge_id)]
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
        }
    }

    /// Add a node
    pub fn add_node(&mut self, id: &str) {
        if !self.nodes.contains_key(id) {
            self.nodes.insert(id.to_string(), CausalNode::new(id));
        }
    }

    /// Add an edge
    pub fn add_edge(&mut self, edge: CausalEdge) {
        let cause = &edge.cause;
        let effect = &edge.effect;

        // Ensure nodes exist
        self.add_node(cause);
        self.add_node(effect);

        // Add edge
        self.edges.insert(edge.id, edge.clone());

        // Update adjacency lists
        self.adjacency
            .entry(cause.clone())
            .or_default()
            .push((effect.clone(), edge.id));

        self.reverse_adjacency
            .entry(effect.clone())
            .or_default()
            .push((cause.clone(), edge.id));

        // Update degrees
        if let Some(node) = self.nodes.get_mut(cause) {
            node.out_degree += 1;
        }
        if let Some(node) = self.nodes.get_mut(effect) {
            node.in_degree += 1;
        }
    }

    /// Get outgoing edges from a node
    pub fn outgoing(&self, node: &str) -> Vec<&CausalEdge> {
        self.adjacency
            .get(node)
            .map(|edges| {
                edges
                    .iter()
                    .filter_map(|(_, id)| self.edges.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get incoming edges to a node
    pub fn incoming(&self, node: &str) -> Vec<&CausalEdge> {
        self.reverse_adjacency
            .get(node)
            .map(|edges| {
                edges
                    .iter()
                    .filter_map(|(_, id)| self.edges.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find all causal paths from source to target (up to max_depth)
    pub fn find_paths(&self, source: &str, target: &str, max_depth: usize) -> Vec<CausalPath> {
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        self.dfs_paths(source, target, max_depth, &mut visited, Vec::new(), &mut paths);
        paths
    }

    fn dfs_paths(
        &self,
        current: &str,
        target: &str,
        remaining: usize,
        visited: &mut HashSet<String>,
        path: Vec<String>,
        paths: &mut Vec<CausalPath>,
    ) {
        if remaining == 0 || visited.contains(current) {
            return;
        }

        let mut new_path = path.clone();
        new_path.push(current.to_string());

        if current == target {
            paths.push(CausalPath {
                nodes: new_path,
                confidence: 1.0, // Would need edge confidences
            });
            return;
        }

        visited.insert(current.to_string());

        if let Some(outgoing) = self.adjacency.get(current) {
            for (next, _) in outgoing {
                self.dfs_paths(next, target, remaining - 1, visited, new_path.clone(), paths);
            }
        }

        visited.remove(current);
    }

    /// Get top N hubs (nodes with most causal connections)
    pub fn top_hubs(&self, n: usize) -> Vec<&CausalNode> {
        let mut nodes: Vec<_> = self.nodes.values().collect();
        nodes.sort_by(|a, b| {
            (b.in_degree + b.out_degree)
                .cmp(&(a.in_degree + a.out_degree))
        });
        nodes.truncate(n);
        nodes
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if path exists
    pub fn path_exists(&self, source: &str, target: &str) -> bool {
        if !self.nodes.contains_key(source) || !self.nodes.contains_key(target) {
            return false;
        }
        let paths = self.find_paths(source, target, self.nodes.len());
        !paths.is_empty()
    }
}

/// A causal path through the graph
#[derive(Debug, Clone)]
pub struct CausalPath {
    pub nodes: Vec<String>,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_graph_add_edge() {
        let mut graph = CausalGraph::new();
        let edge = CausalEdge {
            id: CausalEdgeId::new(1),
            cause: "fire".to_string(),
            effect: "heat".to_string(),
            confidence: 0.9,
            evidence_count: 5,
            temporal_lag: Some(1),
            mechanism: None,
        };

        graph.add_edge(edge);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        let outgoing = graph.outgoing("fire");
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].effect, "heat");
    }

    #[test]
    fn test_path_exists() {
        let mut graph = CausalGraph::new();
        graph.add_edge(CausalEdge {
            id: CausalEdgeId::new(1),
            cause: "A".to_string(),
            effect: "B".to_string(),
            confidence: 0.9,
            evidence_count: 1,
            temporal_lag: None,
            mechanism: None,
        });
        graph.add_edge(CausalEdge {
            id: CausalEdgeId::new(2),
            cause: "B".to_string(),
            effect: "C".to_string(),
            confidence: 0.8,
            evidence_count: 1,
            temporal_lag: None,
            mechanism: None,
        });

        assert!(graph.path_exists("A", "C"));
        assert!(!graph.path_exists("C", "A")); // Reverse
    }

    #[test]
    fn test_find_paths() {
        let mut graph = CausalGraph::new();
        graph.add_edge(CausalEdge {
            id: CausalEdgeId::new(1),
            cause: "A".to_string(),
            effect: "B".to_string(),
            confidence: 0.9,
            evidence_count: 1,
            temporal_lag: None,
            mechanism: None,
        });
        graph.add_edge(CausalEdge {
            id: CausalEdgeId::new(2),
            cause: "B".to_string(),
            effect: "C".to_string(),
            confidence: 0.8,
            evidence_count: 1,
            temporal_lag: None,
            mechanism: None,
        });

        let paths = graph.find_paths("A", "C", 10);
        assert!(!paths.is_empty());
    }
}
