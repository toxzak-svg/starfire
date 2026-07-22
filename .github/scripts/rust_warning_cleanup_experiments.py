from pathlib import Path


def replace(path: str, old: str, new: str, *, all_matches: bool = False) -> None:
    file = Path(path)
    text = file.read_text()
    if new and new in text:
        return
    if old not in text:
        return
    file.write_text(text.replace(old, new) if all_matches else text.replace(old, new, 1))


# These fields are retained as frozen fixture provenance even though the current
# executors do not read them. Keep the allowances local to the exact metadata.
replace(
    "lib/examples/h13c_structural_role_transfer_stress.rs",
    "    proposal_budget: TransportBudget,\n",
    "    #[allow(dead_code)] // Frozen proposal-budget provenance.\n    proposal_budget: TransportBudget,\n",
)
replace(
    "lib/examples/h10_evidence_bound_rule_induction.rs",
    "    root_id: u64,\n",
    "    #[allow(dead_code)] // Frozen root provenance.\n    root_id: u64,\n",
)
replace(
    "lib/examples/h10_evidence_bound_rule_induction.rs",
    "    family: &'static str,\n",
    "    #[allow(dead_code)] // Frozen split-family provenance.\n    family: &'static str,\n",
)
replace(
    "lib/examples/h9_executable_commitment_state.rs",
    "    id: u64,\n",
    "    #[allow(dead_code)] // Frozen task provenance.\n    id: u64,\n",
)
replace(
    "lib/examples/h9_executable_commitment_state.rs",
    "    family_name: &'static str,\n",
    "    #[allow(dead_code)] // Frozen family label.\n    family_name: &'static str,\n",
)
replace(
    "lib/examples/h9_executable_commitment_state.rs",
    "    decoy_goal: Atom,\n",
    "    #[allow(dead_code)] // Retained control-fixture endpoint.\n    decoy_goal: Atom,\n",
)
replace(
    "lib/examples/h11_graph_discovered_relation_induction.rs",
    "    family: &'static str,\n",
    "    #[allow(dead_code)] // Frozen split-family provenance.\n    family: &'static str,\n",
)
replace(
    "lib/examples/h11_graph_discovered_relation_induction.rs",
    "    irrelevant_goal: Atom,\n",
    "    #[allow(dead_code)] // Retained negative-control endpoint.\n    irrelevant_goal: Atom,\n",
)

# Resolver labels remain part of the frozen H4 experiment vocabulary even when
# the current executable derives its active resolver set elsewhere.
replace(
    "lib/examples/h4_real_cycle_shadow_probe.rs",
    "const RESOLVERS: [&str; 5] = [\n",
    "#[allow(dead_code)] // Frozen resolver vocabulary.\nconst RESOLVERS: [&str; 5] = [\n",
)
replace(
    "lib/examples/h4_shadow_promotion_probe.rs",
    'const RESOLVERS: [&str; 3] = ["memory", "reasoning", "causal"];\n',
    '#[allow(dead_code)] // Frozen resolver vocabulary.\nconst RESOLVERS: [&str; 3] = ["memory", "reasoning", "causal"];\n',
)

# Every successful R1 execution path necessarily completed a proposal before it
# can reach the returned Execution record. Represent that invariant directly.
replace(
    "lib/examples/omega_r1_representative_transportability/part2.rs",
    "    let mut proposal_succeeded = false;\n",
    "    let proposal_succeeded = true;\n",
)
replace(
    "lib/examples/omega_r1_representative_transportability/part2.rs",
    "            proposal_succeeded = true;\n",
    "",
    all_matches=True,
)

# The descendant language is the source of truth after the path window. Remove
# transient writes that were always overwritten by this authoritative check.
replace(
    "lib/examples/omega_d1_transport_certified_descendant_necessity/part3.rs",
    "    let mut descendant_admitted_during_window = false;\n",
    "",
)
replace(
    "lib/examples/omega_d1_transport_certified_descendant_necessity/part3.rs",
    "            descendant_admitted_during_window = true;\n",
    "",
    all_matches=True,
)
replace(
    "lib/examples/omega_d1_transport_certified_descendant_necessity/part3.rs",
    "    descendant_admitted_during_window = layered.descendant_count() > 0;\n",
    "    let descendant_admitted_during_window = layered.descendant_count() > 0;\n",
)
