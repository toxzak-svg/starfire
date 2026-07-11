//! Proof-carrying latent structural roles for the H12 shadow experiment.
//!
//! H12 deliberately keeps representation invention separate from executable
//! trust. This module may discover and independently validate recurring unnamed
//! structural roles, recognize them in held-out graphs, and use a validated role
//! to project raw evidence. It cannot directly mutate PECS. Any executable rule
//! derived from projected evidence must still pass the unchanged H11/H10 proof
//! and admission path.

use crate::commitment_state::Atom;
use crate::graph_discovery::MixedEvidenceGraph;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DirectedEdge {
    pub from: Atom,
    pub to: Atom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralGraph {
    pub graph_id: u64,
    pub nodes: Vec<Atom>,
    pub edges: Vec<DirectedEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralCorpus {
    pub graphs: Vec<StructuralGraph>,
}

/// A target-blind, deterministic structural signature.
///
/// The frozen H12 fingerprint intentionally contains no task label, objective,
/// intervention outcome, PECS state, human role name, or future-family identity.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct StructuralFingerprint {
    pub in_degree: u16,
    pub out_degree: u16,
    pub predecessor_convergence_count: u16,
    pub predecessor_divergence_count: u16,
    pub successor_convergence_count: u16,
    pub successor_divergence_count: u16,
    pub has_convergent_ancestor: bool,
    pub has_divergent_ancestor: bool,
    pub has_convergent_descendant: bool,
    pub has_divergent_descendant: bool,
    pub source_reachable: bool,
    pub sink_reachable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoleMember {
    pub graph_id: u64,
    pub node: Atom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleInductionConfig {
    pub min_supporting_graphs: usize,
    pub min_members: usize,
}

impl Default for RoleInductionConfig {
    fn default() -> Self {
        Self {
            min_supporting_graphs: 4,
            min_members: 4,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleInductionBudget {
    pub node_fingerprint_evaluations: usize,
    pub edge_traversals: usize,
    pub candidate_groups_considered: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferRecognitionBudget {
    pub node_fingerprint_evaluations: usize,
    pub edge_traversals: usize,
    pub evidence_episode_scans: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatentRoleProof {
    pub proof_id: u64,
    pub role_id: RoleId,
    pub discovery_scope_digest: u64,
    pub structural_fingerprint: StructuralFingerprint,
    pub members: Vec<RoleMember>,
    pub supporting_graph_ids: Vec<u64>,
}

/// Opaque proof-carrying shadow abstraction. Callers may inspect identity and
/// invariants but cannot construct a trusted certificate without independent
/// validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedLatentRoleCertificate {
    proof_id: u64,
    role_id: RoleId,
    discovery_scope_digest: u64,
    structural_fingerprint: StructuralFingerprint,
    member_count: usize,
    supporting_graph_count: usize,
}

impl ValidatedLatentRoleCertificate {
    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn role_id(&self) -> RoleId {
        self.role_id
    }

    pub fn discovery_scope_digest(&self) -> u64 {
        self.discovery_scope_digest
    }

    pub fn structural_fingerprint(&self) -> StructuralFingerprint {
        self.structural_fingerprint
    }

    pub fn member_count(&self) -> usize {
        self.member_count
    }

    pub fn supporting_graph_count(&self) -> usize {
        self.supporting_graph_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbstractionStatus {
    Candidate,
    ValidatedShadow,
    TransferValidated,
    ExecutableShadow,
    Rejected,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub role_id: RoleId,
    pub proof_id: u64,
    pub discovery_scope_digest: u64,
    pub status: AbstractionStatus,
}

/// A scope-bound shadow registry. There is deliberately no `Live` state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowAbstractionRegistry {
    discovery_scope_digest: u64,
    entries: BTreeMap<RoleId, RegistryEntry>,
}

impl ShadowAbstractionRegistry {
    pub fn new(discovery_scope_digest: u64) -> Self {
        Self {
            discovery_scope_digest,
            entries: BTreeMap::new(),
        }
    }

    pub fn discovery_scope_digest(&self) -> u64 {
        self.discovery_scope_digest
    }

    pub fn register_candidate(&mut self, proof: &LatentRoleProof) -> Result<(), LatentRoleError> {
        self.require_scope(proof.discovery_scope_digest)?;
        match self.entries.get(&proof.role_id) {
            Some(existing) if existing.proof_id != proof.proof_id => {
                return Err(LatentRoleError::RegistryConflict(proof.role_id));
            }
            Some(_) => return Ok(()),
            None => {}
        }
        self.entries.insert(
            proof.role_id,
            RegistryEntry {
                role_id: proof.role_id,
                proof_id: proof.proof_id,
                discovery_scope_digest: proof.discovery_scope_digest,
                status: AbstractionStatus::Candidate,
            },
        );
        Ok(())
    }

    pub fn admit_validated(
        &mut self,
        certificate: &ValidatedLatentRoleCertificate,
    ) -> Result<(), LatentRoleError> {
        self.require_scope(certificate.discovery_scope_digest)?;
        let entry = self
            .entries
            .get_mut(&certificate.role_id)
            .ok_or(LatentRoleError::UnregisteredRole(certificate.role_id))?;
        if entry.proof_id != certificate.proof_id {
            return Err(LatentRoleError::RegistryProofMismatch(certificate.role_id));
        }
        entry.status = AbstractionStatus::ValidatedShadow;
        Ok(())
    }

    pub fn mark_transfer_validated(
        &mut self,
        certificate: &ValidatedLatentRoleCertificate,
    ) -> Result<(), LatentRoleError> {
        self.require_usable(certificate)?;
        let entry = self.entries.get_mut(&certificate.role_id).unwrap();
        entry.status = AbstractionStatus::TransferValidated;
        Ok(())
    }

    pub fn mark_executable_shadow(
        &mut self,
        certificate: &ValidatedLatentRoleCertificate,
    ) -> Result<(), LatentRoleError> {
        self.require_usable(certificate)?;
        let entry = self.entries.get_mut(&certificate.role_id).unwrap();
        entry.status = AbstractionStatus::ExecutableShadow;
        Ok(())
    }

    pub fn status(&self, role_id: RoleId) -> Option<AbstractionStatus> {
        self.entries.get(&role_id).map(|entry| entry.status)
    }

    pub fn entries(&self) -> impl Iterator<Item = &RegistryEntry> {
        self.entries.values()
    }

    fn require_scope(&self, scope: u64) -> Result<(), LatentRoleError> {
        if scope != self.discovery_scope_digest {
            return Err(LatentRoleError::ForeignDiscoveryScope {
                expected: self.discovery_scope_digest,
                actual: scope,
            });
        }
        Ok(())
    }

    fn require_usable(
        &self,
        certificate: &ValidatedLatentRoleCertificate,
    ) -> Result<(), LatentRoleError> {
        self.require_scope(certificate.discovery_scope_digest)?;
        let entry = self
            .entries
            .get(&certificate.role_id)
            .ok_or(LatentRoleError::UnregisteredRole(certificate.role_id))?;
        if entry.proof_id != certificate.proof_id {
            return Err(LatentRoleError::RegistryProofMismatch(certificate.role_id));
        }
        match entry.status {
            AbstractionStatus::ValidatedShadow
            | AbstractionStatus::TransferValidated
            | AbstractionStatus::ExecutableShadow => Ok(()),
            status => Err(LatentRoleError::RoleNotValidated {
                role_id: certificate.role_id,
                status,
            }),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LatentRoleError {
    #[error("structural corpus is empty")]
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
    #[error("role induction produced no recurring structural groups")]
    NoRecurringRoles,
    #[error("proof fingerprint does not recur under the frozen gates")]
    RecurrenceGate,
    #[error("latent role proof mismatch: {0}")]
    ProofMismatch(&'static str),
    #[error("role id collision for {0:?}")]
    RoleIdCollision(RoleId),
    #[error("registry conflict for role {0:?}")]
    RegistryConflict(RoleId),
    #[error("role {0:?} is not registered")]
    UnregisteredRole(RoleId),
    #[error("registry proof mismatch for role {0:?}")]
    RegistryProofMismatch(RoleId),
    #[error("foreign discovery scope: expected {expected}, got {actual}")]
    ForeignDiscoveryScope { expected: u64, actual: u64 },
    #[error("role {role_id:?} is not validated; current status is {status:?}")]
    RoleNotValidated {
        role_id: RoleId,
        status: AbstractionStatus,
    },
    #[error("validated role has no member in transfer graph {0}")]
    NoTransferMember(u64),
    #[error("role-conditioned evidence projection is empty")]
    EmptyProjection,
}

/// Derive every recurring exact structural equivalence class that satisfies the
/// frozen recurrence gates. The function does not receive a target role or a
/// requested role count.
pub fn induce_latent_roles(
    corpus: &StructuralCorpus,
    config: RoleInductionConfig,
    budget: &mut RoleInductionBudget,
) -> Result<Vec<LatentRoleProof>, LatentRoleError> {
    validate_corpus(corpus)?;
    let scope_digest = corpus_digest(corpus);
    let groups = fingerprint_groups(corpus, budget)?;
    let mut proofs = Vec::new();
    let mut seen_role_ids = BTreeMap::<RoleId, StructuralFingerprint>::new();

    for (fingerprint, mut members) in groups {
        budget.candidate_groups_considered = budget.candidate_groups_considered.saturating_add(1);
        members.sort();
        let supporting_graph_ids = members
            .iter()
            .map(|member| member.graph_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if supporting_graph_ids.len() < config.min_supporting_graphs
            || members.len() < config.min_members
        {
            continue;
        }

        let role_id = role_id_for_fingerprint(fingerprint);
        if let Some(existing) = seen_role_ids.insert(role_id, fingerprint) {
            if existing != fingerprint {
                return Err(LatentRoleError::RoleIdCollision(role_id));
            }
        }
        let proof_id = proof_digest(
            scope_digest,
            role_id,
            fingerprint,
            &members,
            &supporting_graph_ids,
        );
        proofs.push(LatentRoleProof {
            proof_id,
            role_id,
            discovery_scope_digest: scope_digest,
            structural_fingerprint: fingerprint,
            members,
            supporting_graph_ids,
        });
    }

    if proofs.is_empty() {
        return Err(LatentRoleError::NoRecurringRoles);
    }
    proofs.sort_by_key(|proof| proof.role_id);
    Ok(proofs)
}

/// Independently recompute exact membership and every proof field from the raw
/// corpus before returning an opaque validated certificate.
pub fn validate_latent_role(
    corpus: &StructuralCorpus,
    proof: &LatentRoleProof,
    config: RoleInductionConfig,
    budget: &mut RoleInductionBudget,
) -> Result<ValidatedLatentRoleCertificate, LatentRoleError> {
    validate_corpus(corpus)?;
    let scope_digest = corpus_digest(corpus);
    let groups = fingerprint_groups(corpus, budget)?;
    budget.candidate_groups_considered = groups.len();

    let mut members = groups
        .get(&proof.structural_fingerprint)
        .cloned()
        .ok_or(LatentRoleError::RecurrenceGate)?;
    members.sort();
    let supporting_graph_ids = members
        .iter()
        .map(|member| member.graph_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if supporting_graph_ids.len() < config.min_supporting_graphs
        || members.len() < config.min_members
    {
        return Err(LatentRoleError::RecurrenceGate);
    }

    let role_id = role_id_for_fingerprint(proof.structural_fingerprint);
    let proof_id = proof_digest(
        scope_digest,
        role_id,
        proof.structural_fingerprint,
        &members,
        &supporting_graph_ids,
    );

    if proof.discovery_scope_digest != scope_digest {
        return Err(LatentRoleError::ProofMismatch("discovery_scope_digest"));
    }
    if proof.role_id != role_id {
        return Err(LatentRoleError::ProofMismatch("role_id"));
    }
    if proof.members != members {
        return Err(LatentRoleError::ProofMismatch("members"));
    }
    if proof.supporting_graph_ids != supporting_graph_ids {
        return Err(LatentRoleError::ProofMismatch("supporting_graph_ids"));
    }
    if proof.proof_id != proof_id {
        return Err(LatentRoleError::ProofMismatch("proof_id"));
    }

    Ok(ValidatedLatentRoleCertificate {
        proof_id,
        role_id,
        discovery_scope_digest: scope_digest,
        structural_fingerprint: proof.structural_fingerprint,
        member_count: members.len(),
        supporting_graph_count: supporting_graph_ids.len(),
    })
}

pub fn discovery_scope_digest(corpus: &StructuralCorpus) -> Result<u64, LatentRoleError> {
    validate_corpus(corpus)?;
    Ok(corpus_digest(corpus))
}

/// Recognize exact structural instances of a validated shadow role in a graph
/// that was not part of the discovery corpus.
pub fn recognize_role_members(
    graph: &StructuralGraph,
    certificate: &ValidatedLatentRoleCertificate,
    registry: &ShadowAbstractionRegistry,
    budget: &mut TransferRecognitionBudget,
) -> Result<Vec<Atom>, LatentRoleError> {
    registry.require_usable(certificate)?;
    validate_graph(graph)?;
    let index = GraphIndex::new(graph);
    let mut members = Vec::new();
    let mut nodes = graph.nodes.clone();
    nodes.sort();
    for node in nodes {
        budget.node_fingerprint_evaluations =
            budget.node_fingerprint_evaluations.saturating_add(1);
        let fingerprint = compute_fingerprint(&index, &node, &mut budget.edge_traversals);
        if fingerprint == certificate.structural_fingerprint {
            members.push(node);
        }
    }
    if members.is_empty() {
        return Err(LatentRoleError::NoTransferMember(graph.graph_id));
    }
    Ok(members)
}

/// Project a mixed H11 evidence graph through a validated structural role.
///
/// The result remains inert raw evidence. H12 cannot admit it to PECS. Callers
/// must still use H11 `infer_graph_rule`, `validate_graph_rule`, and
/// `admit_graph_certificate` for executable effects.
pub fn project_evidence_for_role(
    graph: &StructuralGraph,
    evidence: &MixedEvidenceGraph,
    certificate: &ValidatedLatentRoleCertificate,
    registry: &ShadowAbstractionRegistry,
    budget: &mut TransferRecognitionBudget,
) -> Result<MixedEvidenceGraph, LatentRoleError> {
    let members = recognize_role_members(graph, certificate, registry, budget)?;
    let member_set = members.into_iter().collect::<BTreeSet<_>>();
    project_evidence_for_member_set(evidence, &member_set, budget)
}

/// Explicit control-only projection helper. It performs the same evidence scan
/// as role projection but carries no structural certificate and cannot change a
/// registry state. The H12 experiment uses it for random and size-matched
/// grouping controls before sending the resulting inert evidence through H11.
pub fn project_evidence_for_control_group(
    evidence: &MixedEvidenceGraph,
    members: &BTreeSet<Atom>,
    budget: &mut TransferRecognitionBudget,
) -> Result<MixedEvidenceGraph, LatentRoleError> {
    project_evidence_for_member_set(evidence, members, budget)
}

pub fn structural_fingerprint(
    graph: &StructuralGraph,
    node: &Atom,
    budget: &mut RoleInductionBudget,
) -> Result<StructuralFingerprint, LatentRoleError> {
    validate_graph(graph)?;
    if !graph.nodes.contains(node) {
        return Err(LatentRoleError::UnknownEndpoint {
            graph_id: graph.graph_id,
            node: node.as_str().to_string(),
        });
    }
    let index = GraphIndex::new(graph);
    budget.node_fingerprint_evaluations = budget.node_fingerprint_evaluations.saturating_add(1);
    Ok(compute_fingerprint(
        &index,
        node,
        &mut budget.edge_traversals,
    ))
}

fn project_evidence_for_member_set(
    evidence: &MixedEvidenceGraph,
    members: &BTreeSet<Atom>,
    budget: &mut TransferRecognitionBudget,
) -> Result<MixedEvidenceGraph, LatentRoleError> {
    let mut projected = Vec::new();
    for episode in &evidence.evidence {
        budget.evidence_episode_scans = budget.evidence_episode_scans.saturating_add(1);
        if members.contains(&episode.intervention) {
            projected.push(episode.clone());
        }
    }
    if projected.is_empty() {
        return Err(LatentRoleError::EmptyProjection);
    }
    Ok(MixedEvidenceGraph { evidence: projected })
}

fn fingerprint_groups(
    corpus: &StructuralCorpus,
    budget: &mut RoleInductionBudget,
) -> Result<BTreeMap<StructuralFingerprint, Vec<RoleMember>>, LatentRoleError> {
    let mut groups = BTreeMap::<StructuralFingerprint, Vec<RoleMember>>::new();
    let mut graphs = corpus.graphs.iter().collect::<Vec<_>>();
    graphs.sort_by_key(|graph| graph.graph_id);
    for graph in graphs {
        let index = GraphIndex::new(graph);
        let mut nodes = graph.nodes.clone();
        nodes.sort();
        for node in nodes {
            budget.node_fingerprint_evaluations =
                budget.node_fingerprint_evaluations.saturating_add(1);
            let fingerprint = compute_fingerprint(&index, &node, &mut budget.edge_traversals);
            groups.entry(fingerprint).or_default().push(RoleMember {
                graph_id: graph.graph_id,
                node,
            });
        }
    }
    Ok(groups)
}

fn validate_corpus(corpus: &StructuralCorpus) -> Result<(), LatentRoleError> {
    if corpus.graphs.is_empty() {
        return Err(LatentRoleError::EmptyCorpus);
    }
    let mut ids = BTreeSet::new();
    for graph in &corpus.graphs {
        if !ids.insert(graph.graph_id) {
            return Err(LatentRoleError::DuplicateGraphId(graph.graph_id));
        }
        validate_graph(graph)?;
    }
    Ok(())
}

fn validate_graph(graph: &StructuralGraph) -> Result<(), LatentRoleError> {
    if graph.nodes.is_empty() {
        return Err(LatentRoleError::EmptyGraph(graph.graph_id));
    }
    let mut nodes = BTreeSet::new();
    for node in &graph.nodes {
        if !nodes.insert(node.clone()) {
            return Err(LatentRoleError::DuplicateNode {
                graph_id: graph.graph_id,
                node: node.as_str().to_string(),
            });
        }
    }
    let mut edges = BTreeSet::new();
    for edge in &graph.edges {
        if edge.from == edge.to {
            return Err(LatentRoleError::SelfEdge {
                graph_id: graph.graph_id,
                node: edge.from.as_str().to_string(),
            });
        }
        if !nodes.contains(&edge.from) {
            return Err(LatentRoleError::UnknownEndpoint {
                graph_id: graph.graph_id,
                node: edge.from.as_str().to_string(),
            });
        }
        if !nodes.contains(&edge.to) {
            return Err(LatentRoleError::UnknownEndpoint {
                graph_id: graph.graph_id,
                node: edge.to.as_str().to_string(),
            });
        }
        if !edges.insert(edge.clone()) {
            return Err(LatentRoleError::DuplicateEdge {
                graph_id: graph.graph_id,
                from: edge.from.as_str().to_string(),
                to: edge.to.as_str().to_string(),
            });
        }
    }
    Ok(())
}

struct GraphIndex {
    nodes: Vec<Atom>,
    outgoing: BTreeMap<Atom, Vec<Atom>>,
    incoming: BTreeMap<Atom, Vec<Atom>>,
}

impl GraphIndex {
    fn new(graph: &StructuralGraph) -> Self {
        let mut nodes = graph.nodes.clone();
        nodes.sort();
        let mut outgoing = nodes
            .iter()
            .cloned()
            .map(|node| (node, Vec::new()))
            .collect::<BTreeMap<_, _>>();
        let mut incoming = outgoing.clone();
        for edge in &graph.edges {
            outgoing.get_mut(&edge.from).unwrap().push(edge.to.clone());
            incoming.get_mut(&edge.to).unwrap().push(edge.from.clone());
        }
        for neighbors in outgoing.values_mut() {
            neighbors.sort();
        }
        for neighbors in incoming.values_mut() {
            neighbors.sort();
        }
        Self {
            nodes,
            outgoing,
            incoming,
        }
    }

    fn in_degree(&self, node: &Atom) -> usize {
        self.incoming.get(node).map_or(0, Vec::len)
    }

    fn out_degree(&self, node: &Atom) -> usize {
        self.outgoing.get(node).map_or(0, Vec::len)
    }
}

fn compute_fingerprint(
    index: &GraphIndex,
    node: &Atom,
    edge_traversals: &mut usize,
) -> StructuralFingerprint {
    let predecessors = index.incoming.get(node).unwrap();
    let successors = index.outgoing.get(node).unwrap();

    *edge_traversals = edge_traversals
        .saturating_add(predecessors.len())
        .saturating_add(successors.len());

    let predecessor_convergence_count = predecessors
        .iter()
        .filter(|candidate| index.in_degree(candidate) >= 2)
        .count();
    let predecessor_divergence_count = predecessors
        .iter()
        .filter(|candidate| index.out_degree(candidate) >= 2)
        .count();
    let successor_convergence_count = successors
        .iter()
        .filter(|candidate| index.in_degree(candidate) >= 2)
        .count();
    let successor_divergence_count = successors
        .iter()
        .filter(|candidate| index.out_degree(candidate) >= 2)
        .count();

    let ancestors = reachable(index, node, true, edge_traversals);
    let descendants = reachable(index, node, false, edge_traversals);

    let has_convergent_ancestor = ancestors.iter().any(|atom| index.in_degree(atom) >= 2);
    let has_divergent_ancestor = ancestors.iter().any(|atom| index.out_degree(atom) >= 2);
    let has_convergent_descendant = descendants.iter().any(|atom| index.in_degree(atom) >= 2);
    let has_divergent_descendant = descendants.iter().any(|atom| index.out_degree(atom) >= 2);
    let source_reachable = index.in_degree(node) == 0
        || ancestors.iter().any(|atom| index.in_degree(atom) == 0);
    let sink_reachable = index.out_degree(node) == 0
        || descendants.iter().any(|atom| index.out_degree(atom) == 0);

    StructuralFingerprint {
        in_degree: saturating_u16(index.in_degree(node)),
        out_degree: saturating_u16(index.out_degree(node)),
        predecessor_convergence_count: saturating_u16(predecessor_convergence_count),
        predecessor_divergence_count: saturating_u16(predecessor_divergence_count),
        successor_convergence_count: saturating_u16(successor_convergence_count),
        successor_divergence_count: saturating_u16(successor_divergence_count),
        has_convergent_ancestor,
        has_divergent_ancestor,
        has_convergent_descendant,
        has_divergent_descendant,
        source_reachable,
        sink_reachable,
    }
}

fn reachable(
    index: &GraphIndex,
    start: &Atom,
    reverse: bool,
    edge_traversals: &mut usize,
) -> BTreeSet<Atom> {
    let adjacency = if reverse {
        &index.incoming
    } else {
        &index.outgoing
    };
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    for neighbor in adjacency.get(start).unwrap() {
        *edge_traversals = edge_traversals.saturating_add(1);
        if visited.insert(neighbor.clone()) {
            queue.push_back(neighbor.clone());
        }
    }
    while let Some(node) = queue.pop_front() {
        for neighbor in adjacency.get(&node).unwrap() {
            *edge_traversals = edge_traversals.saturating_add(1);
            if visited.insert(neighbor.clone()) {
                queue.push_back(neighbor.clone());
            }
        }
    }
    visited.remove(start);
    visited
}

fn saturating_u16(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn role_id_for_fingerprint(fingerprint: StructuralFingerprint) -> RoleId {
    let mut bytes = Vec::new();
    append_fingerprint(&mut bytes, fingerprint);
    RoleId(fnv1a64(&bytes))
}

fn corpus_digest(corpus: &StructuralCorpus) -> u64 {
    let mut graphs = corpus.graphs.clone();
    graphs.sort_by_key(|graph| graph.graph_id);
    let mut bytes = Vec::new();
    for graph in graphs {
        bytes.extend_from_slice(&graph.graph_id.to_le_bytes());
        let mut nodes = graph.nodes;
        nodes.sort();
        for node in nodes {
            bytes.extend_from_slice(node.as_str().as_bytes());
            bytes.push(0xa1);
        }
        let mut edges = graph.edges;
        edges.sort();
        for edge in edges {
            bytes.extend_from_slice(edge.from.as_str().as_bytes());
            bytes.push(0xb2);
            bytes.extend_from_slice(edge.to.as_str().as_bytes());
            bytes.push(0xc3);
        }
        bytes.push(0xff);
    }
    fnv1a64(&bytes)
}

fn proof_digest(
    scope_digest: u64,
    role_id: RoleId,
    fingerprint: StructuralFingerprint,
    members: &[RoleMember],
    supporting_graph_ids: &[u64],
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&scope_digest.to_le_bytes());
    bytes.extend_from_slice(&role_id.0.to_le_bytes());
    append_fingerprint(&mut bytes, fingerprint);
    for member in members {
        bytes.extend_from_slice(&member.graph_id.to_le_bytes());
        bytes.extend_from_slice(member.node.as_str().as_bytes());
        bytes.push(0xd4);
    }
    for graph_id in supporting_graph_ids {
        bytes.extend_from_slice(&graph_id.to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn append_fingerprint(bytes: &mut Vec<u8>, fingerprint: StructuralFingerprint) {
    bytes.extend_from_slice(&fingerprint.in_degree.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.out_degree.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.predecessor_convergence_count.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.predecessor_divergence_count.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.successor_convergence_count.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.successor_divergence_count.to_le_bytes());
    bytes.push(fingerprint.has_convergent_ancestor as u8);
    bytes.push(fingerprint.has_divergent_ancestor as u8);
    bytes.push(fingerprint.has_convergent_descendant as u8);
    bytes.push(fingerprint.has_divergent_descendant as u8);
    bytes.push(fingerprint.source_reachable as u8);
    bytes.push(fingerprint.sink_reachable as u8);
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule_induction::EvidenceEpisode;

    fn atom(value: &str) -> Atom {
        Atom::new(value).unwrap()
    }

    fn edge(from: &str, to: &str) -> DirectedEdge {
        DirectedEdge {
            from: atom(from),
            to: atom(to),
        }
    }

    fn motif(graph_id: u64, prefix: &str, extra_decoy: bool) -> StructuralGraph {
        let names = [
            "s0", "s1", "join", "latent", "split", "t0", "t1", "r0", "r1", "r2",
        ]
        .into_iter()
        .map(|name| atom(&format!("{prefix}_{name}")))
        .collect::<Vec<_>>();
        let mut edges = vec![
            edge(&format!("{prefix}_s0"), &format!("{prefix}_join")),
            edge(&format!("{prefix}_s1"), &format!("{prefix}_join")),
            edge(&format!("{prefix}_join"), &format!("{prefix}_latent")),
            edge(&format!("{prefix}_latent"), &format!("{prefix}_split")),
            edge(&format!("{prefix}_split"), &format!("{prefix}_t0")),
            edge(&format!("{prefix}_split"), &format!("{prefix}_t1")),
            edge(&format!("{prefix}_r0"), &format!("{prefix}_r1")),
            edge(&format!("{prefix}_r1"), &format!("{prefix}_r2")),
        ];
        let mut nodes = names;
        if extra_decoy {
            nodes.push(atom(&format!("{prefix}_x0")));
            nodes.push(atom(&format!("{prefix}_x1")));
            edges.push(edge(&format!("{prefix}_x0"), &format!("{prefix}_x1")));
        }
        StructuralGraph {
            graph_id,
            nodes,
            edges,
        }
    }

    fn corpus(prefix: &str, offset: u64) -> StructuralCorpus {
        StructuralCorpus {
            graphs: (0..4)
                .map(|index| {
                    motif(
                        offset + index,
                        &format!("{prefix}_{index}"),
                        index % 2 == 0,
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn induces_and_independently_validates_opaque_recurring_roles() {
        let corpus = corpus("alpha", 10);
        let config = RoleInductionConfig::default();
        let mut discovery_budget = RoleInductionBudget::default();
        let proofs = induce_latent_roles(&corpus, config, &mut discovery_budget).unwrap();
        assert!(!proofs.is_empty());
        assert!(discovery_budget.node_fingerprint_evaluations > 0);

        let scope = discovery_scope_digest(&corpus).unwrap();
        let mut registry = ShadowAbstractionRegistry::new(scope);
        for proof in &proofs {
            registry.register_candidate(proof).unwrap();
            let mut validation_budget = RoleInductionBudget::default();
            let certificate = validate_latent_role(&corpus, proof, config, &mut validation_budget)
                .unwrap();
            assert_eq!(certificate.role_id(), proof.role_id);
            assert_eq!(certificate.member_count(), proof.members.len());
            assert!(validation_budget.node_fingerprint_evaluations > 0);
            registry.admit_validated(&certificate).unwrap();
            assert_eq!(
                registry.status(certificate.role_id()),
                Some(AbstractionStatus::ValidatedShadow)
            );
        }
    }

    #[test]
    fn one_member_tamper_is_rejected_after_independent_recomputation() {
        let corpus = corpus("beta", 100);
        let config = RoleInductionConfig::default();
        let mut discovery_budget = RoleInductionBudget::default();
        let mut proof = induce_latent_roles(&corpus, config, &mut discovery_budget)
            .unwrap()
            .remove(0);
        proof.members[0].node = atom("forged_member");
        let mut validation_budget = RoleInductionBudget::default();
        let error = validate_latent_role(&corpus, &proof, config, &mut validation_budget)
            .unwrap_err();
        assert_eq!(error, LatentRoleError::ProofMismatch("members"));
        assert!(validation_budget.node_fingerprint_evaluations > 0);
    }

    #[test]
    fn foreign_scope_certificate_cannot_project_evidence() {
        let corpus_a = corpus("gamma", 200);
        let corpus_b = corpus("delta", 300);
        let config = RoleInductionConfig::default();

        let mut budget_a = RoleInductionBudget::default();
        let proof_a = induce_latent_roles(&corpus_a, config, &mut budget_a)
            .unwrap()
            .remove(0);
        let mut validate_a = RoleInductionBudget::default();
        let certificate_a = validate_latent_role(&corpus_a, &proof_a, config, &mut validate_a)
            .unwrap();

        let scope_b = discovery_scope_digest(&corpus_b).unwrap();
        let registry_b = ShadowAbstractionRegistry::new(scope_b);
        let transfer = motif(999, "transfer", false);
        let evidence = MixedEvidenceGraph {
            evidence: vec![EvidenceEpisode {
                evidence_id: 1,
                intervention: atom("transfer_latent"),
                outcomes: BTreeSet::from([atom("outcome")]),
            }],
        };
        let mut transfer_budget = TransferRecognitionBudget::default();
        let error = project_evidence_for_role(
            &transfer,
            &evidence,
            &certificate_a,
            &registry_b,
            &mut transfer_budget,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            LatentRoleError::ForeignDiscoveryScope { .. }
        ));
    }

    #[test]
    fn validated_role_projects_only_matching_interventions() {
        let corpus = corpus("epsilon", 400);
        let config = RoleInductionConfig::default();
        let mut discovery_budget = RoleInductionBudget::default();
        let proofs = induce_latent_roles(&corpus, config, &mut discovery_budget).unwrap();
        let scope = discovery_scope_digest(&corpus).unwrap();
        let mut registry = ShadowAbstractionRegistry::new(scope);
        let transfer = motif(1000, "future", true);

        let mut chosen = None;
        for proof in proofs {
            registry.register_candidate(&proof).unwrap();
            let mut validation_budget = RoleInductionBudget::default();
            let certificate =
                validate_latent_role(&corpus, &proof, config, &mut validation_budget).unwrap();
            registry.admit_validated(&certificate).unwrap();
            let mut recognition_budget = TransferRecognitionBudget::default();
            let members = recognize_role_members(
                &transfer,
                &certificate,
                &registry,
                &mut recognition_budget,
            );
            if let Ok(members) = members {
                if members == vec![atom("future_latent")] {
                    chosen = Some(certificate);
                    break;
                }
            }
        }
        let certificate = chosen.expect("recurring unnamed latent motif role");
        let evidence = MixedEvidenceGraph {
            evidence: vec![
                EvidenceEpisode {
                    evidence_id: 1,
                    intervention: atom("future_latent"),
                    outcomes: BTreeSet::from([atom("goal")]),
                },
                EvidenceEpisode {
                    evidence_id: 2,
                    intervention: atom("future_r1"),
                    outcomes: BTreeSet::from([atom("decoy")]),
                },
            ],
        };
        let mut transfer_budget = TransferRecognitionBudget::default();
        let projected = project_evidence_for_role(
            &transfer,
            &evidence,
            &certificate,
            &registry,
            &mut transfer_budget,
        )
        .unwrap();
        assert_eq!(projected.evidence.len(), 1);
        assert_eq!(projected.evidence[0].intervention, atom("future_latent"));
        assert_eq!(transfer_budget.evidence_episode_scans, 2);
    }
}
