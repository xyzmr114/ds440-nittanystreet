# DS 440: TaintBox Runtime & ACI Engine — Technical Specification for AI Agents

**Version:** 3.1 (Pure Rust Architecture & OpenCode Parity)  
**Date:** Fall 2026  
**Capstone Group:** (2) Nittany Street — Penn State University  
**Course:** DS 440, Data Sciences Capstone  
**Instructor:** Dr. Robert Thomson (`rht5162@psu.edu`)  

---

## Team Roles & Governance

| Member | Role | Core Domain & Ownership |
|---|---|---|
| **Harsh Rathi** | **Scrum Master** | Sprint Cadence, Backlog Prioritization, Kanban Velocity, Blocker Removal, Demo Orchestration |
| **Aryamaan** | **Team Lead & Architect** | System Architecture, Rust Module Boundaries, ACI Primitives, **Lead Author on Paper 1 (ACI Capability Scaling)** |
| **Ammar** | **Data & Testing Lead** | Sandbox Virtualization (gVisor/MicroVMs), Ingestion Pipelines (SWE-bench/Terminal-Bench), **CI/CD & Release Automation**, **Co-Author on Paper 1** |
| **Akshat** | **Attack & Security Lead** | Synthetic Adversarial Corpus Generator, Red Team Bunker Models, Query Intent Classifier, **Lead Author on Paper 2 (Taint Boundary Defense)** |
| **Saathvik** | **Dashboard & Backend Lead** | Axum REST API Daemon, Embedded Session Persistence (SQLite/redb), Web/Desktop Dashboard Frontend, **Co-Author on Papers 1 & 2** |

---

> [!IMPORTANT]
> **Single Source of Truth**: This document is the absolute technical blueprint for every AI agent (Antigravity, Claude Code, Cursor, Copilot, Codex, etc.) collaborating on the TaintBox codebase. Read this document completely before writing or editing code.

---

## 1. Project Overview & Heilmeier Formulation

The **Agent-Computer Interface (ACI)** is the single highest capability lever in autonomous AI today. The same frontier or open-weight model on a raw shell performs at one level, but on a structured, typed execution harness performs at an entirely different level (evidenced by SWE-agent and ExploitBench). Concurrently, every agent deployment carries an unmitigated vulnerability: untrusted external content flows directly into the model's context window without provenance tracking, turning the agent's own tools into an adversary's execution vector.

**TaintBox** solves this by providing an instrumented, taint-tracked sandbox runtime and dual-mode interface (Axum REST daemon + interactive Ratatui terminal harness) featuring three core primitives:

1. **Snapshot & Rewind (State Branching)**: Time-travel state management allowing agents to explore Tree-of-Thought branches, commit execution nodes, and roll back on error safely.
2. **Taint-Tracked I/O**: Byte- and object-level provenance tracking where security policies trigger at tool boundaries (untrusted tainted data cannot trigger privileged shell execution, file deletion, or network exfiltration).
3. **Telemetry by Construction**: Every execution trace automatically generates structured audit streams, yielding empirical datasets for two research papers:
   * **Paper 1 (Capability)**: ACI capability curve comparing raw shell vs. structured tools vs. snapshot/rewind (Led by Aryamaan, with Ammar & Saathvik).
   * **Paper 2 (Defense)**: Taint-tracked boundary policies mitigating prompt injection and exfiltration (Led by Akshat, with Saathvik).

---

## 2. System Architecture & Tech Stack

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                           OPERATOR INTERFACES                               │
│   ┌───────────────────────────────────┐ ┌───────────────────────────────┐   │
│   │   Ratatui Terminal TUI            │ │   Tauri v2 / Web Dashboard    │   │
│   │   (Cyber Obsidian: tbox Zen)      │ │   (Cyber Dark Glassmorphic)   │   │
│   │   cargo run -- tui                │ │   apps/desktop/ui/            │   │
│   └─────────────────┬─────────────────┘ └───────────────┬───────────────┘   │
└─────────────────────┼───────────────────────────────────┼───────────────────┘
                      │                                   │
                      ▼                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          AXUM API DAEMON (Port 8000)                        │
│   - REST Endpoints (/v1/sandboxes, /v1/metrics, /v1/providers, /v1/schemas) │
│   - Telemetry Ring Buffer & Provenance Graph (/v1/taint/graph)              │
│   - Provider Manager (Spider Cloud, Ollama Bunker, Frontier LLMs)           │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
        ┌─────────────────────────────┼─────────────────────────────┐
        ▼                             ▼                             ▼
