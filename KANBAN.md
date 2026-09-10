# TaintBox Sprint 1 Multi-Agent Kanban Board

**Repository:** `xyzmr114/ds440-nittanystreet` | **Sprint:** 1 (Foundation, Parity & Research Rig)  
**Scrum Master:** Harsh Rathi | **Team Lead:** Aryamaan  
**Integration Support:** GitHub Projects & Asana MCP

---

## 1. Live Interactive Board

> [!TIP]
> This Kanban board is rendered live inside the **TaintBox Desktop / Web Application** at `http://localhost:8000/?view=kanban` or by opening the Hamburger menu &rarr; **Agent Kanban**.

```text
┌─────────────────┬─────────────────┬─────────────────┬─────────────────┬─────────────────┐
│ 1. BACKLOG      │ 2. PLANNING     │ 3. SANDBOX EXEC │ 4. TAINT AUDIT  │ 5. VERIFIED     │
├─────────────────┼─────────────────┼─────────────────┼─────────────────┼─────────────────┤
│ [TSK-105]       │ [TSK-103]       │ [TSK-102]       │ [TSK-101]       │ [TSK-100]       │
│ Safe Secret     │ 2-Phase Intent  │ 2-Phase DAG     │ M365 SOX Invc   │ 1:1 OpenCode UI │
│ Sharing Vault   │ Triage (ONNX)   │ State Tree      │ Exfil Blocked   │ Parity (99%)    │
│ 👤 Harsh        │ 👤 Akshat       │ 👤 Aryamaan     │ 👤 Ammar        │ 👤 Harsh Rathi  │
│ [High Priority] │ [0x00 Clean]    │ [Sandbox #2]    │ [Bit 0: Web]    │ [60/60 Tests]   │
│                 │                 │                 │                 │                 │
│ [TSK-106]       │ [TSK-107]       │                 │                 │ [TSK-104]       │
│ OpenRouter      │ Paper 1 & 2     │                 │                 │ Single Executable│
│ 429 Failover    │ Draft Outlines  │                 │                 │ Daemon + UI Exe │
│ 👤 Saathvik     │ 👤 4-Author Team│                 │                 │ 👤 Saathvik     │
│ [Resilience]    │ [Academic Rig]  │                 │                 │ [Release Ready] │
└─────────────────┴─────────────────┴─────────────────┴─────────────────┴─────────────────┘
```

---

## 2. Detailed Task Cards

### Column 1: Backlog & Intake
* **`[TSK-105]` Safe Secret Sharing & `.env` In-Memory Vault**
  * **Owner:** Harsh Rathi (Scrum Master)
  * **Status:** High Priority Backlog
  * **Scope:** Parse `.env` at sandbox initialization; mask plaintext credentials into `<SECRET:TOKEN>` sentinels before model context ingestion; inject raw secrets strictly at the ephemeral runtime tool execution boundary.
* **`[TSK-106]` OpenRouter Resilient Fallback Pool**
  * **Owner:** Saathvik (Backend Lead)
  * **Status:** In Intake
  * **Scope:** Automatic circuit-breaker failover to Claude 3.5 Sonnet / Llama 3.3 on HTTP 429 rate limit or primary provider degradation.

### Column 2: Agent Planning & Falsifiable Hypotheses
* **`[TSK-103]` Two-Tier Query Intent Classifier (TF-IDF + ONNX DistilBERT)**
  * **Owner:** Akshat (Attack Vectoring & Security Lead)
  * **Status:** Active Planning
  * **Scope:** Fast sub-millisecond triage of incoming queries (`BENIGN_QUERY`, `CODE_EDIT`, `SYSTEM_ADMIN`, `ADVERSARIAL_PROBE`) using `smartcore` Random Forest with ONNX deep semantic fallback.
* **`[TSK-107]` Research Paper 1 & 2 Execution Drafts**
  * **Owner:** Aryamaan (Lead P1), Ammar (Co-author P1), Akshat (Lead P2), Saathvik (Co-author P1 & P2)
  * **Status:** Outlining Empirical Data Collection

### Column 3: Sandbox Execution (MicroVM / Tempfs)
* **`[TSK-102]` Causal 2-Phase DAG State Tree Refactoring**
  * **Owner:** Aryamaan (System Architect)
  * **Status:** Executing in Sandbox #2
  * **Scope:** Transition linear turn history to an immutable Directed Acyclic Graph preserving causal tool and taint edges for instantaneous branching and rewind.

### Column 4: Security & Taint Audit (Containment Walls)
* **`[TSK-101]` M365 Copilot Indirect Email Exfiltration Defense**
  * **Owner:** Ammar (Data & Testing Lead)
  * **Status:** **INTERCEPTED BY POLICY**
  * **Provenance:** `TaintMask::UNTRUSTED_INPUT` (Bit 0)
  * **Outcome:** Malicious indirect payload embedded in `invoice.eml` attempted to invoke `exec(curl ...)` with stolen `.env` keys. Intercepted and blocked before process spawn.

### Column 5: Verified & Completed (Sprint 1 Production)
* **`[TSK-100]` 1:1 OpenCode Desktop Parity & Rebranding to `tbox`**
  * **Owner:** Harsh Rathi
  * **Status:** **Verified by Subagent (99.0% Visual Fidelity)**
  * **Deliverable:** Welcome View, Sessions View, Hamburger Dropdown, Split Conversation & 16-field Context Inspector.
* **`[TSK-104]` Single-Binary Daemon & UI Integration (`tbox.exe`)**
  * **Owner:** Saathvik
  * **Status:** **Verified & Deployed**
  * **Deliverable:** Self-contained release binary (`9,133,056 bytes`) serving embedded UI on port 8000 and auto-launching browser. All 60 tests passing.

---

## 3. GitHub Projects & Asana MCP Synchronization

### GitHub Projects Integration
TaintBox tasks map directly to **GitHub Projects (v2)**:
1. Navigate to `https://github.com/xyzmr114/ds440-nittanystreet/projects`.
2. Create a new Board project using the **Board** layout.
3. Add the 5 custom workflow status columns: `Backlog`, `Planning`, `Sandbox Exec`, `Taint Audit`, `Verified`.

### Asana MCP Server Integration
To drive TaintBox task management via Asana:
```json
{
  "mcpServers": {
    "asana": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-asana"],
      "env": {
        "ASANA_ACCESS_TOKEN": "<YOUR_ASANA_PAT>"
      }
    }
  }
}
```
When configured, agents automatically synchronize tool step completion and taint alerts directly to your Asana project workspace.
