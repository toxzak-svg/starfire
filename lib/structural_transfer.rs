//! Proof-carrying structural-role transfer under bounded graph deformation.
//!
//! This module is research-executable and shadow-only. It can induce and
//! independently validate opaque transport-role certificates, then recognize
//! exact transfer-invariant signatures in held-out graphs. It cannot mutate
//! PECS, affect `Runtime::chat()`, route responses, promote ontology elements,
//! or acquire autonomous action authority.

use crate::commitment_state::Atom;
use crate::latent_roles::{DirectedEdge, RoleMember, StructuralCorpus, StructuralGraph};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct TransportRoleSignature {
    pub source_reachable: bool,
    pub sink_reachable: bool,
    pub upstream_sources_ge_2: bool,
    pub downstream_sinks_ge_2: bool,
    pub complete_reachable_pair_cut: bool,
    pub lost_reachable_pairs_ge_4: bool,
    pub has_convergent_ancestor: bool,
    pub has_divergent_ancestor: bool,
    pub has_convergent_descendant: bool,
    pub has_divergent_descendant: bool,
    pub upstream_depth_ge_2: bool,
    pub downstream_depth_ge_2: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransportRoleId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportRoleConfig {
    pub min_supporting_graphs: usize,
    pub exact_members_per_graph: usize,
}

impl Default for TransportRoleConfig {
    fn default() -> Self {
        Self {
            min_supporting_graphs: 4,
            exact_members_per_graph: 1,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportRoleBudget {
    pub graph_node_scans: usize,
    pub graph_edge_scans: usize,
    pub node_signature_evaluations: usize,
    pub reachability_edge_traversals: usize,
    pub removal_counterfactual_edge_traversals: usize,
    pub candidate_groups_considered: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportRoleProof {
    pub proof_id: u64,
    pub role_id: TransportRoleId,
    pub development_scope_digest: u64,
    pub signature: TransportRoleSignature,
    pub members: Vec<RoleMember>,
    pub supporting_graph_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedTransportRoleCertificate {
    proof_id: u64,
    role_id: TransportRoleId,
    development_scope_digest: u64,
    signature: TransportRoleSignature,
    member_count: usize,
    supporting_graph_count: usize,
}

impl ValidatedTransportRoleCertificate {
    #[must_use]
    pub const fn proof_id(&self) -> u64 {
        self.proof_id
    }

    #[must_use]
    pub const fn role_id(&self) -> TransportRoleId {
        self.role_id
    }

    #[must_use]
    pub const fn development_scope_digest(&self) -> u64 {
        self.development_scope_digest
    }

    #[must_use]
    pub const fn signature(&self) -> TransportRoleSignature {
        self.signature
    }

    #[must_use]
    pub const fn member_count(&self) -> usize {
        self.member_count
    }

    #[must_use]
    pub const fn supporting_graph_count(&self) -> usize {
        self.supporting_graph_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportRoleStatus {
    Candidate,
    ValidatedShadow,
    TransferValidated,
    ExecutableShadow,
    Rejected,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportRegistryEntry {
    pub role_id: TransportRoleId,
    pub proof_id: u64,
    pub development_scope_digest: u64,
    pub status: TransportRoleStatus,
}

/// Scope-bound shadow registry. There is deliberately no production `Live`
/// state and no method that mutates executable reasoning state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowTransportRoleRegistry {
    development_scope_digest: u64,
    entries: BTreeMap<TransportRoleId, TransportRegistryEntry>,
}

impl ShadowTransportRoleRegistry {
    #[must_use]
    pub fn new(development_scope_digest: u64) -> Self {
        Self {
            development_scope_digest,
            entries: BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn development_scope_digest(&self) -> u64 {
        self.development_scope_digest
    }

    pub fn register_candidate(
        &mut self,
        proof: &TransportRoleProof,
    ) -> Result<(), StructuralTransferError> {
        self.require_scope(proof.development_scope_digest)?;
        match self.entries.get(&proof.role_id) {
            Some(existing) if existing.proof_id != proof.proof_id => {
                return Err(StructuralTransferError::RegistryConflict(proof.role_id));
            }
            Some(_) => return Ok(()),
            None => {}
        }
        self.entries.insert(
            proof.role_id,
            TransportRegistryEntry {
                role_id: proof.role_id,
                proof_id: proof.proof_id,
                development_scope_digest: proof.development_scope_digest,
                status: TransportRoleStatus::Candidate,
            },
        );
        Ok(())
    }

    pub fn admit_validated(
        &mut self,
        certificate: &ValidatedTransportRoleCertificate,
    ) -> Result<(), StructuralTransferError> {
        self.require_scope(certificate.development_scope_digest)?;
        let entry = self
            .entries
            .get_mut(&certificate.role_id)
            .ok_or(StructuralTransferError::UnregisteredRole(certificate.role_id))?;
        if entry.proof_id != certificate.proof_id {
            return Err(StructuralTransferError::RegistryProofMismatch(
                certificate.role_id,
            ));
        }
        entry.status = TransportRoleStatus::ValidatedShadow;
        Ok(())
    }

    pub fn mark_transfer_validated(
        &mut self,
        certificate: &ValidatedTransportRoleCertificate,
    ) -> Result<(), StructuralTransferError> {
        self.require_usable(certificate)?;
        self.entries
            .get_mut(&certificate.role_id)
            .expect("validated entry exists")
            .status = TransportRoleStatus::TransferValidated;
        Ok(())
    }

    pub fn mark_executable_shadow(
        &mut self,
        certificate: &ValidatedTransportRoleCertificate,
    ) -> Result<(), StructuralTransferError> {
        self.require_usable(certificate)?;
        self.entries
            .get_mut(&certificate.role_id)
            .expect("validated entry exists")
            .status = TransportRoleStatus::ExecutableShadow;
        Ok(())
    }

    #[must_use]
    pub fn status(&self, role_id: TransportRoleId) -> Option<TransportRoleStatus> {
        self.entries.get(&role_id).map(|entry| entry.status)
    }

    fn require_scope(&self, scope: u64) -> Result<(), StructuralTransferError> {
        if scope != self.development_scope_digest {
            return Err(StructuralTransferError::ForeignDevelopmentScope {
                expected: self.development_scope_digest,
                actual: scope,
            });
        }
        Ok(())
    }

    fn require_usable(
        &self,
        certificate: &ValidatedTransportRoleCertificate,
    ) -> Result<(), StructuralTransferError> {
        self.require_scope(certificate.development_scope_digest)?;
        let entry = self
            .entries
            .get(&certificate.role_id)
            .ok_or(StructuralTransferError::UnregisteredRole(
                certificate.role_id,
            ))?;
        if entry.proof_id != certificate.proof_id {
            return Err(StructuralTransferError::RegistryProofMismatch(
                certificate.role_id,
            ));
        }
        match entry.status {
            TransportRoleStatus::ValidatedShadow
            | TransportRoleStatus::TransferValidated
            | TransportRoleStatus::ExecutableShadow => Ok(()),
            status => Err(StructuralTransferError::RoleNotValidated {
                role_id: certificate.role_id,
                status,
            }),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StructuralTransferError {
    #[error("development corpus is empty")]
    EmptyCorpus,
    #[error("structural graph {0} has no nodes")]
    EmptyGraph(u64),
    #[error("duplicate structural graph id {0}")]
    DuplicateGraphId(u64),
    #[error("graph {graph_id} contains duplicate node {node}")]
    DuplicateNode { graph_id: u64, node: String },
    #[error("graph {graph_id} contains duplicate edge {from} -> {to}")]
    DuplicateEdge {
        graph_id: u64,
        from: String,
        to: String,
    },
    #[error("graph {graph_id} contains self edge at {node}")]
    SelfEdge { graph_id: u64, node: String },
    #[error("graph {graph_id} edge references absent endpoint {node}")]
    UnknownEndpoint { graph_id: u64, node: String },
    #[error("transport-role induction produced no recurring groups")]
    NoRecurringRoles,
    #[error("transport-role proof mismatch: {0}")]
    ProofMismatch(&'static str),
    #[error("transport role id collision for {0:?}")]
    RoleIdCollision(TransportRoleId),
    #[error("registry conflict for transport role {0:?}")]
    RegistryConflict(TransportRoleId),
    #[error("transport role {0:?} is not registered")]
    UnregisteredRole(TransportRoleId),
    #[error("registry proof mismatch for transport role {0:?}")]
    RegistryProofMismatch(TransportRoleId),
    #[error("foreign development scope: expected {expected}, got {actual}")]
    ForeignDevelopmentScope { expected: u64, actual: u64 },
    #[error("transport role {role_id:?} is not validated; status is {status:?}")]
    RoleNotValidated {
        role_id: TransportRoleId,
        status: TransportRoleStatus,
    },
}

pub fn development_scope_digest(
    corpus: &StructuralCorpus,
) -> Result<u64, StructuralTransferError> {
    validate_corpus(corpus, &mut TransportRoleBudget::default())?;
    Ok(corpus_digest(corpus))
}

pub fn induce_transport_roles(
    corpus: &StructuralCorpus,
    config: TransportRoleConfig,
    budget: &mut TransportRoleBudget,
) -> Result<Vec<TransportRoleProof>, StructuralTransferError> {
    validate_corpus(corpus, budget)?;
    let scope_digest = corpus_digest(corpus);
    let groups = signature_groups(corpus, budget)?;
    let mut proofs = Vec::new();
    let mut seen_ids = BTreeMap::<TransportRoleId, TransportRoleSignature>::new();

    for (signature, mut members) in groups {
        budget.candidate_groups_considered = budget.candidate_groups_considered.saturating_add(1);
        members.sort();
        let per_graph = member_counts(&members);
        let supporting_graph_ids = per_graph.keys().copied().collect::<Vec<_>>();
        if supporting_graph_ids.len() < config.min_supporting_graphs
            || per_graph
                .values()
                .any(|count| *count != config.exact_members_per_graph)
        {
            continue;
        }

        let role_id = role_id_for_signature(signature);
        if let Some(previous) = seen_ids.insert(role_id, signature) {
            if previous != signature {
                return Err(StructuralTransferError::RoleIdCollision(role_id));
            }
        }
        let proof_id = proof_digest(
            scope_digest,
            role_id,
            signature,
            &members,
            &supporting_graph_ids,
        );
        proofs.push(TransportRoleProof {
            proof_id,
            role_id,
            development_scope_digest: scope_digest,
            signature,
            members,
            supporting_graph_ids,
        });
    }

    if proofs.is_empty() {
        return Err(StructuralTransferError::NoRecurringRoles);
    }
    proofs.sort_by_key(|proof| proof.role_id);
    Ok(proofs)
}

pub fn validate_transport_role(
    corpus: &StructuralCorpus,
    proof: &TransportRoleProof,
    config: TransportRoleConfig,
    budget: &mut TransportRoleBudget,
) -> Result<ValidatedTransportRoleCertificate, StructuralTransferError> {
    validate_corpus(corpus, budget)?;
    let scope_digest = corpus_digest(corpus);
    let groups = signature_groups(corpus, budget)?;
    budget.candidate_groups_considered = groups.len();

    let mut members = groups
        .get(&proof.signature)
        .cloned()
        .ok_or(StructuralTransferError::ProofMismatch("signature"))?;
    members.sort();
    let per_graph = member_counts(&members);
    let supporting_graph_ids = per_graph.keys().copied().collect::<Vec<_>>();
    if supporting_graph_ids.len() < config.min_supporting_graphs
        || per_graph
            .values()
            .any(|count| *count != config.exact_members_per_graph)
    {
        return Err(StructuralTransferError::ProofMismatch("recurrence_gate"));
    }

    let role_id = role_id_for_signature(proof.signature);
    let proof_id = proof_digest(
        scope_digest,
        role_id,
        proof.signature,
        &members,
        &supporting_graph_ids,
    );

    if proof.development_scope_digest != scope_digest {
        return Err(StructuralTransferError::ProofMismatch(
            "development_scope_digest",
        ));
    }
    if proof.role_id != role_id {
        return Err(StructuralTransferError::ProofMismatch("role_id"));
    }
    if proof.members != members {
        return Err(StructuralTransferError::ProofMismatch("members"));
    }
    if proof.supporting_graph_ids != supporting_graph_ids {
        return Err(StructuralTransferError::ProofMismatch(
            "supporting_graph_ids",
        ));
    }
    if proof.proof_id != proof_id {
        return Err(StructuralTransferError::ProofMismatch("proof_id"));
    }

    Ok(ValidatedTransportRoleCertificate {
        proof_id,
        role_id,
        development_scope_digest: scope_digest,
        signature: proof.signature,
        member_count: members.len(),
        supporting_graph_count: supporting_graph_ids.len(),
    })
}

pub fn recognize_transport_role(
    graph: &StructuralGraph,
    certificate: &ValidatedTransportRoleCertificate,
    registry: &ShadowTransportRoleRegistry,
    budget: &mut TransportRoleBudget,
) -> Result<Vec<Atom>, StructuralTransferError> {
    registry.require_usable(certificate)?;
    validate_graph(graph, budget)?;
    let index = GraphIndex::new(graph);
    let mut matches = Vec::new();
    for node in &graph.nodes {
        budget.node_signature_evaluations = budget.node_signature_evaluations.saturating_add(1);
        if compute_signature(node, &index, budget) == certificate.signature {
            matches.push(node.clone());
        }
    }
    matches.sort();
    Ok(matches)
}

fn member_counts(members: &[RoleMember]) -> BTreeMap<u64, usize> {
    let mut counts = BTreeMap::new();
    for member in members {
        *counts.entry(member.graph_id).or_insert(0) += 1;
    }
    counts
}

fn signature_groups(
    corpus: &StructuralCorpus,
    budget: &mut TransportRoleBudget,
) -> Result<BTreeMap<TransportRoleSignature, Vec<RoleMember>>, StructuralTransferError> {
    let mut groups = BTreeMap::<TransportRoleSignature, Vec<RoleMember>>::new();
    for graph in &corpus.graphs {
        let index = GraphIndex::new(graph);
        for node in &graph.nodes {
            budget.node_signature_evaluations =
                budget.node_signature_evaluations.saturating_add(1);
            let signature = compute_signature(node, &index, budget);
            groups.entry(signature).or_default().push(RoleMember {
                graph_id: graph.graph_id,
                node: node.clone(),
            });
        }
    }
    Ok(groups)
}

fn validate_corpus(
    corpus: &StructuralCorpus,
    budget: &mut TransportRoleBudget,
) -> Result<(), StructuralTransferError> {
    if corpus.graphs.is_empty() {
        return Err(StructuralTransferError::EmptyCorpus);
    }
    let mut ids = BTreeSet::new();
    for graph in &corpus.graphs {
        if !ids.insert(graph.graph_id) {
            return Err(StructuralTransferError::DuplicateGraphId(graph.graph_id));
        }
        validate_graph(graph, budget)?;
    }
    Ok(())
}

fn validate_graph(
    graph: &StructuralGraph,
    budget: &mut TransportRoleBudget,
) -> Result<(), StructuralTransferError> {
    if graph.nodes.is_empty() {
        return Err(StructuralTransferError::EmptyGraph(graph.graph_id));
    }
    let mut nodes = BTreeSet::new();
    for node in &graph.nodes {
        budget.graph_node_scans = budget.graph_node_scans.saturating_add(1);
        if !nodes.insert(node.clone()) {
            return Err(StructuralTransferError::DuplicateNode {
                graph_id: graph.graph_id,
                node: node.to_string(),
            });
        }
    }
    let mut edges = BTreeSet::new();
    for edge in &graph.edges {
        budget.graph_edge_scans = budget.graph_edge_scans.saturating_add(1);
        if edge.from == edge.to {
            return Err(StructuralTransferError::SelfEdge {
                graph_id: graph.graph_id,
                node: edge.from.to_string(),
            });
        }
        for endpoint in [&edge.from, &edge.to] {
            if !nodes.contains(endpoint) {
                return Err(StructuralTransferError::UnknownEndpoint {
                    graph_id: graph.graph_id,
                    node: endpoint.to_string(),
                });
            }
        }
        if !edges.insert(edge.clone()) {
            return Err(StructuralTransferError::DuplicateEdge {
                graph_id: graph.graph_id,
                from: edge.from.to_string(),
                to: edge.to.to_string(),
            });
        }
    }
    Ok(())
}

struct GraphIndex {
    forward: BTreeMap<Atom, Vec<Atom>>,
    reverse: BTreeMap<Atom, Vec<Atom>>,
    in_degree: BTreeMap<Atom, usize>,
    out_degree: BTreeMap<Atom, usize>,
    sources: Vec<Atom>,
    sinks: Vec<Atom>,
}

impl GraphIndex {
    fn new(graph: &StructuralGraph) -> Self {
        let mut forward = graph
            .nodes
            .iter()
            .cloned()
            .map(|node| (node, Vec::new()))
            .collect::<BTreeMap<_, _>>();
        let mut reverse = forward.clone();
        for DirectedEdge { from, to } in &graph.edges {
            forward.get_mut(from).expect("known endpoint").push(to.clone());
            reverse.get_mut(to).expect("known endpoint").push(from.clone());
        }
        for neighbors in forward.values_mut() {
            neighbors.sort();
        }
        for neighbors in reverse.values_mut() {
            neighbors.sort();
        }
        let in_degree = graph
            .nodes
            .iter()
            .cloned()
            .map(|node| {
                let degree = reverse.get(&node).map_or(0, Vec::len);
                (node, degree)
            })
            .collect::<BTreeMap<_, _>>();
        let out_degree = graph
            .nodes
            .iter()
            .cloned()
            .map(|node| {
                let degree = forward.get(&node).map_or(0, Vec::len);
                (node, degree)
            })
            .collect::<BTreeMap<_, _>>();
        let sources = graph
            .nodes
            .iter()
            .filter(|node| in_degree.get(*node) == Some(&0))
            .cloned()
            .collect();
        let sinks = graph
            .nodes
            .iter()
            .filter(|node| out_degree.get(*node) == Some(&0))
            .cloned()
            .collect();
        Self {
            forward,
            reverse,
            in_degree,
            out_degree,
            sources,
            sinks,
        }
    }
}

fn compute_signature(
    node: &Atom,
    index: &GraphIndex,
    budget: &mut TransportRoleBudget,
) -> TransportRoleSignature {
    let ancestors = reachable_set(node, &index.reverse, None, budget, false);
    let descendants = reachable_set(node, &index.forward, None, budget, false);
    let upstream_sources = index
        .sources
        .iter()
        .filter(|source| ancestors.contains(*source))
        .cloned()
        .collect::<Vec<_>>();
    let downstream_sinks = index
        .sinks
        .iter()
        .filter(|sink| descendants.contains(*sink))
        .cloned()
        .collect::<Vec<_>>();

    let mut lost_pairs = 0usize;
    let total_pairs = upstream_sources
        .len()
        .saturating_mul(downstream_sinks.len());
    for source in &upstream_sources {
        for sink in &downstream_sinks {
            if !path_exists(
                source,
                sink,
                Some(node),
                &index.forward,
                budget,
                true,
            ) {
                lost_pairs = lost_pairs.saturating_add(1);
            }
        }
    }

    let has_convergent_ancestor = ancestors
        .iter()
        .any(|candidate| index.in_degree.get(candidate).copied().unwrap_or(0) >= 2);
    let has_divergent_ancestor = ancestors
        .iter()
        .any(|candidate| index.out_degree.get(candidate).copied().unwrap_or(0) >= 2);
    let has_convergent_descendant = descendants
        .iter()
        .any(|candidate| index.in_degree.get(candidate).copied().unwrap_or(0) >= 2);
    let has_divergent_descendant = descendants
        .iter()
        .any(|candidate| index.out_degree.get(candidate).copied().unwrap_or(0) >= 2);

    let upstream_depth = shortest_distance_from_any(&upstream_sources, node, &index.forward, budget);
    let downstream_depth = shortest_distance_to_any(node, &downstream_sinks, &index.forward, budget);

    TransportRoleSignature {
        source_reachable: !upstream_sources.is_empty(),
        sink_reachable: !downstream_sinks.is_empty(),
        upstream_sources_ge_2: upstream_sources.len() >= 2,
        downstream_sinks_ge_2: downstream_sinks.len() >= 2,
        complete_reachable_pair_cut: total_pairs > 0 && lost_pairs == total_pairs,
        lost_reachable_pairs_ge_4: lost_pairs >= 4,
        has_convergent_ancestor,
        has_divergent_ancestor,
        has_convergent_descendant,
        has_divergent_descendant,
        upstream_depth_ge_2: upstream_depth.is_some_and(|distance| distance >= 2),
        downstream_depth_ge_2: downstream_depth.is_some_and(|distance| distance >= 2),
    }
}

fn reachable_set(
    start: &Atom,
    adjacency: &BTreeMap<Atom, Vec<Atom>>,
    blocked: Option<&Atom>,
    budget: &mut TransportRoleBudget,
    counterfactual: bool,
) -> BTreeSet<Atom> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(current) = queue.pop_front() {
        if blocked == Some(&current) || !visited.insert(current.clone()) {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&current) {
            for neighbor in neighbors {
                if counterfactual {
                    budget.removal_counterfactual_edge_traversals = budget
                        .removal_counterfactual_edge_traversals
                        .saturating_add(1);
                } else {
                    budget.reachability_edge_traversals =
                        budget.reachability_edge_traversals.saturating_add(1);
                }
                if blocked != Some(neighbor) && !visited.contains(neighbor) {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }
    visited.remove(start);
    visited
}

fn path_exists(
    start: &Atom,
    goal: &Atom,
    blocked: Option<&Atom>,
    adjacency: &BTreeMap<Atom, Vec<Atom>>,
    budget: &mut TransportRoleBudget,
    counterfactual: bool,
) -> bool {
    if blocked == Some(start) || blocked == Some(goal) {
        return false;
    }
    if start == goal {
        return true;
    }
    let reachable = reachable_set(start, adjacency, blocked, budget, counterfactual);
    reachable.contains(goal)
}

fn shortest_distance_from_any(
    starts: &[Atom],
    goal: &Atom,
    adjacency: &BTreeMap<Atom, Vec<Atom>>,
    budget: &mut TransportRoleBudget,
) -> Option<usize> {
    starts
        .iter()
        .filter_map(|start| shortest_distance(start, goal, adjacency, budget))
        .min()
}

fn shortest_distance_to_any(
    start: &Atom,
    goals: &[Atom],
    adjacency: &BTreeMap<Atom, Vec<Atom>>,
    budget: &mut TransportRoleBudget,
) -> Option<usize> {
    goals
        .iter()
        .filter_map(|goal| shortest_distance(start, goal, adjacency, budget))
        .min()
}

fn shortest_distance(
    start: &Atom,
    goal: &Atom,
    adjacency: &BTreeMap<Atom, Vec<Atom>>,
    budget: &mut TransportRoleBudget,
) -> Option<usize> {
    let mut visited = BTreeSet::from([start.clone()]);
    let mut queue = VecDeque::from([(start.clone(), 0usize)]);
    while let Some((current, distance)) = queue.pop_front() {
        if &current == goal {
            return Some(distance);
        }
        if let Some(neighbors) = adjacency.get(&current) {
            for neighbor in neighbors {
                budget.reachability_edge_traversals =
                    budget.reachability_edge_traversals.saturating_add(1);
                if visited.insert(neighbor.clone()) {
                    queue.push_back((neighbor.clone(), distance.saturating_add(1)));
                }
            }
        }
    }
    None
}

fn role_id_for_signature(signature: TransportRoleSignature) -> TransportRoleId {
    let mut hash = Fnv64::new();
    hash.push_u64(signature_bits(signature));
    TransportRoleId(hash.finish())
}

fn corpus_digest(corpus: &StructuralCorpus) -> u64 {
    let mut graphs = corpus.graphs.clone();
    graphs.sort_by_key(|graph| graph.graph_id);
    let mut hash = Fnv64::new();
    for graph in graphs {
        hash.push_u64(graph.graph_id);
        let mut nodes = graph.nodes;
        nodes.sort();
        for node in nodes {
            hash.push_str(&node.to_string());
        }
        let mut edges = graph.edges;
        edges.sort();
        for edge in edges {
            hash.push_str(&edge.from.to_string());
            hash.push_str(&edge.to.to_string());
        }
    }
    hash.finish()
}

fn proof_digest(
    scope_digest: u64,
    role_id: TransportRoleId,
    signature: TransportRoleSignature,
    members: &[RoleMember],
    supporting_graph_ids: &[u64],
) -> u64 {
    let mut hash = Fnv64::new();
    hash.push_u64(scope_digest);
    hash.push_u64(role_id.0);
    hash.push_u64(signature_bits(signature));
    for member in members {
        hash.push_u64(member.graph_id);
        hash.push_str(&member.node.to_string());
    }
    for graph_id in supporting_graph_ids {
        hash.push_u64(*graph_id);
    }
    hash.finish()
}

fn signature_bits(signature: TransportRoleSignature) -> u64 {
    let values = [
        signature.source_reachable,
        signature.sink_reachable,
        signature.upstream_sources_ge_2,
        signature.downstream_sinks_ge_2,
        signature.complete_reachable_pair_cut,
        signature.lost_reachable_pairs_ge_4,
        signature.has_convergent_ancestor,
        signature.has_divergent_ancestor,
        signature.has_convergent_descendant,
        signature.has_divergent_descendant,
        signature.upstream_depth_ge_2,
        signature.downstream_depth_ge_2,
    ];
    values
        .into_iter()
        .enumerate()
        .fold(0u64, |bits, (index, value)| {
            bits | (u64::from(value) << index)
        })
}

struct Fnv64(u64);

impl Fnv64 {
    const fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn push_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.push_byte(byte);
        }
    }

    fn push_str(&mut self, value: &str) {
        for byte in value.as_bytes() {
            self.push_byte(*byte);
        }
        self.push_byte(0xff);
    }

    fn push_byte(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recurring_cut_role_is_induced_and_independently_validated() {
        let corpus = StructuralCorpus {
            graphs: (0..4)
                .map(|index| development_graph(index + 1, &format!("d{index}")))
                .collect(),
        };
        let config = TransportRoleConfig::default();
        let mut induction_budget = TransportRoleBudget::default();
        let proofs = induce_transport_roles(&corpus, config, &mut induction_budget).unwrap();
        let target = proofs
            .iter()
            .find(|proof| proof.signature.complete_reachable_pair_cut)
            .expect("cut role is induced");
        let mut validation_budget = TransportRoleBudget::default();
        let certificate =
            validate_transport_role(&corpus, target, config, &mut validation_budget).unwrap();
        assert_eq!(certificate.member_count(), 4);
        assert_eq!(certificate.supporting_graph_count(), 4);
    }

    #[test]
    fn transfer_recognition_survives_subdivision_and_rejects_bypass_decoy() {
        let corpus = StructuralCorpus {
            graphs: (0..4)
                .map(|index| development_graph(index + 1, &format!("d{index}")))
                .collect(),
        };
        let config = TransportRoleConfig::default();
        let mut budget = TransportRoleBudget::default();
        let proofs = induce_transport_roles(&corpus, config, &mut budget).unwrap();
        let target = proofs
            .iter()
            .find(|proof| {
                proof.signature.complete_reachable_pair_cut
                    && proof.signature.has_convergent_ancestor
                    && proof.signature.has_divergent_descendant
            })
            .unwrap();
        let mut validation_budget = TransportRoleBudget::default();
        let certificate =
            validate_transport_role(&corpus, target, config, &mut validation_budget).unwrap();
        let mut registry =
            ShadowTransportRoleRegistry::new(development_scope_digest(&corpus).unwrap());
        registry.register_candidate(target).unwrap();
        registry.admit_validated(&certificate).unwrap();

        let (graph, bridge, bypass) = transfer_graph(99, "future");
        let mut recognition_budget = TransportRoleBudget::default();
        let matches = recognize_transport_role(
            &graph,
            &certificate,
            &registry,
            &mut recognition_budget,
        )
        .unwrap();
        assert_eq!(matches, vec![bridge]);
        assert!(!matches.contains(&bypass));
    }

    #[test]
    fn tampered_member_is_rejected() {
        let corpus = StructuralCorpus {
            graphs: (0..4)
                .map(|index| development_graph(index + 1, &format!("d{index}")))
                .collect(),
        };
        let config = TransportRoleConfig::default();
        let mut budget = TransportRoleBudget::default();
        let mut proof = induce_transport_roles(&corpus, config, &mut budget)
            .unwrap()
            .into_iter()
            .find(|proof| proof.signature.complete_reachable_pair_cut)
            .unwrap();
        proof.members[0].node = atom("forged");
        let mut validation_budget = TransportRoleBudget::default();
        assert!(validate_transport_role(&corpus, &proof, config, &mut validation_budget).is_err());
    }

    fn development_graph(graph_id: u64, prefix: &str) -> StructuralGraph {
        let mut builder = Builder::new(graph_id);
        let join = atom(format!("{prefix}_join"));
        let bridge = atom(format!("{prefix}_bridge"));
        let split = atom(format!("{prefix}_split"));
        builder.edge(atom(format!("{prefix}_s0")), join.clone());
        builder.edge(atom(format!("{prefix}_s1")), join.clone());
        builder.edge(join, bridge.clone());
        builder.edge(bridge, split.clone());
        builder.edge(split.clone(), atom(format!("{prefix}_t0")));
        builder.edge(split, atom(format!("{prefix}_t1")));
        builder.finish()
    }

    fn transfer_graph(graph_id: u64, prefix: &str) -> (StructuralGraph, Atom, Atom) {
        let mut builder = Builder::new(graph_id);
        let join = atom(format!("{prefix}_join"));
        let u1 = atom(format!("{prefix}_u1"));
        let bridge = atom(format!("{prefix}_bridge"));
        let d1 = atom(format!("{prefix}_d1"));
        let split = atom(format!("{prefix}_split"));
        builder.edge(atom(format!("{prefix}_s0")), join.clone());
        builder.edge(atom(format!("{prefix}_s1")), join.clone());
        builder.edge(join, u1.clone());
        builder.edge(u1, bridge.clone());
        builder.edge(bridge.clone(), d1.clone());
        builder.edge(d1, split.clone());
        builder.edge(split.clone(), atom(format!("{prefix}_t0")));
        builder.edge(split, atom(format!("{prefix}_t1")));

        let bypass = atom(format!("{prefix}_decoy"));
        let before = atom(format!("{prefix}_decoy_before"));
        let after = atom(format!("{prefix}_decoy_after"));
        builder.edge(atom(format!("{prefix}_ds0")), before.clone());
        builder.edge(atom(format!("{prefix}_ds1")), before.clone());
        builder.edge(before.clone(), bypass.clone());
        builder.edge(bypass.clone(), after.clone());
        builder.edge(before, after.clone());
        builder.edge(after.clone(), atom(format!("{prefix}_dt0")));
        builder.edge(after, atom(format!("{prefix}_dt1")));
        (builder.finish(), bridge, bypass)
    }

    struct Builder {
        graph_id: u64,
        nodes: BTreeSet<Atom>,
        edges: BTreeSet<DirectedEdge>,
    }

    impl Builder {
        fn new(graph_id: u64) -> Self {
            Self {
                graph_id,
                nodes: BTreeSet::new(),
                edges: BTreeSet::new(),
            }
        }

        fn edge(&mut self, from: Atom, to: Atom) {
            self.nodes.insert(from.clone());
            self.nodes.insert(to.clone());
            self.edges.insert(DirectedEdge { from, to });
        }

        fn finish(self) -> StructuralGraph {
            StructuralGraph {
                graph_id: self.graph_id,
                nodes: self.nodes.into_iter().collect(),
                edges: self.edges.into_iter().collect(),
            }
        }
    }

    fn atom(value: impl Into<String>) -> Atom {
        Atom::new(value.into()).expect("generated atom is non-empty")
    }
}
