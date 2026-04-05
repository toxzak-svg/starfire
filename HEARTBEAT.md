# HEARTBEAT.md — Periodic Background Tasks

Star's heartbeat system runs periodic background tasks to maintain continuity and self-improvement.

## Active Tasks

### Health Monitoring
- **Every 5 minutes**: Check Star API health endpoint
- **Every 10 minutes**: Verify runtime not stuck in reasoning loop

### Memory & Learning
- **Every hour**: Run memory consolidation (decay old, strengthen important)
- **Every 6 hours**: Analyze prediction accuracy, update belief confidence

### Research & Gaps
- **Every day**: Check knowledge gaps, generate curiosity probes for new topics

### Testing
- **Every day**: Run integration test suite, report failures

## Format

```yaml
task:
  name: "Health check"
  every: 5m
  action: GET /health
  on_fail: log
  
task:
  name: "Memory consolidation"
  every: 1h
  action: call consolidate_memories()
```

## Current Config

```yaml
# Health check - verify Star is responding
- every: 5m
  endpoint: /health
  
# Unit test pass rate - catch regressions
- every: 1d
  command: cargo test --workspace -- --no-fail-fast
  alert_if_fail: true
  
# Integration test status
- every: 1d  
  command: cargo test --workspace -- --test-threads=1
  alert_if_fail: true
```

## Notes

- Tasks run in background, non-blocking
- Failures logged but don't crash Star
- Can be disabled by clearing this file (keeps header only)
