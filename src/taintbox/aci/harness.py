"""ACI Harness: Structured, Typed Tools with Snapshot-Rewind and Taint-Tracked I/O."""

from __future__ import annotations

import tempfile
import time
import uuid
from typing import Any, Dict, List, Optional

from taintbox.models import (
    Observation,
    PolicyDecision,
    ProvenanceRecord,
    ProvenanceTag,
    SnapshotMetadata,
    ToolResult,
    TrustLevel,
)
from taintbox.runtime.base import LocalIsolatedRuntime, SandboxRuntime
from taintbox.taint.engine import TaintEngine
from taintbox.telemetry.logger import TelemetryLogger

NETWORK_PROGRAMS = {"curl", "wget", "nc", "ncat", "ssh", "scp", "ftp"}
DELETE_PROGRAMS = {"rm", "del", "unlink", "shred"}


class ACIHarness:
    """The Agent-Computer Interface execution harness.
    
    Replaces raw shell access with structured, typed tools equipped with:
    1. Snapshot and Rewind
    2. Taint-Tracked I/O
    3. Telemetry by construction
    """

    def __init__(
        self,
        workspace_dir: Optional[str] = None,
        runtime: Optional[SandboxRuntime] = None,
        taint_engine: Optional[TaintEngine] = None,
        telemetry_logger: Optional[TelemetryLogger] = None,
    ):
        if runtime is not None:
            self.runtime = runtime
        else:
            self._temp_dir = tempfile.mkdtemp(prefix="taintbox_ws_") if workspace_dir is None else None
            effective_dir = workspace_dir or self._temp_dir
            self.runtime = LocalIsolatedRuntime(root_dir=effective_dir)

        self.taint_engine = taint_engine or TaintEngine()
        self.telemetry = telemetry_logger or TelemetryLogger()
        self._snapshots: Dict[str, SnapshotMetadata] = {}
        self._step_counter = 0

    def read(self, path: str) -> ToolResult:
        """Read a file from the sandbox, returning contents and provenance metadata."""
        call_id = str(uuid.uuid4())
        try:
            content = self.runtime.read_file(path)
            rec = self.taint_engine.get_provenance(path)
            if not rec:
                rec = ProvenanceRecord(
                    source_id=path,
                    tag=ProvenanceTag.SYSTEM,
                    trust_level=TrustLevel.INTERNAL,
                )
                self.taint_engine.record_provenance(path, rec)

            self.telemetry.log_event("TOOL_READ", {"path": path, "size": len(content)})
            return ToolResult(
                call_id=call_id,
                tool_name="read",
                status="SUCCESS",
                output=content,
                provenance=rec,
            )
        except Exception as e:
            return ToolResult(
                call_id=call_id,
                tool_name="read",
                status="ERROR",
                output=None,
                error=str(e),
            )

    def write(
        self,
        path: str,
        content: str,
        source_ids: Optional[List[str]] = None,
    ) -> ToolResult:
        """Write content to a file, propagating provenance from specified source IDs."""
        call_id = str(uuid.uuid4())
        try:
            self.runtime.write_file(path, content)
            if source_ids:
                rec = self.taint_engine.propagate(source_ids, path)
            else:
                rec = ProvenanceRecord(
                    source_id=path,
                    tag=ProvenanceTag.USER,
                    trust_level=TrustLevel.INTERNAL,
                )
                self.taint_engine.record_provenance(path, rec)

            self.telemetry.log_event(
                "TOOL_WRITE",
                {"path": path, "size": len(content), "sources": source_ids or []},
            )
            return ToolResult(
                call_id=call_id,
                tool_name="write",
                status="SUCCESS",
                output=f"Successfully wrote {len(content)} characters to {path}",
                provenance=rec,
            )
        except Exception as e:
            return ToolResult(
                call_id=call_id,
                tool_name="write",
                status="ERROR",
                output=None,
                error=str(e),
            )

    def fetch(
        self,
        url: str,
        save_as: Optional[str] = None,
        mock_content: Optional[str] = None,
    ) -> ToolResult:
        """Fetch remote web resource. The payload is automatically tagged UNTRUSTED_WEB."""
        call_id = str(uuid.uuid4())
        target_path = save_as or f"downloads/{abs(hash(url))}.txt"
        payload = mock_content if mock_content is not None else f"Simulated content from {url}"

        self.runtime.write_file(target_path, payload)
        record = ProvenanceRecord(
            source_id=target_path,
            tag=ProvenanceTag.UNTRUSTED_WEB,
            trust_level=TrustLevel.UNTRUSTED,
            chain_of_custody=[url],
            metadata={"origin_url": url},
        )
        self.taint_engine.record_provenance(target_path, record)

        self.telemetry.log_event(
            "TOOL_FETCH",
            {"url": url, "saved_to": target_path, "trust_level": record.trust_level.value},
        )

        return ToolResult(
            call_id=call_id,
            tool_name="fetch",
            status="SUCCESS",
            output=payload,
            provenance=record,
        )

    def exec(
        self,
        program: str,
        args: List[str],
        env: Optional[Dict[str, str]] = None,
    ) -> ToolResult:
        """Execute a program in the sandbox, checking boundary policy against tainted inputs."""
        call_id = str(uuid.uuid4())

        # Determine action classification
        if program.lower() in NETWORK_PROGRAMS:
            action = "network_egress"
        elif program.lower() in DELETE_PROGRAMS:
            action = "file_delete"
        else:
            action = "exec"

        # Check all files in arguments for taint
        referenced_resources = []
        for arg in args:
            clean_arg = arg.lstrip("@").strip()
            if self.runtime.file_exists(clean_arg):
                referenced_resources.append(clean_arg)

        decision = self.taint_engine.evaluate_policy(action, referenced_resources)

        if not decision.allowed:
            self.telemetry.log_event(
                "POLICY_BLOCK",
                {
                    "program": program,
                    "action": action,
                    "args": args,
                    "reason": decision.reason,
                },
            )
            return ToolResult(
                call_id=call_id,
                tool_name="exec",
                status="BLOCKED_BY_POLICY",
                output=None,
                error=decision.reason,
                policy_decision=decision,
            )

        exit_code, stdout, stderr = self.runtime.execute_command(program, args, env)

        self.telemetry.log_event(
            "TOOL_EXEC",
            {"program": program, "args": args, "exit_code": exit_code},
        )

        return ToolResult(
            call_id=call_id,
            tool_name="exec",
            status="SUCCESS" if exit_code == 0 else "ERROR",
            output={"exit_code": exit_code, "stdout": stdout, "stderr": stderr},
            error=stderr if exit_code != 0 else None,
            policy_decision=decision,
        )

    def snapshot(self, description: str = "") -> SnapshotMetadata:
        """Create a time-travel execution checkpoint."""
        snap_id = f"snap_{int(time.time() * 1000)}"
        file_hashes = self.runtime.create_snapshot(snap_id)
        ledger_state = self.taint_engine.export_state()

        meta = SnapshotMetadata(
            snapshot_id=snap_id,
            description=description,
            file_hashes=file_hashes,
            taint_ledger_state=ledger_state,
        )
        self._snapshots[snap_id] = meta

        self.telemetry.log_event("SNAPSHOT_CREATED", {"snapshot_id": snap_id, "description": description})
        return meta

    def rewind(self, snapshot_id: str) -> bool:
        """Roll back the sandbox filesystem and taint ledger to a checkpoint."""
        if snapshot_id not in self._snapshots:
            raise ValueError(f"Unknown snapshot: {snapshot_id}")

        meta = self._snapshots[snapshot_id]
        self.runtime.restore_snapshot(snapshot_id)
        self.taint_engine.restore_state(meta.taint_ledger_state)

        self.telemetry.log_event("SNAPSHOT_REWOUND", {"snapshot_id": snapshot_id})
        return True

    def observe(self) -> Observation:
        """Return state diffs and active taint inventory."""
        self._step_counter += 1
        all_files = self.runtime.list_files()
        tainted = self.taint_engine.list_tainted_resources()

        obs = Observation(
            step_id=self._step_counter,
            modified_files=[],
            created_files=all_files,
            deleted_files=[],
            active_taint_count=len(tainted),
            tainted_resources=tainted,
        )
        return obs
