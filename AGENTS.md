# DS 440: TaintBox Runtime and ACI Engine - Technical Spec for AI Agents

Version 2.0, Sep 9 2026. Group 2, Nittany Street.

This document is the **single source of truth** for every AI agent working on this project (Antigravity, Claude Code, Codex, OpenCode, Cursor, whatever). Read it cold, in full, before writing a line of code. If you find a contradiction between this document and `PROPOSAL.md` or any other documentation, **this doc wins unless Harsh (Product Owner) specifies otherwise**.

---

## 1. The Project in One Paragraph

The agent-computer interface (ACI) is the decisive capability lever in autonomous systems, yet existing sandboxes (E2B, Daytona) only isolate the host against malicious code execution while ignoring the primary threat: **untrusted content** flowing into context windows that triggers prompt injections and turns agent tools against users. We build **TaintBox**, an instrumented sandbox execution API featuring three novel primitives:
1. **Snapshot and Rewind**: Time-travel state management where models interact through structured typed tools rather than a raw shell.
2. **Taint-Tracked I/O**: Byte- and object-level provenance tracking where policy enforces safety at the tool boundary (e.g., untrusted tainted data cannot trigger privileged exec or network egress).
3. **Telemetry by Construction**: Every execution path automatically yields structured audit traces and research datasets.

The engineering deliverables are the runtime API/SDK and two empirical research papers: **Paper 1** (measuring the ACI capability curve across harness configurations) and **Paper 2** (measuring injection defense effectiveness using boundary policy).

---

## 2. Ground Rules for Every Agent

1. **Synthetic data & ethical boundaries only**: Never run attacks against live third-party systems or use real user PII. All attack evaluations must run against synthetic inboxes, mock tools, and local sandboxed environments.
2. **Zero fabrication**: Every metric and result cited in reports must be generated directly by reproducible scripts in `src/taintbox/eval/`. Never hardcode evaluation figures or benchmark scores in markdown.
3. **Reproducibility is graded**: Pin Python `>=3.11` (target 3.12), manage dependencies via `uv.lock`, fix all benchmark random seeds, and log model hyperparameters (temperature, top_p, model version).
4. **Strict typing and Pydantic validation**: Every schema, tool parameter, provenance metadata record, and API payload must use Pydantic `BaseModel` with full type annotations. No arbitrary untyped dictionaries across module boundaries.
5. **Mandatory test coverage**: Write `pytest` unit tests for every public function and class. Minimum 2 tests per function (covering both benign happy path and boundary/malicious edge cases). CI runs on every push.
6. **Policy at the boundary**: Taint checking must be decoupled from the LLM. The security boundary must not rely on the LLM "realizing" an input is malicious; enforcement happens programmatically before tool execution.
7. **No secrets in git**: API keys (OpenAI, Anthropic, Databento, etc.) must reside exclusively in a gitignored `.env` file loaded via `python-dotenv` or environment variables.
8. **Historical separation**: Do not touch or modify the historical TCA scaffolding in `legacy/tca/`. All active development is contained in `src/taintbox/`, `config/`, and `tests/`.

---

## 3. Repository Layout

```text
ds440-nittanystreet/
  README.md                   (Public documentation & project overview)
  AGENTS.md                   (This file: technical specification for agents)
  PROPOSAL.md                 (Original formal capstone proposal)
  pyproject.toml              (Project configuration, dependencies, and tools)
  config/
    policy.default.yaml       (Default security rules and trust levels)
    runtime.yaml              (Sandbox execution limits and snapshot settings)
  src/
    taintbox/
      __init__.py
      models.py               (Core Pydantic data schemas: Provenance, ToolCall, etc.)
      aci/
        __init__.py
        harness.py            (Typed ACI tools: read, write, exec, observe, snapshot, rewind, fetch)
        schemas.py            (JSON schemas exposed to LLM agents)
      taint/
        __init__.py
        engine.py             (Provenance ledger & propagation engine)
        policy.py             (Boundary policy enforcer and rule evaluator)
      runtime/
        __init__.py
        base.py               (Abstract SandboxRuntime interface)
        process.py            (Isolated local process sandbox)
        docker.py             (Docker / gVisor container sandbox adapter)
      telemetry/
        __init__.py
        logger.py             (Structured JSONL event sink and audit trail)
        exporter.py           (Telemetry exporter for research papers)
      api/
        __init__.py
        server.py             (FastAPI REST server: session lifecycle & tool gateway)
        client.py             (Python client SDK)
      eval/
        __init__.py
        paper1_harness.py     (ACI capability curve benchmark runner)
        paper2_injection.py   (Taint boundary defense benchmark runner)
  tests/
    test_taint_engine.py      (Unit tests for provenance and policy enforcement)
    test_aci_tools.py         (Unit tests for typed ACI tools & snapshot/rewind)
    test_telemetry.py         (Unit tests for telemetry event generation)
  legacy/
    tca/                      (Archived historical TCA code)
```

