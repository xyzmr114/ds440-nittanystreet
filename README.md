# TaintBox: A Taint-Tracked Sandbox Runtime and Agent-Computer Interface for AI Agents

**Penn State DS 440 Capstone — Fall 2026 | Group 2: Nittany Street**  
**Instructor:** Dr. Robert Thomson (`rht5162@psu.edu`)  
**Deliverables:** Business-grade technical report, working sandbox runtime, and two research papers.

---

## 1. Executive Summary

The agent-computer interface (ACI) is the single biggest capability lever in modern AI. The same frontier model executing on a raw shell performs at one level, while on a purpose-built execution harness (SWE-agent, ExploitBench) it performs at an entirely different tier. 

Simultaneously, agent deployments suffer from a critical vulnerability: **untrusted content** (emails, web scrapes, tool responses) flows into model context without provenance. Indirect prompt injection converts the agent's privileged tools into an adversary's tools.

**TaintBox** bridges this gap by delivering an instrumented sandbox execution runtime with three core primitives:

1. **Snapshot and Rewind**: Time-travel execution enabling agents to run, observe, branch, and roll back state across typed, structured tools without burning context on shell navigation.
2. **Taint-Tracked I/O**: Byte- and object-level provenance tracking. Untrusted data carries taint metadata throughout reads, transformations, and memory. Policy rules enforce boundary safety: tainted data cannot trigger privileged actions (network egress, shell exec, deletion).
3. **Telemetry by Construction**: Every execution stream automatically records provenance paths, side effects, and policy decisions—acting as an auditable compliance log and an empirical research dataset.

---

## 2. Research Program: Two Papers

TaintBox is designed not merely as software plumbing, but as a dual-purpose scientific instrument:

- **Paper 1 (Capability — The ACI Curve):** Measures model performance across execution harnesses holding the underlying model fixed:
  - Condition A: Raw shell without observation channel
  - Condition B: Structured typed tools
  - Condition C: Structured tools with snapshot-rewind
  - Condition D: Full ACI with provenance surfaced to the model
- **Paper 2 (Defense — Taint as the Trust Boundary):** Measures prompt injection mitigation against a diverse attack corpus (direct, indirect, multi-turn, tool poisoning):
  - Evaluates baseline vs. prompt hardening vs. TaintBox boundary policy vs. combined defense.
  - Tests whether boundary policy prevents exfiltration and action-on-objectives attacks that bypass prompt-only defenses.

---

## 3. Architecture

```text
Agent (LangChain, smolagents, Custom Harness, SDK)
        │
        ▼
[TaintBox API Gateway]   (REST / Python SDK / Session Store)
        │
        ▼
[ACI Layer]              (Typed tools: read, write, exec, observe, snapshot, rewind, fetch)
        │
        ▼
[Taint Engine]           (Provenance ledger, propagation rules, boundary policy enforcer)
        │
        ▼
[Sandbox Runtime]        (Isolated process sandbox / gVisor microVM adapter)
        │
        ▼
[Telemetry Sink]         (Structured JSONL event logs, attack traces, research metrics)
```

### Component Structure

- `src/aci/`: Structured tool definitions (`view_lines`, `search_files`, `grep`, `edit_block`, `fold_output`, `exec`, `fetch`), state tree branching, and autonomous `AgentLoop`.
- `src/taint/`: Provenance tracking ledger, propagation rules, policy profiles (`Standard`, `Strict`, `AuditOnly`), sensitive path protection, and network allowlisting.
- `src/walls/`: Overkill-inspired containment walls (`PromptInjectScanner`, `OuroborosWall`, `HalluScan`, `EmergencyStop`).
- `src/runtime/`: Sandbox execution isolation (`SandboxRuntime` trait and `LocalIsolatedRuntime`).
- `src/store/`: PostgreSQL persistence with `sqlx` schema migrations and `SessionManager`.
- `src/metrics/`: Telemetry collector and real-time metrics stream.
- `src/config/`: Provider management for Spider Cloud scrapers, Ollama bunker, and frontier LLMs.
- `src/tui/`: Interactive Ratatui Cyber Dark terminal dashboard (`taintbox tui`).
- `apps/desktop/`: Tauri v2 desktop application shell (`apps/desktop/ui/`).

---

## 4. Team Structure & Responsibilities

| Role | Owner | Focus Area |
|---|---|---|
| **Product Owner / Lead Author** | Harsh | System Architecture, ACI Design, Paper 1 Lead, Sponsor Alignment |
| **Scrum Master / Process Lead** | Aryamaan | Sprint Planning, Kanban, Progress Reports |
| **Data and Infrastructure Lead** | Ammar | Sandbox Virtualization (gVisor/MicroVMs), Taint Engine, Eval Harness |
| **Attack and Security Lead** | Akshat | Synthetic Adversarial Corpus Generator, Red Team Models, Paper 2 Lead |
| **Evaluation and Reliability** | Saathvik | SWE-bench/Terminal-Bench Ingestor, Metrics, Reproducibility |

---

## 5. Quick Start

### Prerequisites
* **Rust** (MSRV: 1.78+)
* **Cargo**
* *(Optional)* **PostgreSQL 16** & **Ollama** (for local attacker models)

### Clone & Build
```bash
git clone https://github.com/xyzmr114/ds440-nittanystreet.git
cd ds440-nittanystreet
git checkout harsh-dev
cargo build
```

### Running Tests
```bash
cargo test
```
*(All 48 unit & integration tests across 15 suites compile and pass with 0 warnings).*

### Running the Subsystems

```bash
# 1. Start the Axum REST API Daemon (Port 8000)
cargo run -- daemon --port 8000

# 2. Launch the Ratatui Cyber Dark Terminal UI
cargo run -- tui

# 3. Run an autonomous agent task in an isolated sandbox harness
cargo run -- run "Audit the repository and summarize files" --max-steps 10

# 4. Run the ExploitBench & Defense Benchmark Evaluation Suite
cargo run -- eval

# 5. Run Environment & Subsystem Health Diagnostics
cargo run -- doctor
```

---

## 6. Documentation & Specifications

- [`PROPOSAL.md`](PROPOSAL.md) — Official DS 440 capstone proposal document.
- [`AGENTS.md`](AGENTS.md) — Technical specification and ground rules for AI agents and developers.
- [`TODO.md`](TODO.md) — Sprint backlog, teammate task assignments, and deadlines.

---

## 7. License

Public domain (Unlicense). See [LICENSE](LICENSE).
