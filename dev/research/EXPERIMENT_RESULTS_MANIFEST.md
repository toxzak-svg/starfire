# Experiment And Results Manifest

Generated: 2026-03-03

Companion files:
- `EXPERIMENT_RESULTS_MANIFEST.json` - machine-readable summary manifest
- `EXPERIMENT_RESULTS_INVENTORY.json` - deeper inventory for run trees, result bundles, and SQLite-backed outputs

Scope:
- Root folders examined: `214`, `attention`, `certain`, `circuit_lm`, `context`, `contextual-flux`, `future`, `sep-w`
- Ignored for inventory purposes: `.git`, virtualenvs, caches, and transient build/test directories
- Large archives were summarized instead of exhaustively inlined where the tree is very large

## 214

Primary project: `214/hgsel-moe`

Experiment sources:
- `214/hgsel-moe/experiments/`
- Key scripts include `benchmark_300m.py`, `benchmark_distributed_300m.py`, `benchmark_token_exchange_micro.py`, `expert_interference_benchmark.py`, `microbenchmark_all_to_all.py`, `tail_latency_decomposition.py`, `trace_driven_workset.py`, `phase4_gate_report.py`

Result artifacts:
- `214/hgsel-moe/results/`
- Checked-in outputs include 17 files: JSON smoke reports, PNG plots, and Phase 4 gate reports

Extracted result data:
- `results/interference_smoke.json`
  - Script: `experiments/expert_interference_benchmark.py`
  - Max absolute interference: `15.6146%`
  - Level: `moderate`
  - Verdict: `WARN`
  - Implication: soft partitioning / priority scheduling recommended
- `results/tail_latency_decomp_smoke.json`
  - Script: `experiments/tail_latency_decomposition.py`
  - Average forward-pass coefficient of variation: `0.0762`
  - Tail variation level: `predictable`
  - Verdict: `OK`
  - Recommendation: workload co-scheduling safe
- `results/workset_smoke.json`
  - Script: `experiments/trace_driven_workset.py`
  - Context lengths tested: `16`, `32`, `64`
  - Working set remained fixed at `32`
  - Overall CV: `0.0`
  - Verdict: `OK`
- `results/phase4/phase4_gate_report_smoke_with_ref.json`
  - Overall status: `warn`
  - All three gates (`baseline`, `ddp_parity`, `microbenchmark`) are warning-level, not stop-level
  - Baseline throughput (smoke, CPU): dense `1433.77 tok/s`, hgsel `452.50 tok/s`
  - DDP parity smoke is single-rank CPU only (`world_size=1`), so multi-GPU parity is not yet demonstrated

## attention

Experiment sources:
- Formal spec: `attention/experiment.md`
  - Version `1.1`
  - Date `2026-03-01`
  - Status: `Pre-implementation design`
- Runner: `attention/run_experiment.py`
- Code: `attention/src/experiment/`

Result artifacts:
- `attention/results/`
- 15 checked-in result bundles:
  - `diag_dagger`
  - `imitation_run_01`
  - `imitation_run_02`
  - `imitation_test`
  - `litmus_run_01`
  - `oracle_sanity`
  - `oracle_sanity_v2`
  - `oracle_sanity_v3`
  - `oracle_test_10`
  - `oracle_test_40`
  - `pretrain_run_01`
  - `test_diagnostic`
  - `test_dry`
  - `test_eval_free`
  - `train_diagnostic`

Extracted result data:
- `results/litmus_run_01/litmus_summary.json`
  - Episodes: `200`
  - Transformer-final accuracy: `1.0`
  - No-memory-final accuracy: `0.0225`
  - Gap: `0.9775`
  - `passed: true`
- `results/oracle_sanity_v3/sanity_check.json`
  - `converged: true`
  - `final_accuracy: 0.9`
  - `final_skip_frac: 0.7647`
  - `gru_shortcut_detected: false`
