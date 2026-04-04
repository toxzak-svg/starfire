# Starfire Deployment Guide

> Simple build and run instructions for Starfire + QuaNot.

---

## Quick Start

```bash
# Build everything
./build.sh all

# Run Starfire
./target/release/star.exe

# Or run QuaNot
cd quanot && .venv/Scripts/python src/main.py
```

---

## Prerequisites

| Component | Version |
|-----------|---------|
| Rust | 1.70+ |
| Python | 3.10+ |

---

## Manual Build

### Starfire
```bash
cargo build --release
./target/release/star.exe
```

### QuaNot
```bash
cd quanot
python -m venv .venv
.venv/Scripts\pip install -r ..\requirements.txt
.venv\Scripts\python src\main.py
```

---

## Commands

Starfire accepts these commands:

- `star.exe chat` - Interactive chat
- `star.exe status` - Check memory
- `star.exe api` - Start HTTP API

---

## Troubleshooting

**Module not found:**
```bash
cd quanot && .venv\Scripts\pip install -r ..\requirements.txt
```

**Build error:**
```bash
cargo clean && cargo build --release
```

---

*Last updated: 2026-04-04*
