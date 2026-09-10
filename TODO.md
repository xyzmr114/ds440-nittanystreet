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
| **Aryamaan Dhuwalia** | Scrum Master & Process Lead | Sprint Cadence, Kanban, CI/CD, Progress Reports |
| **Ammar Al-Sabti** | Data & Infrastructure Lead | Sandbox Virtualization (gVisor/MicroVMs), Taint Engine, Eval Pipeline |
| **Akshat Singhal** | Attack & Security Lead | Adversarial Corpus Generator, Red Team Models, Paper 2 Lead |
| **Saathvik Sharma** | Evaluation & Reliability Lead | SWE-bench/Terminal-Bench Ingestor, Reproducibility, Telemetry |

---

## Sprint 1 Task Assignments (Due: Sept 16, 2026)

### 1. Akshat Singhal — Synthetic Attack Corpus Generator (Paper 2)
* **Goal**: Build the autonomous adversarial injection dataset generator pipeline in Rust/Ollama.
* **Component**: `src/bunker/` and CLI subcommand `taintbox attackgen`.
* **Details**:
  * Connect to the local Ollama Bunker instance (`http://localhost:11434`) running `qwen2.5-coder` or `deepseek-r1`.
  * Implement prompts generating test cases across the 4 attack families identified in the proposal:
    1. **Direct Injections**: System instruction overrides, jailbreaks, and delimiter escapes.
    2. **Indirect Injections**: Injections disguised inside web pages, Markdown documentation, and simulated third-party API payloads.
    3. **Multi-Turn Shifts**: Progressive goal drift attacks spanning 3–5 turns.
    4. **Tool Poisoning**: Injected instructions embedded in tool outputs (e.g. `git log` or `grep` results containing malicious `<tool_call>` tags).
  * Format output into standardized JSON benchmark scenarios in `data/injections/corpus_v1.json`.
* **Deliverable**: Working `taintbox attackgen --family indirect --count 50` command with 100+ generated attack cases.

---

### 2. Saathvik Sharma & Ammar Al-Sabti — SWE-bench & Terminal-Bench Dataset Ingestor (Paper 1)
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

### 3. Ammar Al-Sabti — Sandbox Virtualization Tiers & MicroVM Stretch Goal
* **Goal**: Expand execution isolation beyond the portable `LocalIsolatedRuntime`.
* **Component**: `src/runtime/gvisor.rs` and `src/runtime/mod.rs`.
* **Architecture Clarification**:
  * *Why raw Docker/Podman is insufficient for untrusted code*: Standard containers share the host Linux kernel. In adversarial settings, container escape vulnerabilities via `/proc`, `/sys`, or kernel flaws are unacceptable.
  * *Two-Tier Isolation Strategy*:
    * **Tier 1 (Host/Daemon)**: `taintboxd` and PostgreSQL can run in standard containers or host OS.
    * **Tier 2 (Agent Sandbox)**: The workspace where the agent runs arbitrary code requires virtualization.
  * Implement `gVisorRuntime` using Google gVisor (`runsc` user-space kernel) implementing the `SandboxRuntime` trait.
  * *(Stretch)*: Explore lightweight MicroVM drivers (Firecracker or Cloud-Hypervisor) for complete hardware virtualization.
* **Deliverable**: Working `gVisorRuntime` implementation in `src/runtime/gvisor.rs` with integration tests.

---

### 4. Aryamaan Dhuwalia — GitHub Actions CI/CD & Progress Report 1
* **Goal**: Establish continuous integration and formal course reporting.
* **Component**: `.github/workflows/ci.yml` and course documentation.
* **Details**:
  * Set up GitHub Actions workflow executing:
    * `cargo test --all-targets` (verifying all 48+ tests pass).
    * `cargo clippy -- -D warnings` (linting).
    * `cargo fmt --check` (formatting).
  * Coordinate the sprint Kanban board and compile draft for **Progress Report 1** (Literature review, system architecture, and completed Rust foundation).
* **Deliverable**: Automated green CI build on `harsh-dev` and Progress Report 1 draft.

---

### 5. Harsh Rathi — Spider Cloud Scraper Integration & Lead Author
* **Goal**: Wire Spider Cloud's scraping engine into the live agent tool suite and lead Paper 1.
* **Component**: `src/aci/scraper.rs` and `ProviderManager`.
* **Details**:
  * Connect `spider_scrape(url, depth)` in `ACIHarness` to the Spider Cloud API.
  * Automatically register scraped pages into the Taint Ledger with `TrustLevel::Untrusted`.
  * Pass scraped markdown through `PromptInjectScanner` to identify indirect prompt injections embedded in web pages.
  * Author Paper 1 methodology and ACI capability hypothesis.
* **Deliverable**: `spider_scrape` tool integrated into harness with automated taint labeling.

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