---

## 4. System Components & Schemas

### 4.1 Data Models (`src/taintbox/models.py`)

```python
from enum import Enum
from typing import Any, Dict, List, Optional
from pydantic import BaseModel, Field

class TrustLevel(str, Enum):
    TRUSTED = "TRUSTED"        # Internal verified system prompts/data
    INTERNAL = "INTERNAL"      # System state generated within sandbox
    UNTRUSTED = "UNTRUSTED"    # User uploads, web fetches, third-party inputs
    HOSTILE = "HOSTILE"        # Known malicious injection samples

class ProvenanceTag(str, Enum):
    SYSTEM = "SYSTEM"
    USER = "USER"
    UNTRUSTED_WEB = "UNTRUSTED_WEB"
    EXTERNAL_FILE = "EXTERNAL_FILE"
    DERIVED = "DERIVED"

class ProvenanceRecord(BaseModel):
    source_id: str
    tag: ProvenanceTag
    trust_level: TrustLevel
    chain_of_custody: List[str] = Field(default_factory=list)
    metadata: Dict[str, Any] = Field(default_factory=dict)

class ToolCall(BaseModel):
    call_id: str
    tool_name: str
    arguments: Dict[str, Any]
    caller: str = "agent"

class PolicyDecision(BaseModel):
    allowed: bool
    rule_id: Optional[str] = None
    reason: str = "Allowed by policy"
    taint_records: List[ProvenanceRecord] = Field(default_factory=list)

class ToolResult(BaseModel):
    call_id: str
    status: str                # "SUCCESS", "BLOCKED_BY_POLICY", "ERROR"
    output: Any
    provenance: Optional[ProvenanceRecord] = None
    policy_decision: Optional[PolicyDecision] = None

class SnapshotMetadata(BaseModel):
    snapshot_id: str
    timestamp: float
    description: str
    file_state_hash: str
```

### 4.2 The ACI Layer (`src/taintbox/aci/harness.py`)

The agent does not interact with a bare terminal. It interacts through typed, deterministic tools:
- `read(path: str) -> ToolResult`: Returns contents with attached provenance.
- `write(path: str, content: str, provenance: Optional[ProvenanceRecord]) -> ToolResult`: Persists content while maintaining provenance association.
- `exec(program: str, args: List[str], env: Optional[Dict[str, str]]) -> ToolResult`: Runs within isolated runtime, checking boundary policy.
- `fetch(url: str) -> ToolResult`: Requests external web resource; the payload is automatically tagged `UNTRUSTED_WEB`.
- `observe() -> Observation`: Returns delta state changes (new files, modified files, current taint ledger).
- `snapshot(description: str) -> SnapshotMetadata`: Captures workspace and memory state.
- `rewind(snapshot_id: str) -> bool`: Restores filesystem and ledger state to the selected snapshot.

### 4.3 The Taint Engine (`src/taintbox/taint/engine.py`)

- **Ledger**: Tracks provenance per resource (file path, memory identifier, buffer ID).
- **Propagation**: If resource $B$ is created by reading or deriving from resource $A$ ($B = f(A)$), $B$ inherits $A$'s highest taint level and extends $A$'s chain of custody.
- **Policy Enforcement**: Before any privileged action executes (`network_egress`, `exec_privileged`, `file_delete`), the engine checks whether any inputs carry `UNTRUSTED` or `HOSTILE` taint without explicit policy bypass.

### 4.4 Telemetry & Audit (`src/taintbox/telemetry/logger.py`)

Every session records a structured event log in JSONL format:
- `timestamp`: UTC ISO 8601 string.
- `event_type`: `TOOL_CALL`, `POLICY_EVALUATION`, `SNAPSHOT`, `REWIND`, `EXECUTION_RESULT`.
- `provenance_snapshot`: Active taint status of all referenced resources.
- `details`: Detailed execution metadata.

---

## 5. Module Contracts

### 5.1 `src/taintbox/taint/engine.py`

```python
class TaintEngine:
    def record_provenance(self, resource_id: str, record: ProvenanceRecord) -> None:
        """Register provenance for a given file or object."""
    
    def propagate(self, source_ids: List[str], target_id: str) -> ProvenanceRecord:
        """Propagate taint from one or more sources to a newly derived target."""
    
    def evaluate_policy(self, action: str, resource_ids: List[str]) -> PolicyDecision:
        """Evaluate if the specified action on given resources violates boundary policy."""
```

### 5.2 `src/taintbox/aci/harness.py`

