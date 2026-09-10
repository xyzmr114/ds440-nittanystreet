# TaintBox: Production Roadmap & Stretch Goals

**Document Status:** Architectural Vision & Production Strategy  
**Authors:** Group 2 (Nittany Street) — Penn State DS 440  
**Target:** Post-Capstone Enterprise Sandbox & Commercial Deployment  

---

## 1. Executive Strategy

While TaintBox's current baseline (`LocalIsolatedRuntime` in pure Rust) delivers portable, zero-overhead execution across Windows, macOS, and Linux for academic research, commercial deployments (e.g. Modal, E2B, AWS Lambda) demand **hardware-level tenant isolation** and **advanced autonomous runtime safeguards**.

This document outlines TaintBox's advanced stretch architectural goals, integrating production-proven patterns from **Overkill** (`Sahaj-Tech-ltd/overkill`) and **OpenCode**:

1. **Multi-Platform MicroVM Hypervisor Backends** (Linux KVM, Windows WSL2/Hyper-V, macOS Virtualization.framework).
2. **Autonomous Model Parameter Registry via `models.dev`** (dynamic context window sizing, pricing, and tool format detection).
3. **Enterprise Spider Cloud Crawling Pools** (distributed, taint-tracked adversarial web scraping).
4. **Dual-Tier Hot/Cold Memory System** (Lossless Context Manager + 2-Phase DAG).
5. **Two-Tier Runtime Query & Intent Classifier** (TF-IDF Random Forest vs. ONNX Transformer).
6. **Dynamic Model Recalibration Engine (Overkill Port)** (probed schema adaptation on mid-session model swaps).
7. **Anti-Sycophancy & Falsifiable Hypothesis Gate (Overkill Port)** (guards against hallucinatory compliance on destructive operations).
8. **Cryptographic Flight Recorder (Journal Narration & Replay)** (tamper-proof signed execution bundles).
9. **Behavior Anomaly Ticker & Sliding-Window E-Stop (Overkill Port)** (circuit breaker for runaway agent loops).
10. **OpenCode-Grade Browser Web IDE** (full cloud IDE with file tree, editor, split diffs, and terminal dock).

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

---

## 3. Advanced Overkill-Inspired Architectural Exclusives

### A. Dynamic Model Swap Recalibration Engine
* **Origin**: Overkill `recalibration.go`.
* **Problem**: When an operator switches inference models mid-session (e.g. Claude 3.7 Sonnet &rarr; Qwen 2.5 Coder), competence and tool formats shift drastically. Claude excels with native JSON schemas, whereas Qwen prefers XML tool blocks (`<tool_call>`).
* **Solution**: On model swap, TaintBox executes an automated 3-turn probe suite in an isolated sub-sandbox:
  1. Probe 1: JSON vs. XML tool invocation accuracy.
  2. Probe 2: Windowed file inspection capability (`view_lines`).
  3. Probe 3: Delimiter resilience against injected prompt overrides.
* The system adjusts prompting templates and tool windowing parameters automatically with zero operator intervention.

### B. Anti-Sycophancy & Falsifiable Hypothesis Gate
* **Origin**: Overkill `failhypo_cmd.go` and `sycophancy_adapter.go`.
* **Problem**: LLMs suffer from severe sycophancy—they agree with flawed user instructions and hallucinate success even when tests fail.
* **Solution**: Before executing any destructive operation (`rm`, `git reset`, `edit_block` on critical core modules):
  1. The agent must emit a **Falsifiable Failure Hypothesis**: what could go wrong, and what metric proves failure?
  2. The boundary policy enforcer compares the agent's hypothesis against pre-execution sandbox hashes.
  3. If tests or assertions fail, the agent is barred from apologizing or making excuse claims; it must immediately trigger `/rewind`.

### C. Cryptographic Flight Recorder (Journal Narration & Replay)
* **Origin**: Overkill `journal_narrate.go` and `journal_replay.go`.
* **Problem**: Security post-mortems require verifiable proof of how an injection payload penetrated defenses.
* **Solution**: TaintBox writes an append-only, HMAC-SHA256 signed JSONL journal recording:
  - Byte-exact stdin/stdout of every tool step.
  - Taint bitmask delta per file mutation.
  - PromptInject scanner threat vectors and classification scores.
  - The journal can be replayed deterministically in a sterile sandbox via `taintbox replay <log.jsonl>`.

