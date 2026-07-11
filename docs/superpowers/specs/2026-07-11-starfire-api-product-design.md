# Starfire API Product Design

**Date:** 2026-07-11

**Status:** Approved design

**Scope:** Initial product specification for a managed API optimization gateway and evidence-driven code patch system.

## Product Summary

Starfire is a managed control plane and gateway that improves OpenAI-compatible and configured REST API integrations. A user stores an API credential, points application traffic at Starfire, chooses an autonomy policy, and receives measurable improvements to quality, reliability, latency, and cost.

Starfire improves the request pipeline around an external API. It does not claim to change a provider's model weights, private infrastructure, billing configuration, or internal behavior.

The core product advantage is a closed evidence loop:

1. Observe representative traffic and establish a baseline.
2. Identify a bounded opportunity.
3. Test a reversible gateway intervention on eligible traffic.
4. Measure the result against explicit objectives and guardrails.
5. Promote, reject, or roll back the intervention.
6. When appropriate, turn proven interventions into durable application code patches.
7. Verify the deployed patch and retire redundant gateway logic.

## Goals

- Provide a drop-in OpenAI-compatible gateway.
- Proxy explicitly configured generic REST endpoints.
- Store provider credentials securely on the Starfire server.
- Improve only behavior Starfire can observe and evaluate.
- Give users explicit control over how changes are approved.
- Make every change attributable, measurable, and reversible.
- Convert proven runtime interventions into reviewable repository patches.
- Preserve conservative claim boundaries when evidence is insufficient.

## Non-Goals for the Initial Release

- Modifying provider model weights or provider-side infrastructure.
- General-purpose autonomous software development.
- Automatically discovering and calling arbitrary internet endpoints.
- Automatically executing destructive REST operations.
- Automatically merging or deploying security-sensitive code changes.
- Supporting every proprietary LLM API shape. The initial LLM interface is OpenAI-compatible.
- Claiming output-quality improvements without an application-specific evaluator or user feedback signal.

## Delivery Sequence

### Phase 1: Gateway and Control Plane

Ship the OpenAI-compatible gateway, configured REST proxy, encrypted credential vault, project onboarding, autonomy controls, metrics, experiments, audit ledger, and rollback.

### Phase 2: Evidence-Driven Code Patches

Connect repositories and create targeted branches or pull requests based on measured gateway findings. Patches include tests, supporting evidence, expected impact, and rollback instructions.

### Phase 3: Closed-Loop Deployment Verification

Optionally connect CI and deployment systems. Verify production behavior after an approved patch, then retire the corresponding gateway workaround when it is no longer needed.

Automatic merge or deployment is not enabled merely by selecting API Auto mode. Repository automation has separate permissions and guardrails.

## Supported Integrations

### OpenAI-Compatible APIs

Applications change their API base URL to a project-specific Starfire endpoint and continue using an OpenAI-compatible SDK. The first release prioritizes chat/completion-style requests, structured outputs, and streaming passthrough.

Provider-specific fields that Starfire does not understand are preserved unless a documented policy explicitly transforms them.

### Generic REST APIs

Users register an upstream origin and an allowlist of paths and methods. They may provide an OpenAPI document or define request and response schemas manually. Starfire issues project-specific proxy URLs and rejects traffic outside the configured host, path, and method policy.

## Improvement Catalog

### OpenAI-Compatible Traffic

Starfire may:

- restructure prompts for clarity and consistency;
- remove redundant context to reduce token usage;
- choose a configured model based on request class;
- tune temperature, reasoning effort, maximum output, and related parameters;
- add approved memory or retrieved context;
- split complex work into planning, execution, and verification stages;
- enforce structured output schemas and repair safe formatting failures;
- add a verification pass for eligible high-uncertainty or high-stakes requests;
- retry with a corrected prompt or configured fallback model;
- route simple tasks to cheaper models and difficult tasks to stronger models;
- cache semantically equivalent requests when the policy marks them cache-safe;
- detect prompt injection, secret leakage, sensitive data, and unsupported claims;
- run bounded experiments and retain a change only when it meets promotion criteria.

