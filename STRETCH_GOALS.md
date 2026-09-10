# TaintBox: Production Roadmap & Stretch Goals

**Document Status:** Architectural Vision & Production Strategy  
**Authors:** Group 2 (Nittany Street) — Penn State DS 440  
**Target:** Post-Capstone Enterprise Sandbox & Commercial Deployment  

---

## 1. Executive Strategy

While TaintBox's current baseline (`LocalIsolatedRuntime` in pure Rust) delivers portable, zero-overhead execution across Windows, macOS, and Linux for academic research, commercial deployments (e.g. Modal, E2B, AWS Lambda) demand **hardware-level tenant isolation**.

This document outlines TaintBox's three major stretch architectural goals:
1. **Multi-Platform MicroVM Hypervisor Backends** (Linux KVM, Windows WSL2/Hyper-V, macOS Virtualization.framework).
2. **Autonomous Model Parameter Registry via `models.dev`** (dynamic context window sizing, pricing, and tool format detection).
3. **Enterprise Spider Cloud Crawling Pools** (distributed, taint-tracked adversarial web scraping).

---

## 2. Multi-Platform MicroVM Architecture

Standard Docker and Podman containers share the host Linux kernel. When executing arbitrary adversarial agent code, container breakout exploits (eBPF vulnerabilities, `/proc` / `/sys` leaks, kernel privilege escalations) can compromise the host OS. 

