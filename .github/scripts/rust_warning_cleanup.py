from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    old_count = text.count(old)
    new_count = text.count(new)
    if old_count == 1:
        file.write_text(text.replace(old, new))
        return
    if old_count == 0 and new_count == 1:
        return
    raise RuntimeError(
        f"unexpected replacement state in {path}: old={old_count}, new={new_count}"
    )


# H5-B intentionally includes the complete frozen H4 executable as a fixture,
# while exercising only its observation path. Keep the allowance local to the
# fixture rather than suppressing dead-code warnings across the crate.
replace_once(
    "lib/examples/h5_b_identifiability_probe.rs",
    'mod frozen_h4_fixture {\n',
    '#[allow(dead_code)]\nmod frozen_h4_fixture {\n',
)

# Encapsulate basin internals and make the configured capacity operational.
replace_once(
    "lib/prediction/basin.rs",
    '    pub nodes: HashMap<NodeId, BasinNode>,\n',
    '    nodes: HashMap<NodeId, BasinNode>,\n',
)
replace_once(
    "lib/prediction/basin.rs",
    '    pub id: NodeId,\n',
    '',
)
replace_once(
    "lib/prediction/basin.rs",
    '''        let node_id = NodeId::new(id);
        
        // Generate alternatives based on value type
''',
    '''        let node_id = NodeId::new(id);
        if self.nodes.len() >= self.max_nodes && !self.nodes.contains_key(&node_id) {
            return;
        }

        // Generate alternatives based on value type
''',
)
replace_once(
    "lib/prediction/basin.rs",
    '''        self.nodes.insert(node_id.clone(), BasinNode {
            id: node_id,
''',
    '''        self.nodes.insert(node_id, BasinNode {
''',
)
replace_once(
    "lib/prediction/basin.rs",
    '        let node = self.nodes.get(node_id)?;\n',
    '''        if !self.nodes.contains_key(node_id) {
            return None;
        }
''',
)

# Replace tests whose unsigned-length comparisons could never fail.
replace_once(
    "lib/prediction/basin.rs",
    '''        // Should have at least one prediction (the necessary truth)
        assert!(!predictions.is_empty() || predictions.len() >= 0);
''',
    '''        assert!(
            !predictions.is_empty(),
            "a causal constraint should produce an equilibrium prediction"
        );
''',
)
replace_once(
    "lib/input_normalizer/mod.rs",
    '''    fn test_only_emoji() {
        let result = n("❤️😂🤣");
        // Should process without crashing - output may contain text or markers
        assert!(result.uncertain_tokens.len() >= 0);
    }
''',
    '''    fn test_only_emoji() {
        let result = n("❤️😂🤣");
        assert!(result.markers.has_emoji);
    }
''',
)
replace_once(
    "lib/input_normalizer/mod.rs",
    '''    fn test_uncertain_tokens_tracked() {
        let result = n("xyzqwerty asdfghjkl");
        // These aren't common typos so should be flagged as uncertain
        // But the normalizer should NOT crash
        assert!(result.uncertain_tokens.len() >= 0);
    }
''',
    '''    fn test_uncertain_tokens_tracked() {
        let result = n("xyzqwerty asdfghjkl");
        assert!(!result.uncertain_tokens.is_empty());
    }
''',
)