- `results/test_diagnostic/sanity_check.json`
  - `converged: false`
  - `final_accuracy: 0.3`
  - `final_skip_frac: 1.0`
  - Reads/writes collapsed to `0.0`
- `results/pretrain_run_01/summary.md`
  - Primary test decision: `H0 not rejected`
  - 60% distractor gap: `0.6 pp` vs required `10 pp`
  - Slope ratio: `0.00` vs required `2.0`
  - `B_oracle` is `100%` across distractor levels while learned variants remain near chance
- `results/train_diagnostic/summary.md`
  - Primary test decision: `H0 not rejected`
  - 60% distractor gap: `0.8 pp`
  - `B_rand` blocked fraction: `0.546`
- `results/oracle_sanity_v3/summary.md`
  - Marks the experiment as blocked
  - Recorded blockers:
    - Type-K distractors are effectively broken
    - Output head is untrained
    - Budget is not binding
- Most other checked-in summaries (`imitation_test`, `oracle_test_10`, `test_dry`, `test_eval_free`, `test_diagnostic`) report either `Insufficient data` or `H0 not rejected`
- `results/diag_dagger/diag_dagger_latest.pt` is a model checkpoint, not a human-readable summary

## certain

Experiment sources:
- `certain/src/metacog_confidence/experiments.py`
  - Defines `run_controller_sweep(...)`
  - Produces per-sweep `SweepRecord` rows including:
    - controller thresholds / gains
    - online vs sequential accuracy
    - decision rate
    - confidence
    - expected calibration error
    - mean steps
    - deltas between online and sequential baselines
- `certain/scripts/sweep_controller.py`
  - Default sweep grid:
    - `threshold_gain`: `0.2, 0.6, 1.0`
    - `threshold_relaxation`: `0.1, 0.3, 0.5`
    - `attention_gain`: `0.5, 1.0, 1.5`
    - `signal_strengths`: `0.2, 0.4, 0.8`
    - `repeats`: `8`
  - Can emit CSV via `--output`

Result artifacts:
- No checked-in logs, CSV sweep outputs, or result directories were found
- This folder contains experiment code, but no committed experiment outputs

## circuit_lm

Experiment sources:
- Benchmark/experiment scripts:
  - `circuit_lm/scripts/benchmark_small.py`
  - `circuit_lm/scripts/benchmark_matrix.py`
  - `circuit_lm/scripts/benchmark_serialization.py`
  - `circuit_lm/scripts/reproduce_depth_generalization.py`
  - `circuit_lm/scripts/robustness_experiment.py`
  - `circuit_lm/scripts/verify_joint_pda_small.py`
- `circuit_lm/scripts/robustness_experiment.py` defines four axes:
  - seed stability
  - token-ID invariance
  - noise robustness
  - stack-alphabet size

Result artifacts:
- No dedicated `results/` directory is checked in
- Benchmark outcomes are documented in `circuit_lm/STATUS.md`

Extracted result data from `STATUS.md`:
- Snapshot date: `2026-02-25`
- Full test suite: `199 passed`
- Baseline benchmark (`scripts/benchmark_small.py`)
  - `text_len=3105`
  - `effective vocab_size=30`
  - `sequences=13`
  - `total_tokens=3117`
  - `num_states=8`
  - `train_time=203ms`
  - `eval accuracy=20.84%` (`647/3104`)
- Depth-generalization experiment (`scripts/reproduce_depth_generalization.py`)
  - Train depth `<=3`, `300` train sequences, `100` test sequences per depth
  - Documented as a four-model comparison (`PDA-2ph`, `PDA-jt`, `FSM`, `PPM`)
  - The checked-in table shows PDA-2ph improving beyond the training depth while FSM/PPM degrade
- Joint-PDA small-scale verification (`scripts/verify_joint_pda_small.py`)
  - Joint-PDA discovered the push stack rule (`[PASS] joint-PDA discovered stack`)
  - Pop token was not discovered in the logged run