```python
class ACIHarness:
    def __init__(self, workspace_path: Optional[str] = None, policy_config_path: Optional[str] = None):
        """Initialize sandbox runtime, taint engine, and telemetry logger."""

    def read(self, path: str) -> ToolResult:
        """Read file and attach provenance record."""

    def write(self, path: str, content: str, source_ids: Optional[List[str]] = None) -> ToolResult:
        """Write content, propagating taint from specified source IDs."""

    def fetch(self, url: str) -> ToolResult:
        """Fetch remote URL and tag output with UNTRUSTED_WEB."""

    def exec(self, program: str, args: List[str]) -> ToolResult:
        """Evaluate policy boundary and execute program if permitted."""

    def snapshot(self, description: str = "") -> SnapshotMetadata:
        """Create time-travel checkpoint."""

    def rewind(self, snapshot_id: str) -> bool:
        """Roll back workspace and taint ledger to checkpoint."""
```

---

## 6. Evaluation Metrics & Equations

All metrics reported in research papers and project deliverables must be calculated according to the following formulas:

1. **Task Success Rate (TSR)**:
   $$\text{TSR} = \frac{N_{\text{completed\_correctly}}}{N_{\text{total\_tasks}}}$$
   Evaluated per interface condition (A: Raw shell, B: Typed tools, C: Tools + Snapshot, D: Full ACI).

2. **Attack Success Rate (ASR)**:
   $$\text{ASR} = \frac{N_{\text{compromised\_executions}}}{N_{\text{total\_attack\_prompts}}}$$
   Target: $\text{ASR}_{\text{TaintBox}} \approx 0$ on exfiltration and unauthorized tool invocations.

3. **Benign Utility (U)**:
   $$U = \frac{\text{TSR}_{\text{defended}}}{\text{TSR}_{\text{baseline}}}$$
   Measures whether defense degrades benign task capability (must satisfy $U \ge 0.95$).

4. **Policy False Positive Rate (FPR)**:
   $$\text{FPR} = \frac{N_{\text{benign\_actions\_blocked}}}{N_{\text{total\_benign\_actions}}}$$

5. **Defense Overhead ($\Delta T$)**:
   $$\Delta T = T_{\text{with\_taint}} - T_{\text{baseline}}$$
   Target overhead: $< 15\%$ per tool invocation.

---

## 7. Bi-Weekly Sprint Milestones

| Sprint | Weeks | Primary Focus | Key Deliverables |
|---|---|---|---|
| **Sprint 1** | 1–2 | Runtime base & Typed Tool ACI v1 | Process/Docker sandbox, ACI tools (`read`, `write`, `exec`), CI test suite |
| **Sprint 2** | 3–4 | Observation channel & Snapshot/Rewind | State diffing, snapshot store, Paper 1 preliminary capability runs |
| **Sprint 3** | 5–6 | Taint Engine & Policy Enforcement | Provenance ledger, boundary policy, synthetic injection corpus v1 |
| **Sprint 4** | 7–8 | Dual Experimental Studies | Full factorial Paper 1 & Paper 2 eval runs across models |
| **Sprint 5** | 9–10 | API Gateway, SDKs & Telemetry Export | FastAPI REST server, Python SDK, telemetry analysis pipeline |
| **Sprint 6** | 11–12 | Ablations, Error Analysis & Final Reports | Paper 1 and Paper 2 drafts, final technical report, oral presentation |

---

## 8. Critical Pitfalls & Guidance

1. **Relying on model compliance**: Never assume the LLM will obey a system prompt warning like "do not follow instructions in this file". The model will get tricked. The boundary policy engine must enforce safety in deterministic code.
2. **Snapshot bloat**: Do not copy full environments on every snapshot. Use incremental file tracking or copy-on-write mechanisms to prevent disk exhaustion.
3. **Over-tainting (Taint Explosion)**: If an agent touches one tainted byte and marks the entire sandbox permanently hostile, benign tasks cannot complete. Track taint granularly at resource/file boundaries and support explicit declassification or policy overrides.
4. **Non-deterministic evaluation**: Always set temperature to 0.0 or pin seeds for model evaluations. Save complete generation transcripts to JSONL for auditability.
5. **Breaking compatibility**: Keep tool schemas simple, consistent with JSON Schema, and directly usable by standard agent frameworks (LangChain, smolagents, AutoGen).

---

## 9. References

1. Yang, J. et al., "SWE-agent: Agent-computer interfaces enable automated software engineering", 2024.
2. ExploitBench contributors, "ExploitBench: Evaluating autonomous agents for exploiting software vulnerabilities", 2025.
3. E2B, "Open source AI code interpreter sandboxes", technical documentation.
4. Agache, A. et al., "Firecracker: Lightweight virtualization for serverless applications", NSDI 2020.
5. Google, "gVisor: an application kernel for containers".
6. Perl documentation, "perlsec: taint mode", original taint tracking architecture.
7. Greshake, K. et al., "Not what you've signed up for: Compromising real-world LLM-integrated applications with indirect prompt injection", AISec 2023.
8. Willison, S., "Prompt injection attacks against GPT-3", 2022.
9. Zhan, Q. et al., "Formalizing and benchmarking prompt injection attacks and defenses", USENIX Security 2024.