MicroVMs solve this by spinning up a minimal, stripped-down guest Linux kernel in **< 100 milliseconds** with only 5MB of RAM per instance.

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                      OPERATOR CLIENT (GUI / TUI / API)                      │
│   Windows (Tauri / Terminal) │ macOS (Native App) │ Linux (CLI / Dashboard) │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │ (Localhost HTTP / vsock / Unix Socket)
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TIER 1: TAINTBOX DAEMON (taintboxd)                      │
│   - Provenance Ledger  - Containment Walls  - Telemetry Stream  - Axum API  │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │ (Hypervisor API / Device IOCTL)
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                   TIER 2: MICROVM HARDWARE ISOLATION                        │
│                                                                             │
│   ┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐   │
│   │   LINUX (Server)    │ │  WINDOWS (Desktop)  │ │   macOS (M-Series)  │   │
│   │   Firecracker / KVM │ │  WSL2 Nested KVM /  │ │  Virtualization.fw  │   │
│   │   /dev/kvm          │ │  Windows Sandbox    │ │  (libkrun / vfkit)  │   │
│   │   ~50ms boot        │ │  Mirrored Localhost │ │  ~120ms boot        │   │
│   └─────────────────────┘ └─────────────────────┘ └─────────────────────┘   │
│                                                                             │
│   Guest VM: Minimal Linux Kernel + Read-Only Base Rootfs + Ephemeral Overlay│
│   Communication: vsock (AF_VSOCK) virtual socket (zero TCP network exposure)│
└─────────────────────────────────────────────────────────────────────────────┘
```

### Platform Implementations

#### A. Linux (Bare-Metal / Cloud VPS)
* **Hypervisor**: **Firecracker** (AWS) or **Cloud-Hypervisor** (Rust-VMM).
* **Interface**: Linux KVM (`/dev/kvm`).
* **Boot Time**: ~45–60 ms.
* **Storage**: Read-only base SquashFS image + memory-backed `overlayfs`. When the sandbox rewinds or terminates, the overlay is discarded instantly.
* **Networking**: TAP device or pure `vsock` (no guest network interface, completely isolating the VM unless explicitly granted egress by boundary policy).

#### B. Windows (WSL2 Subsystem & Windows Sandbox)
* **Approach 1: WSL2 with Nested KVM**:
  * Windows 11 WSL2 supports nested virtualization (`nestedVirtualization=true` in `.wslconfig`).
  * `taintboxd` runs inside WSL2 with direct access to `/dev/kvm`, enabling native Linux microVMs on developer laptops.
  * Windows native GUI (Tauri v2) communicates via mirrored `localhost:8000` with zero latency.
* **Approach 2: Windows Sandbox (Hyper-V Containers)**:
  * Uses Microsoft's native Hyper-V partition isolation (`runhcs`), creating disposable Windows micro-environments for agents executing native Windows binaries and PowerShell scripts.

#### C. macOS (Apple Silicon M-Series)
* **Hypervisor**: Apple **`Virtualization.framework`** / **`Hypervisor.framework`**.
* **Driver**: `libkrun` or `vfkit` compiled into Rust.
* **Performance**: Sub-150ms boot of ARM64 Linux kernels on Apple Silicon (M1/M2/M3/M4) without requiring third-party hypervisors (VirtualBox/VMware).

---

## 3. Dynamic Model Metadata via `models.dev`

Created by the **OpenCode** team (`anomalyco/models.dev`), **models.dev** provides an open-source, community-maintained database of AI model specifications, context windows, token limits, and pricing.

### Proposed Integration in `src/config/models_dev.rs`:

```rust
pub struct ModelSpec {
    pub id: String,
    pub name: String,
    pub context_window: usize,
    pub max_output_tokens: usize,
    pub tool_format: ToolCallingFormat, // XmlTags vs NativeJson
    pub cost_input_per_million: f64,
    pub cost_output_per_million: f64,
}
```

### Benefits for TaintBox:
1. **Dynamic Context Window Sizing**:
   * `ACIHarness::fold_output` automatically calculates the optimal head/tail window based on the model's actual context window (e.g. truncating aggressively for 8k models, but expanding windowing for 128k/1M models).
2. **Autonomous Tool Format Selection**:
   * Models like Qwen 2.5 and DeepSeek prefer XML `<tool_call>` tags, while GPT-4o and Claude 3.5 Sonnet prefer JSON function calling. Models.dev metadata lets TaintBox auto-configure the parser.
3. **Paper 1 Economic Metrics**:
   * Measures token cost per task solved, proving that TaintBox's ACI harness saves up to 40% in token expenses compared to raw terminal execution.

---

## 4. Enterprise Spider Cloud Scraping Pool

* **Integration**: Pool of authenticated Spider Cloud worker nodes managed by `ProviderManager`.
* **Adversarial Web Crawling**: Ingests multi-page web applications for indirect injection defense evaluations.
* **Taint Propagation**: Every crawled asset (HTML, Markdown, extracted PDF text) carries cryptographic provenance hashes back to its source URL.

---

## 5. Lightweight Embedded Session DB & Mid-Session Model Switching

A critical requirement for enterprise developer workflows is **zero context loss across model switches**. When an operator switches from a local offline model (e.g. `qwen2.5-coder`) to a frontier reasoning model (e.g. `claude-3.7-sonnet` or `o3-mini`) mid-turn, the full working memory, variable provenance, active diffs, and tool execution state must remain intact.

### A. Dual-Tier Hot/Cold Memory System

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                       SESSION MEMORY ARCHITECTURE                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  [ HOT MEMORY ]  Lossless Context Manager (LCM)                             │
│   • Active token window sized dynamically to target ModelSpec               │
│   • Live System Prompt + Recent Turn History (Sliding Working Window)       │
│   • Head/Tail Output Windowing on tool stdout/stderr                        │
├─────────────────────────────────────┬───────────────────────────────────────┤
│                                     │ (Automatic Spillover / Compaction)   │
│                                     ▼                                       │
│  [ COLD MEMORY ] 2-Phase Directed Acyclic Graph (DAG)                      │
│   • Vertex: Immutable Turn State (UserPrompt, ToolCall, Observation, Diff)   │
│   • Edge: Causal Provenance Transition + Token Delta                       │
│   • Branching: Multiple alternative solution paths from any checkpoint      │
│   • Rollback: O(1) pointer relocation to ancestor DAG node                  │
├─────────────────────────────────────┴───────────────────────────────────────┤
│  [ STORAGE BACKEND ] Embedded Pure-Rust Engine (redb / SQLite / sled)       │
│   • Zero external daemon dependency (runs in-process)                       │
│   • Serializes turn snapshots, provenance bitmasks, and filesystem diffs    │
│   • Enables session resume across restarts: `tbox resume <session_id>`      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### B. Hot/Cold Memory Implementation Principles:
1. **Lossless Context Management (LCM)**:
   - Compaction retains essential AST symbols, modified file summaries, and error traces while folding repetitive terminal logs.
   - Preserves exact byte hashes of created artifacts so model transitions don't invalidate filesystem tracking.
2. **2-Phase DAG State Machine**:
   - **Phase 1 (Speculative Expansion)**: Agent explores tool calls, branches speculative edits, tests unit tests.
   - **Phase 2 (Commit / Declassify)**: On passing tests or user verification, the branch is merged into the mainline DAG and provenance ledger.

---

## 6. Query & Intent Classification Engine (Paper 2 Defense-in-Depth)

To safeguard agent tools against sophisticated indirect injections, TaintBox investigates automated **query and payload intent classification** to complement heuristic boundary rules.

### Classification Architecture Comparison

| Model Family | Inference Latency | Memory Footprint | Runtime Dependency | Detection Capabilities |
|---|---|---|---|---|
| **Tier 0: Regex & Heuristics** | `< 0.05 ms` | Negligible (`< 1 MB`) | Pure Rust | Instruction overrides, delimiter escapes, common exfiltration patterns |
| **Tier 1: TF-IDF + Random Forest** | `< 0.8 ms` | `~ 5 MB` | `smartcore` / pure Rust | Bag-of-words token distribution shifts, role confusion, multi-turn drift |
| **Tier 2: DistilBERT / ModernBERT ONNX** | `~ 8–15 ms` | `~ 65 MB` | `ort` (ONNX Runtime) | Semantic obfuscation, encoded payloads, indirect social engineering attacks |

### The 2-Phase Cascaded Pipeline:
1. **Fast Filter (Tier 0)**: 95% of normal coding queries pass through instantly with zero noticeable latency.
2. **Feature Extractor & Random Forest (Tier 1)**: Computes Shannon entropy, token frequency anomalies, and sensitive keyword density. Flagged inputs are quarantined.
3. **Deep Semantic Verifier (Tier 2)**: For borderline inputs (e.g. compliance memos, external customer emails), a lightweight local ONNX transformer scores the injection probability before content is passed to the agent.

---

## 7. Dual Research Papers Formulation (Penn State DS 440)

### Paper 1: Agent-Computer Interface (ACI) & Capability Scaling
* **Title**: *Evaluating Tool Ergonomics and Context Compression in Autonomous AI Software Engineering*
* **Core Hypothesis**: Replacing unstructured raw bash shell access with a typed, windowed ACI (`view_lines`, `edit_block`, `search_files`, `snapshot`/`rewind`) improves SWE-bench task completion by `> 30%` while cutting token expenditure by `> 40%`.
* **Lead Author**: Harsh Rathi (supported by Saathvik Sharma & Ammar Al-Sabti).
* **Datasets**: SWE-bench Lite, Terminal-Bench, HumanEval-Rust.

### Paper 2: Data-Level Taint Tracking & Boundary Containment
* **Title**: *Provable Containment of Indirect Prompt Injections via Bitmask Taint Ledgers and Sandbox Boundary Enforcement*
* **Core Hypothesis**: Fine-grained data provenance propagation prevents credential exfiltration and unauthorized workspace modifications from untrusted web feeds with `0%` false positives on standard developer tasks.
* **Lead Author**: Akshat Singhal (supported by Harsh Rathi & Aryamaan Dhuwalia).
* **Datasets**: AgentDojo, InjecAgent, BIPIA, M365 Indirect Attack Corpus.

---

## 8. Multi-Platform MicroVM & OS Virtualization

* **macOS (Apple Silicon M-Series)**:
  * Hypervisor: Apple `Virtualization.framework` via `libkrun` / `vfkit`.
  * Boot Time: `< 120 ms`.
  * RAM: Ephemeral 128MB micro-container per agent session.
* **Linux (Bare-Metal / Cloud)**:
  * Hypervisor: KVM (`/dev/kvm`) + Firecracker / Cloud-Hypervisor.
  * Boot Time: `< 50 ms`.
  * Storage: Read-only SquashFS base rootfs + memory-backed `overlayfs`.
* **Windows 11**:
  * Approach 1: WSL2 with Nested KVM (`.wslconfig: nestedVirtualization=true`).
  * Approach 2: Windows Sandbox (`runhcs`) for native Windows binaries and PowerShell scripts.

---

## 9. Development Milestones

| Milestone | Phase | Focus |
|---|---|---|
| **Phase 1** (Completed) | Capstone Core | Pure Rust runtime, LocalIsolatedRuntime, SWE-agent ACI, Taint Ledger, PromptInjectScanner, OpenCode-grade Zen TUI |
| **Phase 2** (Current Sprint) | Mid-Capstone | `/init` repo indexing, dynamic `models.dev` catalog client, M365 authentic attack corpus loader, live `fetch()` egress gate |
| **Phase 3** (Sprint 2) | Architecture Expansion | Lightweight embedded session DB (redb/SQLite), 2-Phase DAG hot/cold memory, Random Forest query classifier |
| **Phase 4** (Commercial Post-Cap) | Hardware Virtualization | Firecracker/KVM microVM driver, macOS Apple Silicon `Virtualization.framework`, Windows WSL2 KVM bridge |

