# TaintBox: Team Backlog & Sprint Assignments

**Sprint Cycle:** Sprint 1 — Subsystem Expansion & Data Pipelines  
**Sprint Deadline:** September 15, 2026 (1-Week Target)  
**Capstone Group:** (2) Nittany Street — Penn State DS 440  
**Repository Branch:** `harsh-dev` (Production Merge Target: `main`)  

---

## 1. Team Roster & Organizational Roles

| Member | Role | Core Domain & Ownership |
|---|---|---|
| **Harsh Rathi** | **Scrum Master** | Agile Sprint Cadence, Backlog Prioritization, Kanban Velocity, Blocker Removal, Demo Orchestration, Stakeholder Alignment |
| **Aryamaan** | **Team Lead & Architect** | System Architecture, Rust Module Boundaries, ACI Primitives, **Lead Author on Paper 1 (ACI Capability Scaling)** |
| **Ammar** | **Data & Testing Lead** | Sandbox Virtualization (gVisor/MicroVMs), Ingestion Pipelines (SWE-bench/Terminal-Bench), **CI/CD & Release Automation**, **Co-Author on Paper 1** |
| **Akshat** | **Attack & Security Lead** | Synthetic Adversarial Corpus Generator, Red Team Bunker Models, Query Intent Classifier, **Lead Author on Paper 2 (Taint Boundary Defense)** |
| **Saathvik** | **Dashboard & Backend Lead** | Axum REST API Daemon, Embedded Session Persistence (SQLite/redb), Web/Desktop Dashboard Frontend, **Co-Author on Papers 1 & 2** |

---

## 2. Explicit Sprint 1 Tasks & Deliverables (Due: Sept 16, 2026)

### 1. Harsh Rathi — Scrum Master & Process Operations
* **Primary Focus**: Sprint velocity, agile governance, and capstone deliverable readiness.
* **Explicit Deliverables**:
  * **Agile Governance & Kanban**: Maintain GitHub Projects Kanban board tracking items across *Backlog*, *In Progress*, *Review*, and *Done*.
  * **Sprint Cadence**: Run weekly standup syncs, document velocity, and identify dependencies between infrastructure (Ammar) and attack evaluation (Akshat).
  * **Demo Orchestration**: Coordinate live end-to-end demo runs of the `tbox` binary and desktop dashboard for faculty reviews.
  * **Course Milestone Alignment**: Ensure all DS 440 grading criteria, sponsor deliverables, and team contract expectations are met on schedule.

---

### 2. Aryamaan — Team Lead & Paper 1 (ACI Capability Scaling)
* **Primary Focus**: Overall architectural integrity and leading Paper 1 execution.
* **Component**: `src/aci/`, `src/runtime/`, `docs/`, and `papers/paper1_aci.md`.
* **Explicit Deliverables**:
  * **ACI Architectural Enforcement**:
    * Formalize structured tool execution pipeline (`view_lines`, `edit_block`, `search_files`, `grep`, `fetch`, `exec`).
    * Implement Tree-of-Thought state branching algorithms in `src/aci/state_tree.rs`.
    * Verify sandbox invariants and ensure zero context leaks across tool turns.
  * **Paper 1 Lead Author**:
    * **Title**: *Evaluating Tool Ergonomics and Context Compression in Autonomous AI Software Engineering*
    * **Hypothesis**: Structured windowed tools with snapshot/rewind improve task completion by `> 30%` while slashing token burn by `> 40%` compared to unstructured raw bash shells.
    * **Experimental Setup**: Define 4 benchmark conditions holding frontier models constant:
      1. *Condition A*: Vanilla raw bash shell without structured observation.
      2. *Condition B*: Structured typed tools (`view_lines`, `edit_block`).
      3. *Condition C*: Structured tools with state snapshot and rollback (`/rewind`).
      4. *Condition D*: Full TaintBox ACI with provenance surfaced to model context.
    * **Draft Deliverables**: Complete Introduction, Methodology, System Design, and Evaluation Plan for Paper 1.

---

