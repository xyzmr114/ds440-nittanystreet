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

## 5. Development Milestones

| Milestone | Phase | Focus |
|---|---|---|
| **Phase 1** (Current) | Capstone Core | Pure Rust runtime, LocalIsolatedRuntime, SWE-agent ACI, Taint Ledger, Overkill Walls, TUI, Tauri GUI |
| **Phase 2** (Sprint 1) | Mid-Capstone | gVisor (`runsc`) user-space sandbox driver, Spider Cloud live scraper, Synthetic Attack Generator (Akshat) |
| **Phase 3** (Post-Cap) | Commercial Infra | Firecracker / KVM MicroVM driver, WSL2 nested virtualization bridge, `models.dev` dynamic client |
