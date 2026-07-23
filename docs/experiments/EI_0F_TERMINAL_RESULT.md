# EI-0F Frozen Terminal Result

> **Classification:** `FAIL`  
> **Preregistration:** `ei-0e-terminal-v1`  
> **Execution commit:** `5c4fded7eda16cbf3a6673880557c2242e430c14`  
> **Failed workflow run:** `30027946179`  
> **Report SHA-256:** `9680309edf7267215229c05b6fbb5fb4ccf14ec1ce260609cb806f4481b7c0e0`

## Result

The first qualifying frozen terminal execution exited nonzero before emitting a report. The preregistered fail-closed rules therefore classify EI-0F as `FAIL`. The experiment was not rerun or repaired under the same preregistration identifier.

Frozen classifier failures: `arm_budget_mismatch`, `causal_chain_count`, `crash`, `evaluator_not_independent`, `fixed_policy_evaluation_count`, `harmful_challenge_count`, `harmful_detection`, `harmful_exact_rollback`, `incomplete_run`, `learning_evaluation_count`, `learning_update_count`, `memory_disabled_evaluation_count`, `missing_evaluations`, `no_update_evaluation_count`, `primary_advantage_vs_fixed_policy`, `primary_advantage_vs_memory_disabled`, `primary_advantage_vs_no_update`, `primary_advantage_vs_random_update`, `random_update_evaluation_count`, `renamed_transfer_vs_fixed_policy`, `renamed_transfer_vs_memory_disabled`, `renamed_transfer_vs_no_update`, `renamed_transfer_vs_random_update`.

## Failure excerpt

- `thread 'main' (2356) panicked at lib/examples/ei_0f_terminal_experiment.rs:660:24:`
- `EI-0F terminal experiment must emit a complete report or fail: InvalidDigestText("learning proposal digest")`
- `##[error]Process completed with exit code 101.`

## Preserved evidence

- Frozen source and freeze-lock verification passed before execution.
- The exact terminal command was invoked once.
- The failed job log is preserved in `EI_0F_TERMINAL_EXECUTION_FAILURE.log`.
- No second qualifying execution was performed.
- Authority remained closed.

## Claim boundary

EI-0 did not pass. EI-0G is not authorized. Any remediation requires a new preregistration identifier while preserving this result.
