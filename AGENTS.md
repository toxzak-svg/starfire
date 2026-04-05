# AGENTS.md — Agent Modes & Capabilities

This file documents the agent modes available for working with Star.

> **NOTE**: AGENTS.md defines how the agent framework interacts with this codebase. See [Code References](#code-references) below.

---

## Available Modes

### Code Mode (💻)
**Slug**: `code`  
**Purpose**: Write, modify, refactor code. Implement features, fix bugs, create files.

```json
{
  "mode": "code",
  "description": "Write, modify, or refactor code",
  "restrictions": null
}
```

**Best for**:
- Implementing new features in `lib/` or `src/`
- Fixing bugs in Rust code
- Adding unit tests with `#[cfg(test)]`
- Creating new modules

**Files editable**:
- All `.rs` files
- `Cargo.toml`

---

### Architect Mode (🏗️)
**Slug**: `architect`  
**Purpose**: Plan, design, strategize before implementation.

```json
{
  "mode": "architect",
  "description": "Plan and design system architecture",
  "restrictions": ["*.md only"]
}
```

**Best for**:
- Creating technical specifications
- Designing new system components
- Planning migrations
- Breaking down complex features

**Files editable**:
- `*.md` files only

---

### Debug Mode (🪲)
**Slug**: `debug`  
**Purpose**: Troubleshooting issues, investigating errors, diagnosing problems.

```json
{
  "mode": "debug",
  "description": "Debug and troubleshoot issues"
}
```

**Best for**:
- Adding logging to trace errors
- Analyzing stack traces
- Identifying root causes
- Investigating test failures

**Tools enabled**:
- `execute_command` with debugging flags
- `search_files` for error patterns
- `read_file` with line numbers

---

### Ask Mode (❓)
**Slug**: `ask`  
**Purpose**: Explanations, documentation, answers to technical questions.

```json
{
  "mode": "ask",
  "description": "Explain and answer questions"
}
```

**Best for**:
- Understanding concepts
- Analyzing existing code
- Getting recommendations
- Learning about technologies

**Files editable**: None (read-only)

---

### Orchestrator Mode (🪃)
**Slug**: `orchestrator`  
**Purpose**: Coordinate complex multi-step projects across domains.

```json
{
  "mode": "orchestrator",
  "description": "Coordinate multi-domain tasks"
}
```

**Best for**:
- Breaking down large tasks
- Managing workflows
- Coordinating across expertise areas

---

### Documentation Writer Mode (✍️)
**Slug**: `documentation-writer`  
**Purpose**: Create, update, improve technical documentation.

```json
{
  "mode": "documentation-writer",
  "description": "Write and update documentation"
}
```

**Best for**:
- README files
- API documentation
- User guides
- Installation instructions

---

### DevOps Mode (🚀)
**Slug**: `devops`  
**Purpose**: Deploy applications, manage infrastructure, CI/CD.

```json
{
  "mode": "devops",
  "description": "Deploy and manage infrastructure"
}
```

**Best for**:
- Cloud resource provisioning
- Configuring deployments
- Managing environments
- Setting up monitoring

---

### Project Research Mode (🔍)
**Slug**: `project-research`  
**Purpose**: Investigate codebase, analyze architecture, gather context.

```json
{
  "mode": "project-research",
  "description": "Research and analyze codebase"
}
```

**Best for**:
- Onboarding to new projects
- Understanding complex codebases
- Analyzing feature implementations

---

## Code References

Agent tooling is defined in the system prompt and connects to:

| Tool | Purpose |
|------|---------|
| `execute_command` | Run `cargo`, `rustc`, CLI tools |
| `search_files` | Regex search across `.rs` files |
| `read_file` | Read source with line numbers |
| `write_to_file` | Create new files |
| `search_and_replace` | Edit existing files |
| `list_files` | Explore directory structure |

---

## Workspace Context

### Project Structure
```
starfire/
├── lib/              # Core intelligence library (84 .rs files, ~26K LOC)
├── src/              # Binary entry point
├── data/             # Runtime data (IDENTITY.md, star.db)
├── scripts/          # Python helpers
├── ui/              # Next.js web interface
├── docs/            # API & architecture docs
└── plans/           # Feature roadmaps
```

### Key Commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Build release binary |
| `cargo test` | Run 241 unit tests |
| `cargo run --bin star` | Start Star API server |
| `cargo run --example *` | Run examples |

---

## Mode Selection Guide

Use this decision tree to pick the right mode:

```
Is this about writing, modifying, or creating code?
├── YES → Is it a complex multi-domain task?
│     ├── YES → orchestrator
│     ├── NO → Is it debugging/troubleshooting?
│         ├── YES → debug  
│         ├── NO → code
├── NO → Is it planning/designing?
    ├── YES → architect
    ├── NO → Is it explaining/answering?
        ├── YES → ask
        ├── NO → Is it documentation?
            ├── YES → documentation-writer
            ├── NO → Is it research/analysis?
                ├── YES → project-research
                ├── NO → devops (if deployment)
```

---

## Notes

- Modes restrict which files can be edited
- Mode switches require user approval
- Complex tasks may use multiple modes sequentially
- The "code" mode is most commonly used for Star development