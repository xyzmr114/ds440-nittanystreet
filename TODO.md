# TaintBox: Team Backlog & Sprint Assignments

**Sprint Cycle:** Sprint 1 — Subsystem Expansion & Data Pipelines  
**Sprint Deadline:** September 16, 2026 (1-Week Target)  
**Capstone Group:** (2) Nittany Street — Penn State DS 440  
**Repository Branch:** `harsh-dev`  

---

## Team Roster & Roles

| Name | Role | Primary Domain |
|---|---|---|
| **Harsh Rathi** | Product Owner & Lead Author | System Architecture, ACI Design, Paper 1 Lead, Sponsor Alignment |
| **Aryamaan** | Scrum Master & Process Lead | Sprint Cadence, Kanban, CI/CD, Progress Reports |
| **Ammar** | Data & Infrastructure Lead | Sandbox Virtualization (gVisor/MicroVMs), Taint Engine, Eval Pipeline |
| **Akshat** | Attack & Security Lead | Adversarial Corpus Generator, Red Team Models, Paper 2 Lead |
| **Saathvik** | Evaluation & Reliability Lead | SWE-bench/Terminal-Bench Ingestor, Reproducibility, Telemetry |

---

## Sprint 1 Task Assignments (Due: Sept 16, 2026)

### 1. Akshat — Synthetic Attack Corpus & Query Classifier (Paper 2)
* **Goal**: Build the autonomous adversarial injection dataset generator and train a lightweight runtime query classifier.
* **Component**: `src/bunker/` and `src/walls/classifier.rs`.
* **Details**:
  * Connect to local Ollama Bunker instance (`http://localhost:11434`) running `qwen2.5-coder` or `deepseek-r1`.
  * Implement prompts generating test cases across the 4 attack families (Direct, Indirect, Multi-Turn, Tool Poisoning).
  * **Query / Intent Classification**:
    * Design Tier 1 classifier: TF-IDF feature extraction + Random Forest (sub-millisecond CPU inference via pure Rust / `smartcore`).
    * Benchmark against Tier 2 ONNX transformer (DistilBERT / ModernBERT via `ort`).
    * Classify incoming tool inputs and user prompts for injection intent before execution.
  * Format output into standardized JSON benchmark scenarios in `data/injections/corpus_v1.json`.
* **Deliverable**: Working `taintbox attackgen` command with 100+ generated attack cases and trained Random Forest model artifact.

---

### 2. Saathvik & Ammar — SWE-bench & Terminal-Bench Dataset Ingestor (Paper 1)
* **Goal**: Ingest real-world agent capability benchmarks for the Paper 1 ACI curve study.
* **Component**: `src/aci/dataset.rs` and `src/aci/benchmark.rs`.
* **Details**:
  * Build an ingestion pipeline for:
    * **SWE-bench Lite** (`princeton-nlp/SWE-bench_Lite`): Python repository issue resolution tasks.
    * **Terminal-Bench** (`laude-institute/terminal-bench`): CLI and terminal execution scenarios.
  * Populate isolated workspace sandboxes with the initial repo state, problem statements, and test suites.
  * Connect with the `AgentLoop` to run comparative evaluations:
    * Trial A: Vanilla raw bash shell.
    * Trial B: TaintBox structured typed tool harness (`view_lines`, `edit_block`, `search_files`).
    * Trial C: TaintBox ACI with Tree-of-Thought snapshot and rewind.
* **Deliverable**: Automated runner loading SWE-bench instances and generating CSV capability curves.

---

### 3. Ammar — Sandbox Virtualization Tiers & Cross-Platform MicroVMs
* **Goal**: Expand execution isolation beyond the portable `LocalIsolatedRuntime`.
* **Component**: `src/runtime/gvisor.rs`, `src/runtime/microvm.rs`, and `src/runtime/mod.rs`.
* **Details**:
  * **Two-Tier Isolation Strategy**:
    * **Tier 1 (Host/Daemon)**: `taintboxd` and session persistence.
    * **Tier 2 (Agent Sandbox)**: The workspace where the agent executes arbitrary code.
  * Implement `gVisorRuntime` using Google gVisor (`runsc` user-space kernel).
  * **Cross-Platform MicroVM Exploration**:
    * Linux: KVM (`/dev/kvm`) + Firecracker / Cloud-Hypervisor.
    * macOS: Apple Silicon `Virtualization.framework` (`libkrun` / `vfkit`).
    * Windows: WSL2 nested KVM (`.wslconfig`) and Hyper-V Windows Sandbox (`runhcs`).
* **Deliverable**: Working `gVisorRuntime` in `src/runtime/gvisor.rs` and architectural prototype for hardware microVM backends.

---

### 4. Aryamaan — Embedded Session DB, CI/CD & Progress Report 1
* **Goal**: Implement lightweight session persistence and establish continuous integration.
* **Component**: `src/store/session_db.rs`, `.github/workflows/ci.yml`, and course documentation.
* **Details**:
  * **Lightweight Embedded Session Database**:
    * Pure-Rust in-process storage (`redb` or embedded SQLite) so context is never lost when switching models mid-turn.
    * Dual-tier Hot/Cold Memory: Hot LCM (Lossless Context Manager) + Cold 2-Phase DAG state machine.
  * Set up GitHub Actions CI workflow executing `cargo test --all-targets` and `cargo clippy`.
  * Compile draft for **Progress Report 1** (Literature review, system architecture, and completed Rust foundation).
* **Deliverable**: Embedded session DB module and automated green CI build on `harsh-dev`.

---

### 5. Harsh Rathi — ACI Engine, models.dev Catalog & Lead Author
* **Goal**: Finalize OpenCode feature parity, dynamic catalog integration, and lead Paper 1.
* **Component**: `src/aci/harness.rs`, `src/config/models_dev.rs`, `src/tui/zen.rs`.
* **Status / Achievements**:
  * [x] **Path Traversal Escape Patched**: Canonicalization + root prefix enforcement.
  * [x] **Shell Wrapper Interception**: `bash`/`sh -c` classified as privileged with sub-command scanning.
  * [x] **Safe Destructive Rewind**: Protected against wiping non-sandbox directories.
  * [x] **Native Tool Calls Parsing**: OpenAI function calling parsed before content fallback.
  * [x] **PromptInjectScanner Active**: Wired directly into tool argument and output execution pipeline.
  * [x] **Real `fetch()` Tool**: Active HTTP GET with domain allowlist egress gate & `UntrustedWeb` provenance.
  * [x] **Workspace Repository `/init`**: Full directory indexing, `AGENTS.md` verification, baseline hashing.
  * [x] **Dynamic `models.dev` Catalog**: Mid-session model switching via `/models` without context loss.
  * [x] **Authentic Attack Corpus**: Dynamic loader for `m365_indirect_attacks.json` in Zen TUI.
* **Deliverable**: OpenCode-grade production terminal application with 100% test coverage.

---

## Getting Started for Teammates

1. **Clone & Switch to Dev Branch**:
   ```bash
   git clone https://github.com/xyzmr114/ds440-nittanystreet.git
   cd ds440-nittanystreet
   git checkout harsh-dev
   ```

2. **Verify Environment**:
   ```bash
   cargo test
   ```
   *(All 48 tests across 15 suites must pass).*

3. **Launch the TUI**:
   ```bash
   cargo run -- tui
   ```