### D. Behavior Anomaly Ticker & Sliding-Window E-Stop
* **Origin**: Overkill `behavior_ticker.go` and `estop.go`.
* **Problem**: Runaway agent loops can burn massive token budgets and spam shell commands in repetitive failure loops.
* **Solution**: A sliding-window entropy analyzer tracking the last 10 tool invocations:
  - If tool name and argument repetition exceeds 80%, trigger **CIRCUIT_TRIPPED**.
  - If directory traversal depth exceeds 6 levels, halt with **ANOMALY_HALT**.
  - Trips the physical Emergency Stop circuit, severing API communication and rolling back state.

---

## 4. OpenCode-Grade Browser Web IDE

To complement the native Ratatui terminal application, TaintBox envisions an OpenCode-style browser Web IDE:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  ⚡ TAINTBOX WEB IDE                                    [qwen2.5-coder] [LIVE]│
├───────────────┬───────────────────────────────────────────────┬─────────────┤
│ FILE EXPLORER │ EDITOR: src/aci/harness.rs                    │ TAINT GRAPH │
│               │                                               │             │
│ ▼ src/        │  1 │ pub struct ACIHarness {                  │ [Clean]     │
│   ▼ aci/      │  2 │     pub runtime: LocalIsolatedRuntime,   │   src/      │
│     harness.rs│  3 │     pub taint_engine: TaintEngine,       │             │
│     loop.rs   │  4 │ }                                        │ [Tainted]   │
│   ▼ taint/    │  5 │                                          │   inbox/    │
│     engine.rs │  6 │ impl ACIHarness {                        │   invoice   │
│   AGENTS.md   │  7 │     pub fn exec(&mut self, cmd: &str)    │             │
│               │    │                                          │ [Rule-004]  │
├───────────────┴───────────────────────────────────────────────┴─────────────┤
│ TERMINAL & DIFF DRAWER                                                      │
│ $ /attack m365                                                              │
│ [POLICY BLOCK] Rule-004: Network egress on tainted payload blocked.         │
└─────────────────────────────────────────────────────────────────────────────┘
```

* **Features**:
  * Multi-tab Monaco / CodeMirror code editor with Rust and JSON syntax highlighting.
  * Interactive Git diff inspector with side-by-side addition/deletion view.
  * Real-time WebSocket terminal running the full `tbox Zen` session.
  * Visual Bitmask Taint Graph rendering node contamination in real time.

---

## 5. Dual-Tier Hot/Cold Memory Systems

```text
┌────────────────────────────────────────────────────────┐
│              OPERATOR & MODEL CONVERSATION             │
└──────────────────────────┬─────────────────────────────┘
                           │
             ┌─────────────┴─────────────┐
             ▼                           ▼
┌─────────────────────────┐ ┌─────────────────────────┐
│       HOT MEMORY        │ │       COLD MEMORY       │
│  Lossless Context (LCM) │ │     2-Phase Session DAG │
│  - Active conversation  │ │  - Immutable turns DB   │
│  - In-memory ring buffer│ │  - Causal execution graph│
│  - Formatted per model  │ │  - Checkpoint snapshots │
│  - Sub-millisecond read │ │  - SQLite / redb store  │
└─────────────────────────┘ └─────────────────────────┘
```

1. **Hot Memory**: Lossless Context Manager (LCM) dynamically scaled to the active model's context window (`ModelSpec.context_window`).
2. **Cold Memory**: Immutable 2-Phase DAG where every turn is preserved as a DAG node connected by causal tool/provenance edges.

---

## 6. Two-Tier Runtime Query & Intent Classifier

* **Tier 1 (Sub-millisecond Triage)**: Pure-Rust TF-IDF feature extraction + Random Forest classifier (`smartcore`). Classifies tool input intent in `< 0.8ms`.
* **Tier 2 (Deep Semantic Evaluation)**: ONNX Runtime transformer (ModernBERT/DistilBERT via `ort`). Evaluates ambiguous queries in `8–15ms`.
