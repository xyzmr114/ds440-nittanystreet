"""Abstract and local implementation of SandboxRuntime."""

from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class SandboxRuntime(ABC):
    """Abstract interface for sandbox execution backends."""

    @abstractmethod
    def read_file(self, rel_path: str) -> str:
        """Read text from a relative path within the sandbox."""

    @abstractmethod
    def write_file(self, rel_path: str, content: str) -> None:
        """Write text to a relative path within the sandbox."""

    @abstractmethod
    def file_exists(self, rel_path: str) -> bool:
        """Check if a file exists in the sandbox."""

    @abstractmethod
    def list_files(self) -> List[str]:
        """List relative paths of all files in the sandbox."""

    @abstractmethod
    def execute_command(
        self,
        program: str,
        args: List[str],
        env: Optional[Dict[str, str]] = None,
        timeout: int = 30,
    ) -> Tuple[int, str, str]:
        """Execute a program in the sandbox, returning (exit_code, stdout, stderr)."""

    @abstractmethod
    def create_snapshot(self, snapshot_id: str) -> Dict[str, str]:
        """Snapshot current workspace state, returning map of rel_path to content hash."""

    @abstractmethod
    def restore_snapshot(self, snapshot_id: str) -> None:
        """Restore workspace state to the given snapshot."""


class LocalIsolatedRuntime(SandboxRuntime):
    """Local filesystem sandbox with snapshot and rewind capabilities."""

    def __init__(self, root_dir: str):
        self.root_path = Path(root_dir).resolve()
        self.root_path.mkdir(parents=True, exist_ok=True)
        self.snapshots_dir = self.root_path / ".taintbox_snapshots"
        self.snapshots_dir.mkdir(parents=True, exist_ok=True)

    def _resolve(self, rel_path: str) -> Path:
        resolved = (self.root_path / rel_path).resolve()
        if not str(resolved).startswith(str(self.root_path)):
            raise PermissionError(f"Access denied: path '{rel_path}' escapes sandbox root.")
        return resolved

    def read_file(self, rel_path: str) -> str:
        path = self._resolve(rel_path)
        if not path.exists():
            raise FileNotFoundError(f"File '{rel_path}' not found in sandbox.")
        return path.read_text(encoding="utf-8")

    def write_file(self, rel_path: str, content: str) -> None:
        path = self._resolve(rel_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def file_exists(self, rel_path: str) -> bool:
        return self._resolve(rel_path).exists()

    def list_files(self) -> List[str]:
        files = []
        for p in self.root_path.rglob("*"):
            if p.is_file() and not str(p).startswith(str(self.snapshots_dir)):
                files.append(str(p.relative_to(self.root_path)).replace("\\", "/"))
        return sorted(files)

    def execute_command(
        self,
        program: str,
        args: List[str],
        env: Optional[Dict[str, str]] = None,
        timeout: int = 30,
    ) -> Tuple[int, str, str]:
        cmd = [program] + args
        cmd_env = os.environ.copy()
        if env:
            cmd_env.update(env)

        try:
            res = subprocess.run(
                cmd,
                cwd=str(self.root_path),
                env=cmd_env,
                capture_output=True,
                text=True,
                timeout=timeout,
                shell=False,
            )
            return res.returncode, res.stdout, res.stderr
        except subprocess.TimeoutExpired:
            return 124, "", f"Command '{program}' timed out after {timeout} seconds."
        except Exception as e:
            return 1, "", str(e)

    def create_snapshot(self, snapshot_id: str) -> Dict[str, str]:
        target_dir = self.snapshots_dir / snapshot_id
        if target_dir.exists():
            shutil.rmtree(target_dir)
        target_dir.mkdir(parents=True, exist_ok=True)

        hashes: Dict[str, str] = {}
        for rel_file in self.list_files():
            src_file = self._resolve(rel_file)
            content = src_file.read_bytes()
            hashes[rel_file] = hashlib.sha256(content).hexdigest()

            dest_file = target_dir / rel_file
            dest_file.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src_file, dest_file)

        return hashes

    def restore_snapshot(self, snapshot_id: str) -> None:
        source_dir = self.snapshots_dir / snapshot_id
        if not source_dir.exists():
            raise ValueError(f"Snapshot '{snapshot_id}' does not exist.")

        # Clean current workspace (excluding snapshot store)
        for p in self.root_path.iterdir():
            if p == self.snapshots_dir:
                continue
            if p.is_dir():
                shutil.rmtree(p)
            else:
                p.unlink()

        # Restore from snapshot
        for src in source_dir.rglob("*"):
            if src.is_file():
                rel = src.relative_to(source_dir)
                dest = self.root_path / rel
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src, dest)
