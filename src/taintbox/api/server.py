"""Production REST API Server for TaintBox Sandbox Execution."""

from __future__ import annotations

import shutil
import uuid
from typing import Any, Dict, List, Optional
from fastapi import FastAPI, HTTPException, status
from pydantic import BaseModel, Field

from taintbox.aci.harness import ACIHarness
from taintbox.models import Observation, SnapshotMetadata, ToolResult


# --- Request & Response Models ---

class CreateSandboxRequest(BaseModel):
    description: str = ""
    timeout: int = 300
    policy_profile: str = "default_strict"


class CreateSandboxResponse(BaseModel):
    sandbox_id: str
    status: str
    description: str


class ToolReadRequest(BaseModel):
    path: str


class ToolWriteRequest(BaseModel):
    path: str
    content: str
    source_ids: Optional[List[str]] = None


class ToolFetchRequest(BaseModel):
    url: str
    save_as: Optional[str] = None
    mock_content: Optional[str] = None


class ToolExecRequest(BaseModel):
    program: str
    args: List[str] = Field(default_factory=list)
    env: Optional[Dict[str, str]] = None


class SnapshotRequest(BaseModel):
    description: str = ""


class RewindRequest(BaseModel):
    snapshot_id: str


# --- Session Manager ---

class SessionManager:
    """Manages active sandbox instances and isolates workspaces."""

    def __init__(self):
        self._sessions: Dict[str, ACIHarness] = {}
        self._metadata: Dict[str, Dict[str, Any]] = {}

    def create_session(self, description: str = "") -> str:
        sandbox_id = f"sbx_{uuid.uuid4().hex[:12]}"
        harness = ACIHarness()
        self._sessions[sandbox_id] = harness
        self._metadata[sandbox_id] = {
            "sandbox_id": sandbox_id,
            "description": description,
            "status": "active",
        }
        return sandbox_id

    def get_session(self, sandbox_id: str) -> ACIHarness:
        if sandbox_id not in self._sessions:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail=f"Sandbox '{sandbox_id}' not found",
            )
        return self._sessions[sandbox_id]

    def list_sessions(self) -> List[str]:
        return list(self._sessions.keys())

    def terminate_session(self, sandbox_id: str) -> None:
        harness = self.get_session(sandbox_id)
        # Clean workspace directory
        try:
            if hasattr(harness.runtime, "root_path") and harness.runtime.root_path.exists():
                shutil.rmtree(harness.runtime.root_path, ignore_errors=True)
        except Exception:
            pass
        del self._sessions[sandbox_id]
        if sandbox_id in self._metadata:
            del self._metadata[sandbox_id]

    def clear_all(self) -> None:
        for sid in list(self._sessions.keys()):
            self.terminate_session(sid)


session_manager = SessionManager()

# --- FastAPI Application ---

app = FastAPI(
    title="TaintBox API",
    version="0.1.0",
    description="Taint-Tracked Sandbox Runtime and ACI Gateway for AI Agents",
)


@app.get("/health")
def health_check():
    return {"status": "healthy", "version": "0.1.0"}


@app.post("/v1/sandboxes", response_model=CreateSandboxResponse, status_code=status.HTTP_201_CREATED)
def create_sandbox(req: CreateSandboxRequest):
    sid = session_manager.create_session(description=req.description)
    return CreateSandboxResponse(
        sandbox_id=sid,
        status="active",
        description=req.description,
    )


@app.get("/v1/sandboxes")
def list_sandboxes():
    return {"sandboxes": session_manager.list_sessions()}


@app.delete("/v1/sandboxes/{sandbox_id}")
def delete_sandbox(sandbox_id: str):
    session_manager.terminate_session(sandbox_id)
    return {"sandbox_id": sandbox_id, "status": "terminated"}


@app.post("/v1/sandboxes/{sandbox_id}/tools/read", response_model=ToolResult)
def tool_read(sandbox_id: str, req: ToolReadRequest):
    harness = session_manager.get_session(sandbox_id)
    return harness.read(req.path)


@app.post("/v1/sandboxes/{sandbox_id}/tools/write", response_model=ToolResult)
def tool_write(sandbox_id: str, req: ToolWriteRequest):
    harness = session_manager.get_session(sandbox_id)
    return harness.write(req.path, req.content, source_ids=req.source_ids)


@app.post("/v1/sandboxes/{sandbox_id}/tools/fetch", response_model=ToolResult)
def tool_fetch(sandbox_id: str, req: ToolFetchRequest):
    harness = session_manager.get_session(sandbox_id)
    return harness.fetch(req.url, save_as=req.save_as, mock_content=req.mock_content)


@app.post("/v1/sandboxes/{sandbox_id}/tools/exec", response_model=ToolResult)
def tool_exec(sandbox_id: str, req: ToolExecRequest):
    harness = session_manager.get_session(sandbox_id)
    return harness.exec(req.program, req.args, env=req.env)


@app.post("/v1/sandboxes/{sandbox_id}/snapshots", response_model=SnapshotMetadata, status_code=status.HTTP_201_CREATED)
def create_snapshot(sandbox_id: str, req: SnapshotRequest):
    harness = session_manager.get_session(sandbox_id)
    return harness.snapshot(req.description)


@app.post("/v1/sandboxes/{sandbox_id}/rewind")
def rewind_snapshot(sandbox_id: str, req: RewindRequest):
    harness = session_manager.get_session(sandbox_id)
    success = harness.rewind(req.snapshot_id)
    return {"sandbox_id": sandbox_id, "snapshot_id": req.snapshot_id, "success": success}


@app.get("/v1/sandboxes/{sandbox_id}/observe", response_model=Observation)
def observe_sandbox(sandbox_id: str):
    harness = session_manager.get_session(sandbox_id)
    return harness.observe()


@app.get("/v1/sandboxes/{sandbox_id}/telemetry")
def export_telemetry(sandbox_id: str):
    harness = session_manager.get_session(sandbox_id)
    events = harness.telemetry.get_events()
    return {"sandbox_id": sandbox_id, "count": len(events), "events": events}
