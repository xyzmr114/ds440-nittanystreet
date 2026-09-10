# DS 440: TaintBox Runtime & ACI Engine — Technical Specification for AI Agents

**Version:** 3.0 (Pure Rust Architecture)  
**Date:** Fall 2026  
**Capstone Group:** (2) Nittany Street — Penn State University  
**Course:** DS 440, Data Sciences Capstone  
**Instructor:** Dr. Robert Thomson (`rht5162@psu.edu`)  
**Product Owner:** Harsh Rathi  

---

> [!IMPORTANT]
> **Single Source of Truth**: This document is the absolute technical blueprint for every AI agent (Antigravity, Claude Code, Cursor, Copilot, Codex, etc.) collaborating on the TaintBox codebase. Read this document completely before writing or editing code.

---

## 1. Project Overview & Heilmeier Formulation

The **Agent-Computer Interface (ACI)** is the single highest capability lever in autonomous AI today. The same frontier or open-weight model on a raw shell performs at one level, but on a structured, typed execution harness performs at an entirely different level (evidenced by SWE-agent and ExploitBench). Concurrently, every agent deployment carries an unmitigated vulnerability: untrusted external content flows directly into the model's context window without provenance tracking, turning the agent's own tools into an adversary's execution vector.

**TaintBox** solves this by providing an instrumented, taint-tracked sandbox runtime and dual-mode interface (Axum REST daemon + interactive Ratatui terminal harness) featuring three core primitives:

1. **Snapshot & Rewind (State Branching)**: Time-travel state management allowing agents to explore Tree-of-Thought branches, commit execution nodes, and roll back on error.
2. **Taint-Tracked I/O**: Byte- and object-level provenance tracking where security policies trigger at tool boundaries (untrusted tainted data cannot trigger privileged shell execution, file deletion, or network exfiltration).
3. **Telemetry by Construction**: Every execution trace automatically generates structured audit streams, yielding empirical datasets for two research papers:
   * **Paper 1 (Capability)**: ACI capability curve comparing raw shell vs. structured tools vs. snapshot/rewind.
   * **Paper 2 (Defense)**: Taint-tracked boundary policies mitigating prompt injection and exfiltration.

---

