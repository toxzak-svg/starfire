//! Proof-carrying structural-role transport for H13-C.
//!
//! This module derives a bounded global gateway signature from an independently
//! validated H12 latent role and recognizes that signature in unseen graphs. It
//! remains shadow-only: there is no Live registry state, PECS mutation, runtime
//! routing, persistence authority, or autonomous action authority.

use crate::commitment_state::Atom;
use crate::latent_roles::{
    validate_latent_role, LatentRoleError, LatentRoleProof, RoleId, RoleInductionBudget,
    RoleInductionConfig, RoleMember, StructuralCorpus, StructuralGraph,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransportSignature {
    pub source_ancestor_bucket: u8,
    pub sink_descendant_bucket: u8,
    pub reachable_source_sink_pairs: u8,
    pub mandatory_source_sink_pairs: u8,
    pub all_reachable_pairs_mandatory: bool,
}

impl TransportSignature {
    pub fn is_gateway_role(self) -> bool {
        self.source_ancestor_bucket == 2
            && self.sink_descendant_bucket == 2
            && self.reachable_source_sink_pairs == 4
            && self.mandatory_source_sink_pairs == 4
            && self.all_reachable_pairs_mandatory
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportBudget {
    pub h12_node_fingerprint_evaluations: usize,
    pub h12_edge_traversals: usize,
    pub h12_candidate_groups_considered: usize,
    pub proof_members_recomputed: usize,
    pub node_signature_evaluations: usize,
    pub reachability_edge_traversals: usize,
    pub candidate_matches: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportRoleProof {
    pub proof_id: u64,
    pub transport_role_id: u64,
    pub source_role_id: RoleId,
    pub development_scope_digest: u64,
    pub signature: TransportSignature,
    pub source_members: Vec<RoleMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedTransportRoleCertificate {
    proof_id: u64,
    transport_role_id: u64,
    source_role_id: RoleId,
    development_scope_digest: u64,
    signature: TransportSignature,
    supporting_member_count: usize,
}

impl ValidatedTransportRoleCertificate {
    pub fn proof_id(&self) -> u64 {
        self.proof_id
    }

    pub fn transport_role_id(&self) -> u64 {
        self.transport_role_id
    }

    pub fn source_role_id(&self) -> RoleId {
        self.source_role_id
    }

    pub fn development_scope_digest(&self) -> u64 {
        self.development_scope_digest
    }

    pub fn signature(&self) -> TransportSignature {
        self.signature
    }

    pub fn supporting_member_count(&self) -> usize {
        self.supporting_member_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportStatus {
    Candidate,
    ValidatedShadow,
    TransferValidated,
    Rejected,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportRegistryEntry {
    pub transport_role_id: u64,
    pub proof_id: u64,
    pub development_scope_digest: u64,
    pub status: TransportStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowTransportRegistry {
    development_scope_digest: u64,
    entries: BTreeMap<u64, TransportRegistryEntry>,
}

impl ShadowTransportRegistry {
    pub fn new(development_scope_digest: u64) -> Self {
        Self {
            development_scope_digest,
            entries: BTreeMap::new(),
        }
    }

    pub fn development_scope_digest(&self) -> u64 {
        self.development_scope_digest
    }

    pub fn register_candidate(&mut self, proof: &TransportRoleProof) -> Result<(), TransportError> {
        self.require_scope(proof.development_scope_digest)?;
        match self.entries.get(&proof.transport_role_id) {
            Some(existing) if existing.proof_id != proof.proof_id => {
                return Err(TransportError::RegistryConflict(proof.transport_role_id));
            }
            Some(_) => return Ok(()),
            None => {}
        }
        self.entries.insert(
            proof.transport_role_id,
            TransportRegistryEntry {
                transport_role_id: proof.transport_role_id,
                proof_id: proof.proof_id,
                development_scope_digest: proof.development_scope_digest,
                status: TransportStatus::Candidate,
            },
        );
        Ok(())
    }

    pub fn admit_validated(
        &mut self,
        certificate: &ValidatedTransportRoleCertificate,
    ) -> Result<(), TransportError> {
        self.require_scope(certificate.development_scope_digest)?;
        let entry = self
            .entries
            .get_mut(&certificate.transport_role_id)
            .ok_or(TransportError::UnregisteredRole(
                certificate.transport_role_id,
            ))?;
        if entry.proof_id != certificate.proof_id {
            return Err(TransportError::RegistryProofMismatch(
                certificate.transport_role_id,
            ));
        }
        entry.status = TransportStatus::ValidatedShadow;
        Ok(())
    }

    pub fn mark_transfer_validated(
        &mut self,
        certificate: &ValidatedTransportRoleCertificate,
    ) -> Result<(), TransportError> {
        self.require_usable(certificate)?;
        self.entries
            .get_mut(&certificate.transport_role_id)
            .expect("usable certificate must have an entry")
            .status = TransportStatus::TransferValidated;
        Ok(())
    }

    pub fn status(&self, transport_role_id: u64) -> Option<TransportStatus> {
        self.entries.get(&transport_role_id).map(|entry| entry.status)
    }

    pub fn entries(&self) -> impl Iterator<Item = &TransportRegistryEntry> {
        self.entries.values()
    }

    fn require_scope(&self, actual: u64) -> Result<(), TransportError> {
        if actual != self.development_scope_digest {
            return Err(TransportError::ForeignDevelopmentScope {
                expected: self.development_scope_digest,
                actual,
            });
        }
        Ok(())
    }

    fn require_usable(
        &self,
        certificate: &ValidatedTransportRoleCertificate,
    ) -> Result<(), TransportError> {
        self.require_scope(certificate.development_scope_digest)?;
        let entry = self
            .entries
            .get(&certificate.transport_role_id)
            .ok_or(TransportError::UnregisteredRole(
                certificate.transport_role_id,
            ))?;
        if entry.proof_id != certificate.proof_id {
            return Err(TransportError::RegistryProofMismatch(
                certificate.transport_role_id,
            ));
        }
        match entry.status {
            TransportStatus::ValidatedShadow | TransportStatus::TransferValidated => Ok(()),
            status => Err(TransportError::RoleNotValidated {
                transport_role_id: certificate.transport_role_id,
                status,
            }),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransportError {
    #[error(transparent)]
    H12(#[from] LatentRoleError),
    #[error("development role has no members")]
    EmptySourceRole,
    #[error("development member references unknown graph {0}")]
    UnknownDevelopmentGraph(u64),
    #[error("development member references absent node {node} in graph {graph_id}")]
    UnknownDevelopmentNode { graph_id: u64, node: String },
    #[error("development role does not have one invariant transport signature")]
    InconsistentDevelopmentSignature,
    #[error("development role does not satisfy the frozen global gateway predicate")]
    NotGatewayRole,
    #[error("transport proof mismatch: {0}")]
    ProofMismatch(&'static str),
    #[error("graph {0} has no nodes")]
    EmptyGraph(u64),
    #[error("graph {graph_id} contains duplicate node {node}")]
    DuplicateNode { graph_id: u64, node: String },
    #[error("graph {graph_id} edge references absent endpoint {node}")]
    UnknownEndpoint { graph_id: u64, node: String },
    #[error("graph {graph_id} contains a self edge at {node}")]
    SelfEdge { graph_id: u64, node: String },
    #[error("graph {graph_id} contains duplicate edge {from} -> {to}")]
    DuplicateEdge {
        graph_id: u64,
        from: String,
        to: String,
    },
    #[error("registry conflict for transport role {0}")]
    RegistryConflict(u64),
    #[error("transport role {0} is not registered")]
    UnregisteredRole(u64),
    #[error("registry proof mismatch for transport role {0}")]
    RegistryProofMismatch(u64),
    #[error("foreign development scope: expected {expected}, got {actual}")]
    ForeignDevelopmentScope { expected: u64, actual: u64 },
    #[error("transport role {transport_role_id} is not validated; current status is {status:?}")]
    RoleNotValidated {
        transport_role_id: u64,
        status: TransportStatus,
    },
}

pub fn propose_transport_role(
    development: &StructuralCorpus,
    source_proof: &LatentRoleProof,
    config: RoleInductionConfig,
    budget: &mut TransportBudget,
) -> Result<TransportRoleProof, TransportError> {
    derive_transport_proof(development, source_proof, config, budget)
}

pub fn validate_transport_role(
    development: &StructuralCorpus,
    source_proof: &LatentRoleProof,
    proposed: &TransportRoleProof,
    config: RoleInductionConfig,
    budget: &mut TransportBudget,
) -> Result<ValidatedTransportRoleCertificate, TransportError> {
    let recomputed = derive_transport_proof(development, source_proof, config, budget)?;
    if proposed.development_scope_digest != recomputed.development_scope_digest {
        return Err(TransportError::ProofMismatch("development_scope_digest"));
    }
    if proposed.source_role_id != recomputed.source_role_id {
        return Err(TransportError::ProofMismatch("source_role_id"));
    }
    if proposed.signature != recomputed.signature {
        return Err(TransportError::ProofMismatch("signature"));
    }
    if proposed.source_members != recomputed.source_members {
        return Err(TransportError::ProofMismatch("source_members"));
    }
    if proposed.transport_role_id != recomputed.transport_role_id {
        return Err(TransportError::ProofMismatch("transport_role_id"));
    }
    if proposed.proof_id != recomputed.proof_id {
        return Err(TransportError::ProofMismatch("proof_id"));
    }
    Ok(ValidatedTransportRoleCertificate {
        proof_id: recomputed.proof_id,
        transport_role_id: recomputed.transport_role_id,
        source_role_id: recomputed.source_role_id,
        development_scope_digest: recomputed.development_scope_digest,
        signature: recomputed.signature,
        supporting_member_count: recomputed.source_members.len(),
    })
}

pub fn recognize_transport_members(
    graph: &StructuralGraph,
    certificate: &ValidatedTransportRoleCertificate,
    registry: &ShadowTransportRegistry,
    budget: &mut TransportBudget,
) -> Result<Vec<Atom>, TransportError> {
    registry.require_usable(certificate)?;
    validate_graph(graph)?;
    let index = GraphIndex::new(graph);
    let mut nodes = graph.nodes.clone();
    nodes.sort();
    let mut matches = Vec::new();
    for node in nodes {
        let signature = compute_transport_signature(&index, &node, budget);
        if signature == certificate.signature {
            budget.candidate_matches = budget.candidate_matches.saturating_add(1);
            matches.push(node);
        }
    }
    Ok(matches)
}

pub fn transport_signature(
    graph: &StructuralGraph,
    node: &Atom,
    budget: &mut TransportBudget,
) -> Result<TransportSignature, TransportError> {
    validate_graph(graph)?;
    if !graph.nodes.contains(node) {
        return Err(TransportError::UnknownDevelopmentNode {
            graph_id: graph.graph_id,
            node: node.as_str().to_string(),
        });
    }
    Ok(compute_transport_signature(
        &GraphIndex::new(graph),
        node,
        budget,
    ))
}

fn derive_transport_proof(
    development: &StructuralCorpus,
    source_proof: &LatentRoleProof,
    config: RoleInductionConfig,
    budget: &mut TransportBudget,
) -> Result<TransportRoleProof, TransportError> {
    let mut h12_budget = RoleInductionBudget::default();
    let source_certificate =
        validate_latent_role(development, source_proof, config, &mut h12_budget)?;
    budget.h12_node_fingerprint_evaluations = budget
        .h12_node_fingerprint_evaluations
        .saturating_add(h12_budget.node_fingerprint_evaluations);
    budget.h12_edge_traversals = budget
        .h12_edge_traversals
        .saturating_add(h12_budget.edge_traversals);
    budget.h12_candidate_groups_considered = budget
        .h12_candidate_groups_considered
        .saturating_add(h12_budget.candidate_groups_considered);

    if source_proof.members.is_empty() {
        return Err(TransportError::EmptySourceRole);
    }

    let mut signatures = BTreeSet::new();
    for member in &source_proof.members {
        let graph = development
            .graphs
            .iter()
            .find(|graph| graph.graph_id == member.graph_id)
            .ok_or(TransportError::UnknownDevelopmentGraph(member.graph_id))?;
        if !graph.nodes.contains(&member.node) {
            return Err(TransportError::UnknownDevelopmentNode {
                graph_id: member.graph_id,
                node: member.node.as_str().to_string(),
            });
        }
        signatures.insert(transport_signature(graph, &member.node, budget)?);
        budget.proof_members_recomputed = budget.proof_members_recomputed.saturating_add(1);
    }
    if signatures.len() != 1 {
        return Err(TransportError::InconsistentDevelopmentSignature);
    }
    let signature = *signatures.iter().next().expect("one signature");
    if !signature.is_gateway_role() {
        return Err(TransportError::NotGatewayRole);
    }

    let development_scope_digest = source_certificate.discovery_scope_digest();
    let source_role_id = source_certificate.role_id();
    let transport_role_id = transport_role_digest(development_scope_digest, source_role_id, signature);
    let proof_id = transport_proof_digest(
        development_scope_digest,
        source_role_id,
        transport_role_id,
        signature,
        &source_proof.members,
    );

    Ok(TransportRoleProof {
        proof_id,
        transport_role_id,
        source_role_id,
        development_scope_digest,
        signature,
        source_members: source_proof.members.clone(),
    })
}

fn validate_graph(graph: &StructuralGraph) -> Result<(), TransportError> {
    if graph.nodes.is_empty() {
        return Err(TransportError::EmptyGraph(graph.graph_id));
    }
    let mut nodes = BTreeSet::new();
    for node in &graph.nodes {
        if !nodes.insert(node.clone()) {
            return Err(TransportError::DuplicateNode {
                graph_id: graph.graph_id,
                node: node.as_str().to_string(),
            });
        }
    }
    let mut edges = BTreeSet::new();
    for edge in &graph.edges {
        if edge.from == edge.to {
            return Err(TransportError::SelfEdge {
                graph_id: graph.graph_id,
                node: edge.from.as_str().to_string(),
            });
        }
        for endpoint in [&edge.from, &edge.to] {
            if !nodes.contains(endpoint) {
                return Err(TransportError::UnknownEndpoint {
                    graph_id: graph.graph_id,
                    node: endpoint.as_str().to_string(),
                });
            }
        }
        if !edges.insert((edge.from.clone(), edge.to.clone())) {
            return Err(TransportError::DuplicateEdge {
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
        let mut outgoing = graph
            .nodes
            .iter()
            .cloned()
            .map(|node| (node, Vec::new()))
            .collect::<BTreeMap<_, _>>();
        let mut incoming = outgoing.clone();
        for edge in &graph.edges {
            outgoing
                .get_mut(&edge.from)
                .expect("validated endpoint")
                .push(edge.to.clone());
            incoming
                .get_mut(&edge.to)
                .expect("validated endpoint")
                .push(edge.from.clone());
        }
        for neighbours in outgoing.values_mut() {
            neighbours.sort();
        }
        for neighbours in incoming.values_mut() {
            neighbours.sort();
        }
        let mut nodes = graph.nodes.clone();
        nodes.sort();
        Self {
            nodes,
            outgoing,
            incoming,
        }
    }

    fn sources(&self) -> impl Iterator<Item = &Atom> {
        self.nodes
            .iter()
            .filter(|node| self.incoming.get(*node).is_some_and(Vec::is_empty))
    }

    fn sinks(&self) -> impl Iterator<Item = &Atom> {
        self.nodes
            .iter()
            .filter(|node| self.outgoing.get(*node).is_some_and(Vec::is_empty))
    }
}

fn compute_transport_signature(
    index: &GraphIndex,
    candidate: &Atom,
    budget: &mut TransportBudget,
) -> TransportSignature {
    budget.node_signature_evaluations = budget.node_signature_evaluations.saturating_add(1);
    let sources = index
        .sources()
        .filter(|source| path_exists(index, source, candidate, None, budget))
        .cloned()
        .collect::<Vec<_>>();
    let sinks = index
        .sinks()
        .filter(|sink| path_exists(index, candidate, sink, None, budget))
        .cloned()
        .collect::<Vec<_>>();

    let reachable_pairs = sources.len().saturating_mul(sinks.len());
    let mut mandatory_pairs = 0usize;
    for source in &sources {
        for sink in &sinks {
            if !path_exists(index, source, sink, Some(candidate), budget) {
                mandatory_pairs = mandatory_pairs.saturating_add(1);
            }
        }
    }

    TransportSignature {
        source_ancestor_bucket: bucket_two(sources.len()),
        sink_descendant_bucket: bucket_two(sinks.len()),
        reachable_source_sink_pairs: bucket_four(reachable_pairs),
        mandatory_source_sink_pairs: bucket_four(mandatory_pairs),
        all_reachable_pairs_mandatory: reachable_pairs > 0 && mandatory_pairs == reachable_pairs,
    }
}

fn path_exists(
    index: &GraphIndex,
    start: &Atom,
    goal: &Atom,
    blocked: Option<&Atom>,
    budget: &mut TransportBudget,
) -> bool {
    if blocked.is_some_and(|blocked| blocked == start || blocked == goal) {
        return false;
    }
    if start == goal {
        return true;
    }
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start.clone());
    queue.push_back(start.clone());
    while let Some(node) = queue.pop_front() {
        if let Some(neighbours) = index.outgoing.get(&node) {
            for next in neighbours {
                budget.reachability_edge_traversals =
                    budget.reachability_edge_traversals.saturating_add(1);
                if blocked.is_some_and(|blocked| blocked == next) {
                    continue;
                }
                if next == goal {
                    return true;
                }
                if visited.insert(next.clone()) {
                    queue.push_back(next.clone());
                }
            }
        }
    }
    false
}

fn bucket_two(value: usize) -> u8 {
    value.min(2) as u8
}

fn bucket_four(value: usize) -> u8 {
    value.min(4) as u8
}

fn transport_role_digest(scope: u64, source_role_id: RoleId, signature: TransportSignature) -> u64 {
    let mut hash = Fnv64::new();
    hash.write_u64(scope);
    hash.write_u64(source_role_id.0);
    write_signature(&mut hash, signature);
    hash.finish()
}

fn transport_proof_digest(
    scope: u64,
    source_role_id: RoleId,
    transport_role_id: u64,
    signature: TransportSignature,
    members: &[RoleMember],
) -> u64 {
    let mut hash = Fnv64::new();
    hash.write_u64(scope);
    hash.write_u64(source_role_id.0);
    hash.write_u64(transport_role_id);
    write_signature(&mut hash, signature);
    hash.write_u64(members.len() as u64);
    for member in members {
        hash.write_u64(member.graph_id);
        hash.write_bytes(member.node.as_str().as_bytes());
    }
    hash.finish()
}

fn write_signature(hash: &mut Fnv64, signature: TransportSignature) {
    hash.write_u8(signature.source_ancestor_bucket);
    hash.write_u8(signature.sink_descendant_bucket);
    hash.write_u8(signature.reachable_source_sink_pairs);
    hash.write_u8(signature.mandatory_source_sink_pairs);
    hash.write_u8(u8::from(signature.all_reachable_pairs_mandatory));
}

struct Fnv64(u64);

impl Fnv64 {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.0 ^= 0xff;
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::latent_roles::{
        induce_latent_roles, DirectedEdge, RoleInductionConfig, StructuralGraph,
    };

    fn atom(value: impl Into<String>) -> Atom {
        Atom::new(value.into()).unwrap()
    }

    fn graph(graph_id: u64, local_degree_change: bool) -> (StructuralGraph, Atom) {
        let names = ["s1", "s2", "a", "b", "target", "c", "d", "k1", "k2"]
            .into_iter()
            .map(|name| atom(format!("g{graph_id}_{name}")))
            .collect::<Vec<_>>();
        let [s1, s2, a, b, target, c, d, k1, k2] = names.as_slice() else {
            unreachable!()
        };
        let mut nodes = names.clone();
        let mut edges = vec![
            (s1, a),
            (s2, b),
            (a, target),
            (b, target),
            (target, c),
            (target, d),
            (c, k1),
            (d, k2),
        ]
        .into_iter()
        .map(|(from, to)| DirectedEdge {
            from: from.clone(),
            to: to.clone(),
        })
        .collect::<Vec<_>>();
        if local_degree_change {
            let s3 = atom(format!("g{graph_id}_s3"));
            let k3 = atom(format!("g{graph_id}_k3"));
            nodes.extend([s3.clone(), k3.clone()]);
            edges.push(DirectedEdge {
                from: s3,
                to: target.clone(),
            });
            edges.push(DirectedEdge {
                from: target.clone(),
                to: k3,
            });
        }
        (
            StructuralGraph {
                graph_id,
                nodes,
                edges,
            },
            target.clone(),
        )
    }

    fn development() -> StructuralCorpus {
        StructuralCorpus {
            graphs: (0..4).map(|id| graph(id, false).0).collect(),
        }
    }

    fn gateway_source_proof(corpus: &StructuralCorpus) -> LatentRoleProof {
        let mut budget = RoleInductionBudget::default();
        induce_latent_roles(corpus, RoleInductionConfig::default(), &mut budget)
            .unwrap()
            .into_iter()
            .find(|proof| {
                let mut transport_budget = TransportBudget::default();
                propose_transport_role(
                    corpus,
                    proof,
                    RoleInductionConfig::default(),
                    &mut transport_budget,
                )
                .is_ok()
            })
            .expect("one gateway role")
    }

    #[test]
    fn transfers_across_local_degree_change() {
        let corpus = development();
        let source = gateway_source_proof(&corpus);
        let mut proposal_budget = TransportBudget::default();
        let proof = propose_transport_role(
            &corpus,
            &source,
            RoleInductionConfig::default(),
            &mut proposal_budget,
        )
        .unwrap();
        let mut validation_budget = TransportBudget::default();
        let certificate = validate_transport_role(
            &corpus,
            &source,
            &proof,
            RoleInductionConfig::default(),
            &mut validation_budget,
        )
        .unwrap();
        let mut registry = ShadowTransportRegistry::new(proof.development_scope_digest);
        registry.register_candidate(&proof).unwrap();
        registry.admit_validated(&certificate).unwrap();

        let (future, target) = graph(100, true);
        let mut recognition_budget = TransportBudget::default();
        let members = recognize_transport_members(
            &future,
            &certificate,
            &registry,
            &mut recognition_budget,
        )
        .unwrap();
        assert_eq!(members, vec![target]);
    }

    #[test]
    fn rejects_tampered_identity_and_delayed_admission() {
        let corpus = development();
        let source = gateway_source_proof(&corpus);
        let mut budget = TransportBudget::default();
        let proof = propose_transport_role(
            &corpus,
            &source,
            RoleInductionConfig::default(),
            &mut budget,
        )
        .unwrap();
        let mut tampered = proof.clone();
        tampered.transport_role_id ^= 1;
        assert_eq!(
            validate_transport_role(
                &corpus,
                &source,
                &tampered,
                RoleInductionConfig::default(),
                &mut TransportBudget::default(),
            ),
            Err(TransportError::ProofMismatch("transport_role_id"))
        );

        let certificate = validate_transport_role(
            &corpus,
            &source,
            &proof,
            RoleInductionConfig::default(),
            &mut TransportBudget::default(),
        )
        .unwrap();
        let mut registry = ShadowTransportRegistry::new(proof.development_scope_digest);
        registry.register_candidate(&proof).unwrap();
        let (future, _) = graph(101, true);
        assert!(matches!(
            recognize_transport_members(
                &future,
                &certificate,
                &registry,
                &mut TransportBudget::default(),
            ),
            Err(TransportError::RoleNotValidated { .. })
        ));
    }
}
