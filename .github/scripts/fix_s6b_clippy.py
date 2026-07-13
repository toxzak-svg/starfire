from pathlib import Path

path = Path(__file__).resolve().parents[2] / "lib/companion_bounded_live_policy.rs"
text = path.read_text()

replacements = [
    (
        "claim_ids.is_empty() || claim_ids.iter().any(|claim_id| *claim_id == 0)",
        "claim_ids.is_empty() || claim_ids.contains(&0)",
    ),
    (
        "lease.source_claim_ids.iter().any(|claim_id| *claim_id == 0)",
        "lease.source_claim_ids.contains(&0)",
    ),
    (
        """fn intent_allowed(intent: &ResponseIntent) -> bool {
    intent_label_allowed(intent.label())
}

""",
        "",
    ),
]
for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one match for {old!r}, found {count}")
    text = text.replace(old, new, 1)

path.write_text(text)