## 2. System Architecture & Tech Stack

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                           OPERATOR INTERFACES                               │
│   ┌───────────────────────────────────┐ ┌───────────────────────────────┐   │
│   │   Ratatui Terminal TUI            │ │   Tauri v2 Desktop App        │   │
│   │   (Cyber Dark: #0d1117 / Cyan)    │ │   (HTML5/CSS/JS Sidecar Shell)│   │
│   │   taintbox tui                    │ │   apps/desktop/               │   │
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
│   - LocalIsolatedRuntime (Tempdir / Workspace isolation)                   │
│   - PostgreSQL 16 Store (sqlx schema, session metadata, audit ledger)       │
└─────────────────────────────────────────────────────────────────────────────┘
```

* **Core Runtime**: 100% Pure Rust (Edition 2021)
* **Web Gateway**: Axum 0.7 + Tower HTTP + Tokio
* **Database**: PostgreSQL 16 via `sqlx` (asynchronous, pooled, migrations included)
* **Terminal TUI**: `ratatui` 0.30 + `crossterm` 0.29
* **Desktop GUI**: Tauri v2 shell (`apps/desktop/`)
* **Scraper Integration**: Spider Cloud API provider
* **Local Attacker / Evaluation**: Ollama (`qwen2.5-coder`, `deepseek-r1`)

---

## 3. Repository Directory Structure

```text
ds440-nittanystreet/
├── Cargo.toml                  # Project manifest, dependencies, features
├── Cargo.lock                  # Pinned dependency lockfile
├── README.md                   # Public documentation & getting started
├── PROPOSAL.md                 # Formal Penn State Capstone Proposal
├── AGENTS.md                   # This document (AI Agent Technical Spec)
├── TODO.md                     # Team task backlog & sprint assignments
├── LICENSE                     # Unlicense
├── .gitignore                  # Git ignore rules
├── .env.example                # Example environment configuration
├── apps/
│   └── desktop/                # Tauri v2 Desktop App
│       ├── src-tauri/          # Tauri Rust configuration & Cargo.toml
│       └── ui/                 # HTML5/CSS/JS frontend matching TUI palette
├── migrations/                 # PostgreSQL relational database migrations
│   └── 20260909000001_initial_schema.sql
├── src/
│   ├── lib.rs                  # Library entrypoint exporting core modules
│   ├── main.rs                 # Binary entrypoint
│   ├── models.rs               # Core shared data types & serialization
│   ├── aci/                    # Agent-Computer Interface Subsystem
│   │   ├── mod.rs
│   │   ├── harness.rs          # ACIHarness with typed SWE-agent tool suite
│   │   ├── state_tree.rs       # Branching State Tree (Tree-of-Thought DAG)
│   │   ├── schemas.rs          # Universal OpenAI / Anthropic JSON schemas
│   │   ├── agent_loop.rs       # Autonomous AgentLoop & XML/JSON parser
│   │   └── benchmark.rs        # ExploitBench & Capability evaluation runner
│   ├── taint/                  # Taint Ledger & Policy Engine
│   │   ├── mod.rs
│   │   ├── engine.rs           # Provenance propagation & ledger tracking
│   │   └── policy.rs           # Policy profiles, sensitive paths, allowlists
│   ├── walls/                  # Overkill Containment Walls
│   │   ├── mod.rs
│   │   ├── promptinject.rs     # 5-family injection scanner (Low/Med/High)
│   │   ├── ouroboros.rs        # Self-modification shield (tests & policies)
│   │   ├── halluscan.rs        # Path validator neutralizing hallucinated loops
│   │   └── estop.rs            # Circuit breaker for critical events
│   ├── runtime/                # Sandbox Execution Isolation
│   │   ├── mod.rs
│   │   └── base.rs             # SandboxRuntime trait & LocalIsolatedRuntime
│   ├── store/                  # Persistence Layer
│   │   ├── mod.rs
│   │   ├── postgres.rs         # PostgresStore with sqlx queries
│   │   └── session.rs          # In-memory & DB SessionManager
│   ├── metrics/                # Telemetry & Shared Metrics Stream
│   │   └── mod.rs              # Thread-safe MetricsCollector & ring buffer
│   ├── config/                 # Configuration & Provider Management
│   │   ├── mod.rs
│   │   └── providers.rs        # Spider Cloud, Ollama, Frontier LLM config
│   ├── api/                    # Axum REST API Gateway
│   │   ├── mod.rs
│   │   └── routes.rs           # Route definitions & JSON/SSE handlers
│   ├── cli/                    # Command-Line Interface
│   │   └── mod.rs              # Clap CLI (daemon, run, eval, doctor, tui)
│   └── tui/                    # Ratatui Cyber Dark Terminal UI
│       ├── mod.rs
│       └── app.rs              # 5-view reactive TUI with keyboard controls
└── tests/                      # Integration Test Suite (48 tests, 100% pass)
    ├── test_rust_aci_advanced.rs
    ├── test_rust_aci_tools.rs
    ├── test_rust_aci_tools_extended.rs
    ├── test_rust_advanced_taint.rs
    ├── test_rust_agent_loop.rs
    ├── test_rust_api_server.rs
    ├── test_rust_benchmark.rs
    ├── test_rust_metrics.rs
    ├── test_rust_postgres_store.rs
    ├── test_rust_providers.rs
    ├── test_rust_schemas.rs
    ├── test_rust_state_tree.rs
    ├── test_rust_taint_engine.rs
    ├── test_rust_tui.rs
    └── test_rust_walls.rs
```

---

## 4. Agent Operating Rules & Commitments

Every AI agent modifying or reading this repository MUST abide by these 8 immutable rules:

1. **Git Branching Protocol**:
   * All development commits MUST be committed to **`harsh-dev`**.
   * **NEVER execute `git push` autonomously.** Pushing to remote is strictly controlled by the user due to local authentication/GCM constraints. Wait for the user's explicit command.
   * Git commit author MUST always be set to: `Harsh Rathi <94193726+xyzmr114@users.noreply.github.com>`.
2. **Strict Test-Driven Development (TDD)**:
   * Every new feature, endpoint, or policy rule must be accompanied by integration tests under `tests/`.
   * The entire test suite must compile and pass cleanly (`cargo test` must exit with `0 passed; 0 failed; 0 warnings`).
3. **No Unchecked `unwrap()` in Production Code**:
   * Use `anyhow::Result<T>` or `thiserror` for error handling in production code. Tool failures must return structured `ToolResult` errors rather than crashing the process.
4. **Internal Documentation Outside Git**:
   * Internal plans and walkthroughs must be maintained in `c:\Users\harsh\Desktop\fall 26\ds 440\dev\` (outside git). Do NOT commit scratch notes or internal implementation plans into git.
5. **Policy at the Boundary**:
   * Taint checking is programmatic. Never trust an LLM to self-report taint or verify safety. The harness intercepts tool invocations before they reach the OS or network.
6. **Zero Fabrication**:
   * Never hardcode benchmark evaluation numbers or paper results. All metrics must be computed dynamically by `BenchmarkRunner` or `MetricsCollector`.
7. **Unified Aesthetic**:
   * Any visual UI element (Ratatui TUI, Tauri desktop app, or web console) must use the standard Cyber Dark color palette:
     * Background: `#0d1117`
     * System / Accent: `#58a6ff` (Neon Cyan)
     * Trusted / Success: `#3fb950` (Emerald Green)
     * Warning / Suspicious: `#f0883e` (Amber Orange)
     * Policy Block / Taint: `#f85149` (Neon Crimson)
     * Dim text: `#8b949e`
8. **No Hardcoded Secrets**:
   * Never commit actual API keys. Always use `.env`, environment variables, or the dynamic `/v1/providers` runtime endpoint.

---

## 5. Subsystem Implementation Guide

### A. SWE-agent Tool Suite (`src/aci/harness.rs`)
Agents interact through structured typed tools rather than a raw terminal:
* `view_lines(path, start, end)`: 1-indexed viewing with line numbers.
* `search_files(pattern)`: Globbing without shell injection risks.
* `grep(query)`: Regex/string matching across workspace files.
* `edit_block(path, search, replace)`: Precise string substitution with taint inheritance.
* `fold_output(text, head, tail)`: Windowing that truncates large tool outputs, preserving up to 40% model context tokens.
* `exec(program, args)`: Executes commands through runtime boundary, enforcing network allowlists and blocking tainted inputs.
* `fetch(url, save_as, mock)`: Retrieves remote resources and tags them `ProvenanceTag::UntrustedWeb` with `TrustLevel::Untrusted`.
* `snapshot(desc)` & `rewind(id)`: Captures filesystem state and taint ledger for instant backtracking.

### B. Containment Walls (`src/walls/`)
Directly ported from Overkill design principles:
* **PromptInjectScanner**: Regex and keyword entropy classifier detecting `instruction_override`, `role_confusion`, `capability_jailbreak`, `exfiltration`, and `tool_misuse`.
* **OuroborosWall**: Path inspection blocking attempts to modify `tests/**`, `src/taint/**`, `src/walls/**`, or `.git/**`.
* **HalluScan**: Verifies path existence before tool calls, preventing hallucinated path retry loops.
* **EmergencyStop**: Circuit breaker that halts execution upon critical security events.

### C. Taint Ledger & Policy Engine (`src/taint/`)
* **`ProvenanceRecord`**: Contains `source_id`, `tag`, `trust_level` (`Untrusted`, `Internal`, `Trusted`), and `chain_of_custody`.
* **`PolicyProfile`**:
  * `Standard`: Blocks privileged shell execution and egress using tainted data.
  * `Strict`: Blocks any file write derived from untrusted inputs.
  * `AuditOnly`: Records violations in telemetry without interrupting the agent turn (for research ablations).
* **Sensitive Path Boundaries**: Blocks access to `.env`, `id_rsa`, `~/.ssh/`, `/etc/passwd`.
* **Network Allowlist**: Egress programs (`curl`, `wget`) are restricted to verified domains (e.g. `api.github.com`).
* **Declassification**: Tainted files can be cleared to `TrustLevel::Trusted` using verifiable tokens.

---

## 6. How to Build, Test, and Run

### Run Full Test Suite
```powershell
cargo test
```

### Start API Daemon
```powershell
cargo run -- daemon --port 8000
```

### Launch Interactive Ratatui Terminal TUI
```powershell
cargo run -- tui
```

### Run Autonomous Agent Task in Harness
```powershell
cargo run -- run "Fix the bug in src/main.rs" --max-steps 15
```

### Run Benchmark Suite
```powershell
cargo run -- eval
```

### Run Subsystem Diagnostics
```powershell
cargo run -- doctor
```