### Generic REST Traffic

Starfire may:

- add timeouts, bounded retries, exponential backoff, and jitter;
- respect rate-limit headers and control concurrency;
- validate requests and responses against configured schemas;
- repair explicitly allowed formatting errors such as dates, enums, or optional defaults;
- refresh authentication for separately configured supported token flows;
- apply circuit breakers and configured fallback endpoints;
- deduplicate requests and add idempotency keys where the upstream contract supports them;
- cache eligible safe reads using endpoint-specific policies;
- improve batching or pagination using an explicit endpoint contract;
- detect contract drift;
- redact secrets and configured sensitive fields from telemetry;
- track latency, cost, errors, and user-defined quality signals;
- recommend payload or endpoint changes supported by measured evidence.

### Application Code

After a gateway intervention has sufficient evidence, Starfire may propose a code patch that:

- replaces brittle direct API calls with a configured client abstraction;
- consolidates retry, timeout, schema, streaming, and error-handling behavior;
- improves prompts or structured-output definitions;
- removes redundant calls or payload context;
- adds caching, batching, pagination, fallbacks, or observability;
- updates tests and configuration required by the change.

Code patches are narrowly scoped to an evidenced issue. They do not include unrelated refactoring.

## Autonomy Model

Each project has one ongoing API change mode:

- **Ask:** Starfire proposes every change and waits for approval before applying it.
- **Safe only:** Starfire automatically applies reversible, low-risk changes within hard guardrails and asks about all other changes.
- **Auto:** Starfire may apply broader changes within user-defined cost, latency, quality, error-rate, traffic, and endpoint limits.

**Approve all** is a one-time action that approves every currently eligible queued recommendation. It does not change the project's ongoing mode. Ineligible, destructive, expired, conflicted, or policy-blocked recommendations remain unapplied.

Repository actions have a separate policy. In the initial release, Starfire creates reviewable branches or pull requests. Auto-merge and deployment require later, explicit repository and deployment permissions and cannot be inferred from the API change mode.

### Risk Classification

Examples normally eligible for Safe only include bounded retry and backoff corrections, schema validation, log redaction, conservative cache behavior, and reversible parameter tuning inside existing limits.

Examples normally requiring approval include model changes, important prompt rewrites, added LLM calls, payload transformations, new fallback origins, meaningful cost increases, repository changes, and deployment actions.

Credential changes, billing changes, destructive REST calls, and security-sensitive repository changes are never silently applied.

## User Flow

1. **Create project:** Name the application and select OpenAI-compatible traffic, REST traffic, or both.
2. **Connect credentials:** Add, test, rotate, disable, or delete encrypted credentials.
3. **Connect traffic:** Receive an OpenAI-compatible base URL, configured REST proxy URLs, and optional JavaScript or Python snippets.
4. **Connect repository:** Optionally authorize narrowly scoped repository access for future patches.
5. **Set autonomy and guardrails:** Choose Ask, Safe only, or Auto and configure cost, latency, quality, error, and traffic limits.
6. **Observe baseline:** Collect sufficient representative traffic before promoting optimizations.
7. **Review improvements:** Inspect proposed, experimenting, applied, rejected, inconclusive, and rolled-back changes.
8. **Apply durable fixes:** Review evidence-backed code patches and their verification status.

## Product Interface

The control plane has five primary areas:

- **Overview:** service health, quality signals, latency, cost, error rate, active experiments, and guardrail status.
- **Traffic:** requests grouped by model, endpoint, operation, experiment, and outcome, subject to retention and redaction policy.
- **Improvements:** recommendations and experiments grouped by status, with evidence, affected traffic, approval controls, and rollback.
- **Code:** connected repositories, generated patches, checks, approvals, deployments, and post-deployment verification.
- **Settings:** credentials, endpoint contracts, autonomy, budgets, privacy, retention, team access, and failover behavior.

The interface must answer four questions without requiring log inspection:

1. What is underperforming?
2. What changed or is proposed to change?
3. Did the change measurably help?
4. How can it be rolled back?

