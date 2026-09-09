"""Paper 1 Harness: The Agent-Computer Interface Capability Curve Evaluator."""

from __future__ import annotations

import tempfile
import time
from enum import Enum
from typing import Any, Callable, Dict, List, Optional
from pydantic import BaseModel, Field

from taintbox.aci.harness import ACIHarness
from taintbox.models import Observation, SnapshotMetadata, ToolResult


class InterfaceCondition(str, Enum):
    CONDITION_A_RAW_SHELL = "CONDITION_A_RAW_SHELL"              # Raw shell, no structured tools, no snapshot
    CONDITION_B_TYPED_TOOLS = "CONDITION_B_TYPED_TOOLS"          # Typed tools (read, write, exec), no snapshot
    CONDITION_C_SNAPSHOT_REWIND = "CONDITION_C_SNAPSHOT_REWIND"  # Typed tools + snapshot/rewind, no prov surfaced
    CONDITION_D_FULL_ACI = "CONDITION_D_FULL_ACI"                # Full ACI with provenance surfaced to model


class BenchmarkTask(BaseModel):
    task_id: str
    name: str
    instructions: str
    initial_files: Dict[str, str] = Field(default_factory=dict)
    expected_output_files: List[str] = Field(default_factory=list)


class ConditionRestrictedHarness(ACIHarness):
    """ACIHarness enforcing interface constraints corresponding to experimental conditions."""

    def __init__(
        self,
        condition: InterfaceCondition,
        workspace_dir: Optional[str] = None,
        **kwargs,
    ):
        super().__init__(workspace_dir=workspace_dir, **kwargs)
        self.condition = condition

    def read(self, path: str) -> ToolResult:
        if self.condition == InterfaceCondition.CONDITION_A_RAW_SHELL:
            raise PermissionError("Typed read is disabled in Condition A (raw shell only).")
        return super().read(path)

    def write(self, path: str, content: str, source_ids: Optional[List[str]] = None) -> ToolResult:
        if self.condition == InterfaceCondition.CONDITION_A_RAW_SHELL:
            raise PermissionError("Typed write is disabled in Condition A (raw shell only).")
        return super().write(path, content, source_ids=source_ids)

    def snapshot(self, description: str = "") -> SnapshotMetadata:
        if self.condition in (
            InterfaceCondition.CONDITION_A_RAW_SHELL,
            InterfaceCondition.CONDITION_B_TYPED_TOOLS,
        ):
            raise PermissionError(f"Snapshot is disabled in Condition B (typed tools only).")
        return super().snapshot(description)

    def rewind(self, snapshot_id: str) -> bool:
        if self.condition in (
            InterfaceCondition.CONDITION_A_RAW_SHELL,
            InterfaceCondition.CONDITION_B_TYPED_TOOLS,
        ):
            raise PermissionError(f"Rewind is disabled in Condition B.")
        return super().rewind(snapshot_id)

    def observe(self) -> Observation:
        if self.condition == InterfaceCondition.CONDITION_A_RAW_SHELL:
            raise PermissionError("Observation channel is disabled in Condition A.")
        obs = super().observe()
        if self.condition != InterfaceCondition.CONDITION_D_FULL_ACI:
            # Provenance is only surfaced to the model in Condition D
            obs.provenance_context = {}
        return obs


class CapabilityHarnessRunner:
    """Orchestrates benchmark task runs across the 4 interface conditions for Paper 1."""

    def __init__(
        self,
        condition: InterfaceCondition,
        workspace_dir: Optional[str] = None,
    ):
        self.condition = condition
        self.workspace_dir = workspace_dir or tempfile.mkdtemp(prefix=f"tb_p1_{condition.value}_")
        self._harness = ConditionRestrictedHarness(
            condition=self.condition,
            workspace_dir=self.workspace_dir,
        )

    def get_harness(self) -> ConditionRestrictedHarness:
        return self._harness

    def setup_task(self, task: BenchmarkTask) -> None:
        """Inject initial task files into sandbox workspace."""
        for path, content in task.initial_files.items():
            self._harness.runtime.write_file(path, content)

    def verify_task_success(self, task: BenchmarkTask) -> bool:
        """Check whether required output files exist and are non-empty."""
        for exp in task.expected_output_files:
            if not self._harness.runtime.file_exists(exp):
                return False
            if len(self._harness.runtime.read_file(exp).strip()) == 0:
                return False
        return True