- JSON serialization benchmark (`scripts/benchmark_serialization.py`)
  - `fsm-sm`: `7020` bytes, `save_ms=1`, `load_ms=0`
  - `pda-sm`: `377610` bytes, `save_ms=23`, `load_ms=6`
  - `pda-md`: `739648` bytes, `save_ms=42`, `load_ms=10`

## context

Experiment sources:
- `context/experiments/`
- Main runners:
  - `run_gls.py`
  - `run_comparison.py`
  - `run_lambda_sweep.py`
- GLS modules:
  - `gls/compare.py`
  - `gls/datasets.py`
  - `gls/diagnostics.py`
  - `gls/lambda_sweep.py`
  - `gls/rnn.py`
  - `gls/spectral.py`
  - `gls/trainer.py`
  - `gls/vae.py`

Result artifacts:
- `context/artifacts/`
- Top-level artifact groups found:
  - `comparison_lorenz`
  - `comparison_sine`
  - `gls_smoke`
  - `lambda_sweep_lorenz`
  - `lambda_sweep_sine`
  - `run-baseline-1772173087430`
  - `run-current-anchor-benchmark-1771461512499`
  - `run-optimized-1772173340687`
  - `run-post-opt-1772177238749`
  - `smoke_highdim`
  - `smoke_multiseed`
  - `smoke_pca`
  - `stress_test_*` directories (many)
  - `archive/` (large historical run archive)
- The archive and stress-test tree are extensive; they were summarized rather than fully expanded here

Extracted result data:
- `artifacts/gls_smoke/summary.json`
  - `rho_rnn_warmup_final: 0.97185`
  - `rho_rnn_ema_final: 1.07919`
  - `rho_vae_final: 0.44131`
  - `stability_constraint_active_frac: 0.062`
  - `vae_inside_spectral_ball_final: true`
- `artifacts/comparison_lorenz/comparison_results.json`
  - Conditions: `gls`, `smf_unconstrained`, `wmf`
  - `gls`: `rho_vae_final=0.92406`, `kl_final=0.42807`, `recon_final=0.0557`, stability active `2.5%`
  - All three conditions report `kl_collapsed: false` and `vae_inside_ball_final: true`
- `artifacts/lambda_sweep_lorenz/lambda_sweep_results.json`
  - Sweep over `lambda_stab`: `0.0`, `0.05`, `0.1`, `0.5`, `1.0`, `2.0`, `5.0`
  - Across the full sweep: no KL collapse, VAE remains inside the spectral ball, stability activation rises from `0.0` to `0.03`
- `artifacts/smoke_multiseed/comparison_results_multiseed.json`
  - Multi-seed aggregates for `gls`, `smf_unconstrained`, `wmf`, `gls_fixed_0.99`
  - `gls` mean `rho_vae_final=0.80469`, mean `kl_final=0.30201`
  - `gls_fixed_0.99` increases stability activation to mean `0.22656`
- `artifacts/run-post-opt-1772177238749/metrics.json`
  - Proposed system:
    - `InstructionRetention@10k=1.0`
    - `CurrentStateAccuracy=1.0`
    - `StaleFactErrorRate=0.0`
    - `AnswerContradictionRate=0.0`
    - `ProvenanceExactMatch=1.0`
    - `CitationCoverage=1.0`
    - `LatencyP95=101.9189`
    - `TokenCostPerAnswer=5.3333`
  - Best visible baseline on state/citation metrics is weaker on `CurrentStateAccuracy` and `CitationCoverage`
- `artifacts/run-post-opt-1772177238749/bootstrap_ci.json`
  - `CurrentStateAccuracy` mean diff vs `long_context_prompt`: `0.517`
  - CI: `[0.1667, 0.8333]`
  - Retention/provenance/contradiction diffs are all `0.0`
