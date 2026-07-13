from pathlib import Path

path = Path(__file__).resolve().parents[2] / "lib/examples/s6a_bounded_live_policy_probe.rs"
text = path.read_text()
old = """        Err(LivePolicyError::PromotionSourceVersionMismatch {
            authorized: 7,
            actual: 8,
        })"""
new = """        Err(LivePolicyError::AuthorizationProposalMismatch)"""
if text.count(old) != 1:
    raise SystemExit(f"expected one stale S6-A mismatch assertion, found {text.count(old)}")
path.write_text(text.replace(old, new, 1))