┌──────────────────┐        ┌──────────────────┐        ┌──────────────────┐
│   ACI HARNESS    │        │   TAINT ENGINE   │        │ CONTAINMENT WALLS│
│ (src/aci/)       │        │ (src/taint/)     │        │ (src/walls/)     │
│ - SWE-agent Tools│        │ - Provenance DAG │        │ - PromptInject   │
│ - State Tree DAG │◄──────►│ - Policy Profiles│◄──────►│ - Ouroboros      │
│ - AgentLoop      │        │ - Sensitive Paths│        │ - HalluScan      │
│ - ExploitBench   │        │ - Egress Filter  │        │ - Emergency Stop │
└────────┬─────────┘        └──────────────────┘        └──────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     SANDBOX RUNTIME & PERSISTENCE                           │
│   - LocalIsolatedRuntime (Tempdir / Workspace isolation with safety guard)  │
│   - Embedded Session Store & PostgreSQL 16 (session metadata, audit ledger) │
└─────────────────────────────────────────────────────────────────────────────┘
```

* **Core Runtime**: 100% Pure Rust (Edition 2021)
* **Web Gateway**: Axum 0.7 + Tower HTTP + Tokio
* **Database**: PostgreSQL 16 via `sqlx` + In-process Embedded Session Store
* **Terminal TUI**: `ratatui` 0.30 + `crossterm` 0.29 (tbox Zen engine)
* **Desktop GUI & Web Dashboard**: HTML5/CSS/JS Glassmorphic Web IDE (`apps/desktop/ui/`)
* **Scraper Integration**: Spider Cloud API provider
* **Local Attacker / Evaluation**: Ollama (`qwen2.5-coder`, `deepseek-r1`)

---

## 3. Production OpenCode Slash Commands (`tbox`)

All AI agents and operators can execute these production slash commands from the command palette (`Ctrl+P`) or bottom prompt dock:

| Command | Action & Architectural Effect |
|---|---|
| **`/init`** | Analyzes and indexes workspace, computes SHA-256 baselines, checks or generates `AGENTS.md`, and flags pre-existing injections. |
| **`/models`** | Queries `models.dev` dynamic catalog specifications (context windows, pricing, tool calling formats). |
| **`/models <id>`** | Hot-swaps the active inference model (e.g. `claude-3-7-sonnet-20250219`, `qwen2.5-coder:7b`) with **zero context loss**. |
| **`/diff`** | Computes character-exact dynamic line additions (`+`) and deletions (`-`) across sandbox files and surfaces taint provenance. |
| **`/attack [id]`** | Stages authentic adversarial scenarios from `data/injections/m365_indirect_attacks.json` against the boundary enforcer. |
| **`/walls`** | Inspects active containment walls (PromptInjectScanner, Ouroboros, E-Stop, Network Egress Gate). |
| **`/taint`** | Dumps the active bitmask provenance ledger and untrusted resource chains. |
| **`/rewind`** | Rolls back the sandbox filesystem safely to the last checkpoint snapshot. |
| **`/setup`** | Opens the interactive in-TUI configuration wizard to select provider, model, and API credentials. |
| **`/clear`** | Clears the in-memory display feed and re-initializes the session greeting. |

---

## 4. Mandatory Sandbox Security Invariants

All agents must strictly obey and preserve these security invariants:

1. **Path Canonicalization & Symlink Escape Guard**:
   All filesystem interactions must go through `resolve_path()`. Paths must be canonicalized and confirmed to start with the sandbox root directory. Relative escapes (`../`) and symlink escapes must return an immediate error.
2. **Safe Destructive Rewind**:
   `restore_snapshot()` must verify the presence of the `.taintbox_sandbox` marker before wiping or restoring any directory. It is forbidden to wipe non-sandbox directories.
3. **Shell Wrapper Mapping**:
   Any invocation of shell interpreters (`bash`, `sh`, `dash`, `zsh`, `cmd`, `powershell`) must be mapped to `exec_privileged` with `-c` content scanning.
4. **Untrusted Egress Quarantine**:
   External web downloads via `fetch()` are permanently labeled with `UntrustedWeb` (bitmask `0x01`). Tainted payloads are prohibited from making unallowlisted network egress calls.

---

## 5. CI/CD & Automated Release Rules

* **Branch Policy**: Active development occurs on `harsh-dev`. Production releases are cut by merging `harsh-dev` &rarr; `main`.
* **CI Matrix**: Every PR must pass `cargo test --all-targets` (59/59 tests passing), `cargo clippy`, and `cargo fmt`.
* **Automated Release**: Pushes to `main` trigger `.github/workflows/release.yml` to compile production release binaries (`tbox-windows-x86_64.exe`, `tbox-linux-x86_64`, `tbox-macos-aarch64`) with SHA256 checksums and automated release notes.
