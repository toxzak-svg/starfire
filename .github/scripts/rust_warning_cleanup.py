# Reviewed, idempotent cleanup pass for PR #145.
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if new and new in text:
        return
    old_count = text.count(old)
    if old_count == 1:
        file.write_text(text.replace(old, new))
        return
    if old_count == 0:
        print(f"pattern already transformed or unavailable in {path}")
        return
    raise RuntimeError(f"ambiguous replacement state in {path}: old={old_count}")


def remove_once(path: str, old: str) -> None:
    file = Path(path)
    text = file.read_text()
    old_count = text.count(old)
    if old_count == 1:
        file.write_text(text.replace(old, ""))
        return
    if old_count == 0:
        return
    raise RuntimeError(f"ambiguous removal state in {path}: old={old_count}")


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
remove_once(
    "lib/prediction/basin.rs",
    '    pub id: NodeId,\n',
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

# Preserve raw-input markers while emoji replacement enriches them. Previously
# replace_emoji set has_emoji and detect_markers immediately overwrote it.
replace_once(
    "lib/input_normalizer/mod.rs",
    '''        let mut text = input.to_string();
        let mut markers = PersonalityMarkers::default();
        let mut uncertain_tokens = Vec::new();

        // Step 1: Extract and replace emoji
        text = self.replace_emoji(&text, &mut markers);

        // Step 2: Detect personality markers BEFORE normalization
        // (so we know what the raw input looked like)
        markers = self.detect_markers(input);
''',
    '''        let mut text = input.to_string();
        let mut markers = self.detect_markers(input);
        let mut uncertain_tokens = Vec::new();

        // Step 1: Extract and replace emoji while preserving raw-input markers.
        text = self.replace_emoji(&text, &mut markers);
''',
)

# Detect obvious keyboard-row runs as uncertain rather than silently accepting
# them as ordinary language. This is deliberately narrow to avoid flagging rare
# but legitimate words merely because they are not in a tiny dictionary.
replace_once(
    "lib/input_normalizer/mod.rs",
    '''        let lower = word.to_lowercase();

        // If word is in a dictionary, it's probably not a typo
''',
    '''        let lower = word.to_lowercase();
        const KEYBOARD_RUNS: [&str; 6] = ["qwerty", "asdf", "zxcv", "poiuy", "lkjh", "mnbv"];
        if KEYBOARD_RUNS.iter().any(|run| lower.contains(run)) {
            return true;
        }

        // If word is in a dictionary, it's probably not a typo
''',
)

# Cargo can remove imports that are unused in production but required by tests.
# Keep those dependencies next to the test modules that use them.
replace_once(
    "lib/world_model/prediction.rs",
    '    use crate::world_model::Entity;\n',
    '    use crate::world_model::{Entity, RelationType};\n',
)
replace_once(
    "lib/curriculum/scheduler.rs",
    '''mod tests {
    use super::*;
''',
    '''mod tests {
    use super::*;
    use crate::curriculum::{GapType, KnowledgeGap};
''',
)

# Give every persistence test its own SQLite database. The old shared test.db
# allowed parallel tests and prior runs to inflate snapshot counts.
replace_once(
    "lib/persistence/store.rs",
    '''mod tests {
    use super::*;

    fn test_store() -> Store {
        let dir = std::env::temp_dir().join("star_test_store");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.db");
        Store::open(&path).unwrap()
    }
''',
    '''mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_STORE: AtomicU64 = AtomicU64::new(0);

    fn test_store() -> Store {
        let unique = NEXT_TEST_STORE.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "star_test_store_{}_{}",
            std::process::id(),
            unique
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Store::open(&dir.join("test.db")).unwrap()
    }
''',
)

# Replace the tag-toggle HTML stripper with a test-scoped state machine that
# skips non-readable containers and inserts boundaries between adjacent blocks.
replace_once(
    "lib/research/web.rs",
    '''fn extract_readable_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut skip_until: Option<&str> = None;
    let mut index = 0;

    while index < html.len() {
        let rest = &html[index..];
        if let Some(tag) = skip_until {
            if rest.to_ascii_lowercase().starts_with(tag) {
                skip_until = None;
            }
        }
        if rest.starts_with("<script") || rest.starts_with("<style") {
            skip_until = Some("</");
        }
        let character = rest.chars().next().expect("index stays in bounds");
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag && skip_until.is_none() => text.push(character),
            _ => {}
        }
        index += character.len_utf8();
    }

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
''',
    '''#[cfg(test)]
fn extract_readable_text(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut output = String::with_capacity(html.len());
    let mut index = 0;
    let mut skipped_tag: Option<&str> = None;

    while index < html.len() {
        if let Some(tag) = skipped_tag {
            let closing = format!("</{tag}");
            if lower[index..].starts_with(&closing) {
                skipped_tag = None;
            } else {
                let ch = html[index..].chars().next().expect("index stays in bounds");
                index += ch.len_utf8();
                continue;
            }
        }

        if html.as_bytes()[index] == b'<' {
            let rest = &lower[index + 1..];
            let tag_start = rest.trim_start_matches('/');
            if ["script", "style", "title"].iter().any(|tag| tag_start.starts_with(tag))
                && !rest.starts_with('/')
            {
                skipped_tag = ["script", "style", "title"]
                    .into_iter()
                    .find(|tag| tag_start.starts_with(tag));
            }
            if !output.ends_with(char::is_whitespace) && !output.is_empty() {
                output.push(' ');
            }
            if let Some(end) = html[index..].find('>') {
                index += end + 1;
                continue;
            }
            break;
        }

        let ch = html[index..].chars().next().expect("index stays in bounds");
        output.push(ch);
        index += ch.len_utf8();
    }

    output.split_whitespace().collect::<Vec<_>>().join(" ")
}
''',
)
