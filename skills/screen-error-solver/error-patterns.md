# Error Pattern Library

Comprehensive regex patterns for detecting errors across languages, frameworks, and tools.
The screen watcher uses these patterns for OCR-based error detection (Strategy A and the first pass of Hybrid mode).

## Python

| Pattern | Type | Severity |
|---------|------|----------|
| `Traceback \(most recent call last\)` | python_traceback | critical |
| `(\w+Error): (.+)` | python_error | critical |
| `(\w+Exception): (.+)` | python_exception | critical |
| `SyntaxError: (.+)` | syntax_error | critical |
| `IndentationError: (.+)` | indentation_error | critical |
| `ModuleNotFoundError: No module named '(.+)'` | import_error | critical |
| `ImportError: cannot import name '(.+)'` | import_error | critical |
| `DeprecationWarning: (.+)` | deprecation | warning |
| `FutureWarning: (.+)` | future_warning | warning |
| `RuntimeWarning: (.+)` | runtime_warning | warning |

## JavaScript / TypeScript

| Pattern | Type | Severity |
|---------|------|----------|
| `Uncaught (\w+Error): (.+)` | js_uncaught | critical |
| `(TypeError\|ReferenceError\|SyntaxError): (.+)` | js_error | critical |
| `error TS(\d+): (.+)` | typescript_error | critical |
| `Cannot find module '(.+)'` | node_module_error | critical |
| `ERR_MODULE_NOT_FOUND` | node_module_error | critical |
| `ERR_REQUIRE_ESM` | esm_error | critical |
| `FATAL ERROR: .+ JavaScript heap out of memory` | heap_oom | critical |
| `UnhandledPromiseRejectionWarning` | unhandled_promise | warning |

## Rust

| Pattern | Type | Severity |
|---------|------|----------|
| `error\[E(\d+)\]: (.+)` | rust_error | critical |
| `panicked at (.+)` | rust_panic | critical |
| `warning\[(.+)\]: (.+)` | rust_warning | warning |
| `cannot find .+ in this scope` | rust_scope | critical |
| `mismatched types` | rust_type | critical |

## Go

| Pattern | Type | Severity |
|---------|------|----------|
| `panic: (.+)` | go_panic | critical |
| `fatal error: (.+)` | go_fatal | critical |
| `undefined: (.+)` | go_undefined | critical |
| `cannot use .+ as .+ in` | go_type | critical |

## C / C++

| Pattern | Type | Severity |
|---------|------|----------|
| `segmentation fault` | segfault | critical |
| `undefined reference to` | linker_error | critical |
| `error: (.+)` | c_error | critical |
| `warning: (.+)` | c_warning | warning |
| `SIGSEGV` | signal_segv | critical |
| `SIGABRT` | signal_abort | critical |

## Build Systems

| Pattern | Type | Severity |
|---------|------|----------|
| `BUILD FAILED` | build_failure | critical |
| `FAILED\|FAILURE` | generic_failure | critical |
| `make: \*\*\* .+ Error \d+` | make_error | critical |
| `npm ERR!` | npm_error | critical |
| `yarn error` | yarn_error | critical |
| `cargo error` | cargo_error | critical |
| `gradle .+ FAILED` | gradle_error | critical |
| `cmake Error` | cmake_error | critical |
| `Compilation failed` | compilation_failed | critical |

## Terminal / Shell

| Pattern | Type | Severity |
|---------|------|----------|
| `command not found` | command_not_found | critical |
| `No such file or directory` | file_not_found | critical |
| `Permission denied` | permission_denied | critical |
| `Operation not permitted` | operation_denied | critical |
| `is not recognized as .+ command` | windows_cmd_not_found | critical |
| `killed` | process_killed | critical |
| `Killed \(out of memory\)` | oom_killed | critical |

## Network / HTTP

| Pattern | Type | Severity |
|---------|------|----------|
| `ECONNREFUSED` | connection_refused | critical |
| `ECONNRESET` | connection_reset | warning |
| `ETIMEDOUT` | timeout | warning |
| `ERR_CONNECTION_REFUSED` | connection_refused | critical |
| `fetch failed` | fetch_error | critical |
| `CORS` | cors_error | warning |
| `SSL_ERROR` | ssl_error | critical |
| `certificate .+ expired` | cert_expired | critical |
| `getaddrinfo ENOTFOUND` | dns_error | critical |

## Docker / Containers

| Pattern | Type | Severity |
|---------|------|----------|
| `exited with code (\d+)` | docker_exit | critical |
| `OOMKilled` | oom_killed | critical |
| `no space left on device` | disk_full | critical |
| `Cannot connect to the Docker daemon` | docker_daemon | critical |
| `image .+ not found` | docker_image_not_found | critical |
| `port is already allocated` | port_conflict | critical |

## GPU / CUDA / ML

| Pattern | Type | Severity |
|---------|------|----------|
| `CUDA out of memory` | cuda_oom | critical |
| `RuntimeError: CUDA` | cuda_runtime | critical |
| `cuDNN error` | cudnn_error | critical |
| `NCCL error` | nccl_error | critical |
| `torch\.cuda\.OutOfMemoryError` | torch_oom | critical |
| `Expected .+ but got .+` | tensor_shape | critical |
| `mat1 and mat2 shapes cannot be multiplied` | matrix_shape | critical |

## Git

| Pattern | Type | Severity |
|---------|------|----------|
| `CONFLICT \(content\)` | git_conflict | warning |
| `fatal: (.+)` | git_fatal | critical |
| `error: failed to push` | git_push_error | critical |
| `Your branch is behind` | git_behind | info |
| `detached HEAD` | git_detached | warning |
| `merge conflict` | git_merge_conflict | warning |

## Database

| Pattern | Type | Severity |
|---------|------|----------|
| `relation .+ does not exist` | pg_relation | critical |
| `ER_ACCESS_DENIED_ERROR` | mysql_access | critical |
| `SQLITE_ERROR` | sqlite_error | critical |
| `connection refused .+ port (5432\|3306\|27017)` | db_connection | critical |
| `deadlock detected` | db_deadlock | critical |

## Kubernetes

| Pattern | Type | Severity |
|---------|------|----------|
| `CrashLoopBackOff` | k8s_crashloop | critical |
| `ImagePullBackOff` | k8s_image_pull | critical |
| `OOMKilled` | k8s_oom | critical |
| `Error from server` | k8s_server_error | critical |
| `Insufficient (cpu\|memory)` | k8s_resources | critical |

## Adding Custom Patterns

To add patterns specific to your workflow, append to the `ERROR_PATTERNS` list in `screen_watcher.py`:

```python
# Example: add a custom pattern for your internal tool
ERROR_PATTERNS.append(
    (r"MyTool Error: (.+)", "my_tool_error", "critical")
)
```

Or add them in `config.yaml`:

```yaml
custom_patterns:
  - pattern: "MyTool Error: (.+)"
    name: my_tool_error
    severity: critical
```