- `artifacts/stress_test_report_v2.json`
  - Total tests: `350`
  - Passed: `110`
  - Failed: `240`
  - Category breakdown:
    - `long_horizon`: `10/14`
    - `contradiction`: `4/28`
    - `provenance`: `20/28`
    - `stress_rapid_change`: `2/42`
    - `stress_cascading_update`: `2/56`
    - `stress_multi_entity_conflict`: `30/56`
    - `stress_extreme_long_horizon`: `18/28`
    - `stress_ambiguous_reference`: `20/42`
    - `stress_temporal_inconsistency`: `4/56`
  - Includes many baseline ingest failures, especially for `graph_rag`

## contextual-flux

This folder is an umbrella with multiple experiment-bearing subprojects.

### bigger-context-runtime

Experiment sources:
- `contextual-flux/bigger-context-runtime/experiments/`
- 12 numbered experiments are defined in separate folders, each with `README.md` and `harness.py`
- Global trackers:
  - `contextual-flux/bigger-context-runtime/EXPERIMENTS.md`
  - `contextual-flux/bigger-context-runtime/results/EXPERIMENT_REPORT.md`
  - `contextual-flux/bigger-context-runtime/results/RUN_METADATA_ROLLUP.json`

Result artifacts:
- `contextual-flux/bigger-context-runtime/results/`
- Current filesystem contains `31` run directories across the 12 experiments
- Current metadata scan across `metadata.json` files shows:
  - `26` completed
  - `5` failed
  - `5` with an `error` field

Per-experiment run directory counts:
- `01-latency-truth-coupling`: `2`
- `02-congestion-reasoning-collapse`: `6`
- `03-certificate-bottleneck`: `2`
- `04-state-beats-summary`: `4`
- `05-retrieval-interference`: `3`
- `06-recompute-offload-boundary`: `3`
- `07-adversarial-diversity-conservation`: `1`
- `08-mechanistic-atom-transportability`: `1`
- `09-mode-locking-hypothesis`: `6`
- `10-hidden-tool-use`: `1`
- `11-memory-as-control`: `1`
- `12-optimal-abstention-frontier`: `1`

Important consistency note:
- `results/RUN_METADATA_ROLLUP.json` is stale relative to the current filesystem
  - Rollup says: `24` total, `20` completed, `4` failed, `4` with error
  - Current on-disk metadata says: `31` total, `26` completed, `5` failed, `5` with error

Extracted result data:
- `results/EXPERIMENT_REPORT.md`
  - All 12 experiments executed
  - `4` confirmed: `04`, `05`, `06`, `11`
  - `3` not confirmed: `02`, `03`, `09`
  - `5` require further investigation: `01`, `07`, `08`, `10`, `12`
- Strongest checked-in confirmations:
  - `04 State Beats Summary`: structured state correctness `0.907` vs summary `0.760`, contradiction reduction `-1.8`, edit locality `+0.200`
  - `05 Retrieval Interference`: mean decline after optimal K `35.0%`
  - `06 Recompute-Offload Boundary`: final regret `0.0127`, transfer regret `0.0000`
  - `11 Memory as Control`: theorem holds, improvement `+30%`
- `contextual-flux/results/EXECUTION_TRACKER.md` adds later status notes:
  - Experiment `09` has later provisional trace-replay runs where `boundary_detected=True`
  - Logged bootstrap trace replay: `avg_drift=0.31857777777777774`, `measurement_count=90`
  - That tracker is newer than the older “not confirmed” summary in `EXPERIMENT_REPORT.md`

### layered-context-experiment

Experiment sources:
- `contextual-flux/layered-context-experiment/`
- Core implementation plus tests and demos

Result artifacts:
- No separate result directory, but checked-in summary exists at `contextual-flux/layered-context-experiment/SUMMARY.md`

Extracted result data:
- `8/8` integration tests passing
- Demo snapshot:
  - Tier 0: `9` messages, `1865` tokens
  - Tier 1: `2` symbols, `1` constraint, `1` decision
  - Tier 2: `17` chunks indexed
  - Tier 3: `22` archived entries