## Architecture

### Control Plane

Manages organizations, projects, users, roles, credentials, endpoint contracts, policies, recommendations, experiments, patches, audit records, and retention settings.

### Gateway Data Plane

Authenticates Starfire project requests, enforces endpoint allowlists, resolves credentials, applies the active policy, proxies upstream traffic, supports streaming, records redacted telemetry, and executes failover behavior.

The gateway must not depend on the interactive dashboard being available.

### Credential Vault

Uses envelope encryption. Each credential is encrypted with a distinct data-encryption key; the encrypted data key is protected by a managed key-encryption key. The browser can submit a credential but cannot retrieve its plaintext later. Decryption is limited to an authorized gateway execution path.

Credential metadata, ciphertext, and audit records are stored separately enough that listing metadata does not expose encrypted material unnecessarily.

### Policy and Evaluation Engine

Classifies traffic, selects eligible interventions, enforces autonomy and guardrails, assigns experiment cohorts, evaluates outcomes, and recommends promotion or rollback.

Quality is application-specific. Starfire can use schema validity, deterministic tests, user feedback, task evaluators, or user-provided scoring callbacks. Without an appropriate quality signal, Starfire may optimize operational metrics but must label output-quality conclusions as unavailable or inconclusive.

### Change and Experiment Ledger

Every recommendation and applied change records:

- original behavior and configuration;
- proposed or applied transformation;
- reason and supporting evidence;
- autonomy decision and approving actor;
- affected traffic and experiment allocation;
- baseline and result metrics;
- promotion, rejection, or rollback decision;
- rollback procedure and outcome;
- links to related patches and deployments.

### Repository Patch Agent

Receives an evidenced finding and a narrowly scoped patch objective. It works on a branch, adds or updates tests, runs configured checks, and creates a reviewable patch or pull request. It does not receive provider credentials unless a separately authorized test requires a scoped test credential.

### Deployment Verification

When connected, records the deployed revision and compares post-deployment behavior with the relevant baseline and experiment. A failed or inconclusive verification does not mark the improvement complete.

## Core Data Model

- **Organization:** tenant and billing boundary.
- **Project:** application-level policy, endpoints, metrics, and retention boundary.
- **Credential:** encrypted secret, provider type, status, timestamps, and rotation metadata.
- **Endpoint:** upstream origin, allowed paths and methods, schema, timeout, retry, caching, and failover policy.
- **Request trace:** redacted operational metadata and optional retained payload sample.
- **Evaluator:** definition of an operational or quality measurement.
- **Recommendation:** proposed change, risk class, evidence, expected effect, and eligibility state.
- **Experiment:** cohorts, traffic allocation, metrics, thresholds, duration, and status.
- **Change:** applied configuration or transformation with prior state and rollback state.
- **Repository connection:** scoped installation identity and allowed repositories.
- **Patch:** branch or pull request, files, tests, evidence, checks, and lifecycle state.
- **Audit event:** immutable actor, action, target, policy decision, timestamp, and correlation identifier.

## Security and Privacy

- Enforce tenant isolation and authorization at every service boundary.
- Never return stored credential plaintext to a client.
- Redact secrets and user-configured sensitive fields before persistent logging.
- Allow independent retention settings for metadata, payload samples, and evaluation results.
- Require upstream host, path, and method allowlists to prevent server-side request forgery.
- Do not infer, replay, or experiment on destructive REST methods.
- Use narrowly scoped repository installation tokens.
- Record credential access, proxy actions, experiments, approvals, patches, and rollbacks in the audit ledger.
- Avoid placing provider keys in patch-agent prompts, logs, source code, or generated commits.
- Support credential rotation and immediate disablement.

## Failure Handling