### 3. Ammar — Data, Testing & Production CI/CD Automation (Paper 1 Co-Author)
* **Primary Focus**: Bulletproof testing infrastructure, GitHub Actions release pipeline, and sandbox virtualization.
* **Component**: `.github/workflows/`, `src/runtime/gvisor.rs`, `src/aci/dataset.rs`.
* **Explicit Deliverables**:
  * **Production CI/CD Automation (GitHub Actions)**:
    * **CI Matrix Workflow (`.github/workflows/ci.yml`)**:
      * Runs on every pull request to `harsh-dev` and `main`.
      * Cross-platform OS matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`.
      * Steps: `cargo test --all-targets`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
    * **Automated Release Workflow (`.github/workflows/release.yml`)**:
      * Trigger: Push/merge directly to `main` branch or tag `v*.*.*`.
      * Cross-compiles optimized production binaries:
        * `tbox-windows-x86_64.exe`
        * `tbox-linux-x86_64`
        * `tbox-macos-aarch64` (Apple Silicon)
      * Generates SHA256 checksums (`SHA256SUMS.txt`).
      * Automatically generates changelog from commit history.
      * Publishes an official GitHub Release with downloadable binary artifacts.
  * **Dataset Ingestion Pipeline**:
    * Python & Rust ingestor loading **SWE-bench Lite** (`princeton-nlp/SWE-bench_Lite`) and **Terminal-Bench** (`laude-institute/terminal-bench`).
    * Populates isolated test sandboxes with initial Git repos, task descriptions, and unit test suites.
  * **Sandbox Virtualization**:
    * Implement `gVisorRuntime` adapter using Google gVisor (`runsc` user-space kernel).
    * Prototype hardware MicroVM backends (Linux KVM / Firecracker).
  * **Paper 1 Co-Author**: Responsible for experimental test harness runs and container performance benchmarks.

---

### 4. Akshat — Attack Vectoring & Security Lead (Paper 2 Lead Author)
* **Primary Focus**: Adversarial attack generation, query classification, and leading Paper 2.
* **Component**: `src/bunker/`, `src/walls/classifier.rs`, `data/injections/`, `papers/paper2_defense.md`.
* **Explicit Deliverables**:
  * **Synthetic Adversarial Corpus Generator**:
    * CLI command `taintbox attackgen` connecting to local Ollama Bunker (`http://localhost:11434`) running `qwen2.5-coder` or `deepseek-r1`.
    * Generates benchmark test cases across the 4 threat families:
      1. *Direct Prompt Injection* (instruction overrides, jailbreaks, system prompt extraction).
      2. *Indirect Prompt Injection* (M365 email attacks, scraped HTML payloads, tool return poisoning).
      3. *Multi-Turn Conversational Injection* (role confusion, delayed payload triggers).
      4. *Tool Poisoning & Supply Chain* (tampered configs, dependency poisoning).
    * Formats outputs into standardized JSON in `data/injections/corpus_v1.json`.
  * **Two-Tier Runtime Query & Intent Classifier**:
    * *Tier 1*: Pure-Rust TF-IDF feature extraction + Random Forest classifier (`smartcore`) for `< 1ms` sub-turn triage.
    * *Tier 2*: ONNX Runtime transformer (DistilBERT / ModernBERT via `ort`) for deep semantic payload detection.
  * **Paper 2 Lead Author**:
    * **Title**: *Provable Containment of Indirect Prompt Injections via Bitmask Taint Ledgers and Sandbox Boundary Enforcement*
    * **Hypothesis**: Byte- and object-level taint tracking prevents credential exfiltration and unauthorized workspace modifications with `0%` false positives on benign developer workflows.
    * **Draft Deliverables**: Threat Model, Attack Taxonomy, Defense Formalism, and Empirical Evaluation against AgentDojo, InjecAgent, and M365 corpora.

---

### 5. Saathvik — Dashboard & Backend Lead (Papers 1 & 2 Co-Author)
* **Primary Focus**: REST API daemon, session persistence, and killer web/desktop UI.
* **Component**: `src/api/routes.rs`, `src/store/`, `apps/desktop/ui/`.
* **Explicit Deliverables**:
  * **Embedded Lightweight Session Database**:
    * In-process zero-daemon embedded persistence (`redb` / SQLite) in `src/store/session_db.rs`.
    * Guarantees zero context loss when hot-swapping models mid-session via Lossless Context Manager (LCM).
  * **Axum REST API Completion**:
    * Complete endpoints for `/v1/sandboxes`, `/v1/tools/execute`, `/v1/metrics`, `/v1/taint/graph`, and `/v1/lab/execute`.
    * Server-Sent Events (SSE) stream broadcasting live telemetry and policy trip alerts.
  * **Killer Web & Desktop Dashboard**:
    * Modern Cyber Dark Obsidian glassmorphic interface with live SVG telemetry sparklines.
    * Integrated Web IDE view tab (File Explorer, Code Editor, Terminal/Diff Drawer).
    * Interactive Attack Lab with live visual attack injection and policy interception cards.
  * **Papers 1 & 2 Co-Author**: Lead telemetry instrumentation, latency overhead quantification, and experimental visualization figures.

---

## 3. Getting Started for Teammates

1. **Clone & Switch to Dev Branch**:
   ```bash
   git clone https://github.com/xyzmr114/ds440-nittanystreet.git
   cd ds440-nittanystreet
   git checkout harsh-dev
   ```

2. **Verify Environment & All Tests**:
   ```bash
   cargo test
   ```
   *(All 59 unit and integration tests across 17 test suites must pass).*

3. **Run the Optimized Terminal App**:
   ```bash
   cargo run --release --bin tbox
   ```