- Reported token reduction: `87.6%`
- Reported retrieval time: `<10ms` for `100` chunks

### smart-context

Experiment sources:
- `contextual-flux/smart-context/`
- Benchmark/readme claims are documented in `README.md`

Result artifacts:
- `contextual-flux/smart-context/results/ollama_comparison_results.xml`

Extracted result data:
- `ollama_comparison_results.xml`
  - Test run timestamp: `2026-02-24T17:25:08.280462-05:00`
  - `7` tests
  - `0` failures
  - `0` errors
- `smart-context/README.md`
  - Claimed token reduction: `87.6%`
  - Claimed retrieval speed: `<50ms` for `10K` chunks
  - Claimed compilation time: `<100ms`
- `contextual-flux/results/EXEC_SUMMARY.md`
  - Notes a coverage baseline of `35.94%` for `smart-context`

### contextual-flux root rollups

Result artifacts:
- `contextual-flux/results/EXEC_SUMMARY.md`
- `contextual-flux/results/EXECUTION_TRACKER.md`

Extracted result data:
- `EXEC_SUMMARY.md` portfolio snapshot:
  - `bigger-context-runtime`: 12 experiments, strongest signals in `04`, `05`, `06`, `11`
  - `layered-context-experiment`: `8/8` tests passing
  - `smart-context`: `7/7` tests passing
- `EXECUTION_TRACKER.md`
  - Reliability workstream marked UTF-8-safe metadata writes complete
  - Failed-batch reruns were executed and the rollup was refreshed at that time

## future

This folder does not use explicit `experiments/` or `results/` directories, but it contains persisted forecasting output in SQLite.

Result-bearing artifacts:
- `future/forecaster.db`
- `future/forecaster.db-wal`
- `future/forecaster.db-shm`
- Runtime configuration: `future/config.json`

Extracted result data from `forecaster.db`:
- Tables present:
  - `articles`
  - `cluster_articles`
  - `clusters`
  - `events`
  - `forecast_audit`
  - `forecast_probability_history`
  - `forecasts`
  - `resolution_audit`
  - `schema_migrations`
- Row counts:
  - `articles`: `185`
  - `cluster_articles`: `185`
  - `clusters`: `184`
  - `events`: `183`
  - `forecast_audit`: `106`
  - `forecast_probability_history`: `0`
  - `forecasts`: `80`
  - `resolution_audit`: `9`
  - `schema_migrations`: `1`
- Forecast status summary:
  - `OPEN`: `71`
  - `REJECTED`: `7`
  - `RESOLVED`: `2`
- Resolved forecast quality:
  - Resolved forecasts with Brier score: `2`
  - Average Brier: `0.7225`
- Open forecasts already due by `2026-03-03T00:00:00+00:00`: `67`
- Domain distribution in `forecasts`:
  - `general`: `20`
  - `sports`: `20`
  - `international`: `19`
  - `business`: `9`
  - `politics`: `8`
  - `usa`: `4`
- Sample checked-in rows include both unresolved manual geopolitical forecasts and resolved event-linked forecasts

Configuration notes from `config.json`:
- `auto_resolve_enabled: true`
- `reprice_enabled: true`
- `calibration_enabled: true`
- `ollama_model: llama3.2:3b`

## sep-w

Experiment sources:
- No dedicated `experiments/` or `results/` directories
- Package code and tests only:
  - `sep-w/sepw/`
  - `sep-w/tests/test_integration.py`

Result artifacts:
- No checked-in experiment outputs, benchmarks, manifests, or result bundles were found

## Summary

Folders with substantial checked-in experiment/result data:
- `214`
- `attention`
- `circuit_lm` (documented in `STATUS.md`)
- `context`
- `contextual-flux`
- `future` (SQLite-backed runtime results)

Folders with experiment code but no checked-in result outputs:
- `certain`

Folders with no explicit checked-in experiment/result artifacts:
- `sep-w`
