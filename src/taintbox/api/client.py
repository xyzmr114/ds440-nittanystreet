"""Python Client SDK for TaintBox REST API."""

from __future__ import annotations

from typing import Any, Dict, List, Optional
import httpx

from taintbox.models import Observation, SnapshotMetadata, ToolResult


class TaintBoxClient:
    """Synchronous Python client SDK for communicating with TaintBox server."""

    def __init__(self, base_url: str = "http://localhost:8000", client: Optional[httpx.Client] = None):
        self.base_url = base_url.rstrip("/")
        self._http = client or httpx.Client(base_url=self.base_url)

    def create_sandbox(self, description: str = "") -> str:
        res = self._http.post("/v1/sandboxes", json={"description": description})
        res.raise_for_status()
        return res.json()["sandbox_id"]

    def list_sandboxes(self) -> List[str]:
        res = self._http.get("/v1/sandboxes")
        res.raise_for_status()
        return res.json()["sandboxes"]

    def delete_sandbox(self, sandbox_id: str) -> None:
        res = self._http.delete(f"/v1/sandboxes/{sandbox_id}")
        res.raise_for_status()

    def read(self, sandbox_id: str, path: str) -> ToolResult:
        res = self._http.post(f"/v1/sandboxes/{sandbox_id}/tools/read", json={"path": path})
        res.raise_for_status()
        return ToolResult.model_validate(res.json())

    def write(
        self,
        sandbox_id: str,
        path: str,
        content: str,
        source_ids: Optional[List[str]] = None,
    ) -> ToolResult:
        res = self._http.post(
            f"/v1/sandboxes/{sandbox_id}/tools/write",
            json={"path": path, "content": content, "source_ids": source_ids},
        )
        res.raise_for_status()
        return ToolResult.model_validate(res.json())

    def fetch(
        self,
        sandbox_id: str,
        url: str,
        save_as: Optional[str] = None,
        mock_content: Optional[str] = None,
    ) -> ToolResult:
        res = self._http.post(
            f"/v1/sandboxes/{sandbox_id}/tools/fetch",
            json={"url": url, "save_as": save_as, "mock_content": mock_content},
        )
        res.raise_for_status()
        return ToolResult.model_validate(res.json())

    def exec(
        self,
        sandbox_id: str,
        program: str,
        args: Optional[List[str]] = None,
        env: Optional[Dict[str, str]] = None,
    ) -> ToolResult:
        res = self._http.post(
            f"/v1/sandboxes/{sandbox_id}/tools/exec",
            json={"program": program, "args": args or [], "env": env},
        )
        res.raise_for_status()
        return ToolResult.model_validate(res.json())

    def snapshot(self, sandbox_id: str, description: str = "") -> SnapshotMetadata:
        res = self._http.post(
            f"/v1/sandboxes/{sandbox_id}/snapshots",
            json={"description": description},
        )
        res.raise_for_status()
        return SnapshotMetadata.model_validate(res.json())

    def rewind(self, sandbox_id: str, snapshot_id: str) -> bool:
        res = self._http.post(
            f"/v1/sandboxes/{sandbox_id}/rewind",
            json={"snapshot_id": snapshot_id},
        )
        res.raise_for_status()
        return res.json()["success"]

    def observe(self, sandbox_id: str) -> Observation:
        res = self._http.get(f"/v1/sandboxes/{sandbox_id}/observe")
        res.raise_for_status()
        return Observation.model_validate(res.json())
