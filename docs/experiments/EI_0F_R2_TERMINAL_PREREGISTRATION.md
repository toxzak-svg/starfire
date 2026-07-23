# EI-0F-R2 Schema-Bound Terminal Remediation

> **Status:** frozen specification, no terminal result executed  
> **Preregistration ID:** `ei-0f-remediation-v2`  
> **Freeze base:** `f400a6a139e1a8820f80310b86b84d2b124de1fd`  
> **Canonical manifest SHA-256:** `89909b52cadd394207bafc7526e992a3c20ca0a923e35c2bea7290a306eefec5`  
> **Report schema Git blob:** `7ef8bb3d72a8ad6f2219dd62d2f8d4c0f2954d43`  
> **Runner Git blob:** `2c9663ab2e01152fc9c83e8fc818e3e848d54bc8`

## Reason for V2

The unexecuted V1 freeze reused a schema whose preregistration constant named the original EI-0F experiment. V2 preserves V1 as invalid and creates a schema whose identifier and preregistration constant match this package.

## Frozen changes

The runner differs from the original only by preregistration identity, manifest digest, and the already-preflighted proposal-digest normalization. The report schema differs from the original only by its `$id` and preregistration constant.

## Boundary

No terminal output has been produced or inspected. Tasks, arms, seeds, budgets, evaluators, thresholds, classifier semantics, safety gates, and authority remain unchanged. EI-0G remains unauthorized.
