from pathlib import Path

path = Path(__file__).resolve().parents[2] / "lib/companion_bounded_live_policy.rs"
text = path.read_text()

old_call = "activate(0, &gate,"
count = text.count(old_call)
if count < 2:
    raise SystemExit(f"expected at least two remaining legacy activation calls, found {count}")
text = text.replace(old_call, "activate(0, &authorization,")

old_error = """            Err(LivePolicyError::PromotionSourceVersionMismatch {
                authorized: 3,
                actual: 4,
            })"""
new_error = """            Err(LivePolicyError::AuthorizationProposalMismatch)"""
if old_error not in text:
    raise SystemExit("stale-proposal expectation not found")
text = text.replace(old_error, new_error, 1)

path.write_text(text)