- Each endpoint explicitly selects **fail closed** or **fail open**. Fail open bypasses optional optimization but still uses the authorized upstream configuration; it never exposes credentials to the calling client.
- Bounded retry budgets prevent retry storms and unexpected cost multiplication.
- Circuit breakers isolate failing upstreams or optimization components.
- Streaming errors preserve an attributable terminal status even when a full retry is impossible.
- Experiments use limited traffic and automatic rollback thresholds.
- A recommendation requires a defined baseline, objective, eligibility policy, and rollback action.
- Conflicting goals follow user-defined priority order with hard ceilings.
- Insufficient evidence is reported as inconclusive.
- Credential failures disable only the affected connection and expose no secret material.
- Failed tests, policy checks, or deployment verification block patch progression.

## API Surface

The precise endpoint naming may evolve during implementation planning, but the product requires these stable capability groups:

- project, membership, and role management;
- credential creation, testing, rotation, disablement, and deletion;
- OpenAI-compatible proxy endpoints;
- registered REST proxy endpoints;
- endpoint contract and policy configuration;
- metrics, traces, evaluators, and baselines;
- recommendations, approvals, experiments, changes, and rollbacks;
- repository connections, patches, checks, and deployment verification;
- audit export.

Public management APIs are authenticated independently from project proxy tokens. Provider credentials are never used as Starfire authentication tokens.

## Observability and Claim Rules

Starfire tracks request count, success and error rate, latency distributions, provider usage or estimated cost, retries, schema validity, cache outcomes, experiment assignment, and available quality signals.

Every improvement claim names the compared baseline, metric, traffic population, evidence window, and confidence or sample limitation. Starfire does not collapse lower cost, lower latency, schema validity, and task quality into one unlabeled score.

An experiment outcome is one of promoted, rejected, rolled back, or inconclusive. A usable configuration and a proven quality improvement are reported as distinct outcomes.

## Testing Strategy

### Unit and Property Tests

Cover policy decisions, risk classification, guardrails, redaction, routing, schema validation, caching eligibility, encryption boundaries, cohort assignment, metric computation, and rollback decisions.

### Contract Tests

Run supported OpenAI SDK calls against Starfire and a mock upstream, including streaming, structured output, provider-specific passthrough fields, errors, and rate limits.

### REST Integration Tests

Use mock providers to verify allowlists, schemas, retries, timeouts, pagination, idempotency, caching, authentication failures, circuit breakers, contract drift, and upstream outages.

### End-to-End Tests

Exercise project creation, credential storage, traffic connection, baseline collection, recommendation, approval, experiment, promotion, rollback, patch creation, and verification.

### Security Tests

Test tenant isolation, authorization, SSRF prevention, credential non-disclosure, log redaction, audit integrity, replay resistance, token scope, and deletion or disablement behavior.

### Patch-Agent Tests

Use disposable repositories containing representative OpenAI-compatible and REST integrations. Verify scoped diffs, tests, policy blocks, failed checks, and rollback documentation.

### Product Benchmark

Maintain a reproducible demonstration comparing baseline and optimized traffic across operational metrics and explicitly configured quality evaluators. Smoke artifacts cannot be presented as general production-quality proof.

## Initial Release Acceptance Criteria

- A user can create a project and securely store, test, rotate, disable, and delete a provider credential.
- An existing OpenAI-compatible client works after changing its base URL and Starfire project token.
- A configured REST endpoint rejects traffic outside its host, path, and method allowlist.
- Ask, Safe only, Auto, and the one-time Approve all action behave as specified.
- Every applied change has evidence, an audit trail, prior state, and a working rollback path.
- Experiments cannot exceed configured traffic, cost, latency, or error guardrails.
- Starfire distinguishes operational improvements from output-quality improvements.
- Secrets do not appear in application logs, telemetry, patches, or API responses.
- A measured finding can produce a scoped, tested, reviewable repository patch.
- Failure of Starfire or an upstream follows the endpoint's explicit fail-open or fail-closed policy.

## Deferred Decisions

The following are intentionally deferred to implementation planning because they do not alter the approved product behavior:

- cloud vendor and managed key service;
- relational database and telemetry store selection;
- job queue implementation;
- repository provider order beyond the first supported integration;
- billing model;
- exact frontend component system;
- exact management API route names.

These decisions must preserve the security, evidence, autonomy, and rollback requirements in this design.
