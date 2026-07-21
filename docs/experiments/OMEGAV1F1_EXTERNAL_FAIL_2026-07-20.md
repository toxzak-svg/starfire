# ΩV1-F1 External Result: FAIL

**Execution date:** July 20, 2026  
**Executed source:** `bd518d44889dfde33dc47a65ac5705778e26957b`  
**Parent implementation merge:** `1915c63fe9a6acab5ba378b693de90a2f2d07a70`  
**External environment:** Render Docker builder  
**Terminal classification:** `FAIL`

## What succeeded

The evaluator compiled and executed through the complete 122-fixture corpus. The supplied external log confirms:

- exact split counts remained 74 training, 24 validation, and 24 test fixtures;
- stale digest and wrong-scope controls passed;
- typed-reference preservation remained `1.0`;
- zeroed VoiceState selections remained independently verifiable;
- the evaluator failed closed before image construction.

The failure did not authorize learned output or alter the production feature set.

## Observed failure

The held-out selected surfaces collapsed onto a mandatory epistemic phrase:

```json
{
  "terminal_classification": "FAIL",
  "top_template_trigram": "is possible that",
  "top_template_trigram_frequency": 0.8333333333333334,
  "top_template_trigram_relative_reduction": 0.0
}
```

The original grammar-v3 lattice exposed only one direct and one warm surface family for each operation. Because most held-out claims carried `EpistemicStatus::Possible`, the independent semantic marker became the dominant trigram across 20 of 24 outputs. The selector therefore preserved semantics but failed the frozen anti-template signal.

## Evaluator defect discovered after the run

Section 9 of the frozen F0 preregistration states that learned-signal metrics are measured **on the frozen held-out test set**. The implementation compared the 24 held-out learned outputs against ΩV1-A anti-template constants calculated over all 122 fixtures. This is a denominator mismatch.

The defect does not reverse the F1 result. The observed frequency of `0.8333333333333334` is a clear failure under either denominator. It must be corrected before another result is interpreted.

## Scientific disposition

ΩV1-F1 is permanently recorded as `FAIL`; it is not rewritten as PASS.

A separately identified remediation may:

1. retain all semantic, safety, determinism, resource, and authority floors;
2. compare learned outputs with the same 24 ΩV1-A test fixtures;
3. freeze a six-surface maximum containing three direct and three warm alternatives per operation;
4. require a second inverse verifier to reconstruct the original grammar-v3 candidate before acceptance;
5. preserve exact grammar-v2 neutral fallback;
6. state that the previously observed test set is no longer an untouched human-quality holdout.

No remediation may authorize live learned output. A remediation PASS can establish bounded mechanics only and must defer fresh human preference evidence to a later shadow stage.
