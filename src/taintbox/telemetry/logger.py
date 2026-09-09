"""Structured Telemetry Logger and Event Sink."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any, Dict, List, Optional


class TelemetryLogger:
    """Structured telemetry event sink for audit compliance and empirical research."""

    def __init__(self, log_dir: Optional[str] = None):
        self.events: List[Dict[str, Any]] = []
        self.log_file: Optional[Path] = None
        if log_dir:
            dir_path = Path(log_dir).resolve()
            dir_path.mkdir(parents=True, exist_ok=True)
            self.log_file = dir_path / f"telemetry_{int(time.time())}.jsonl"

    def log_event(
        self,
        event_type: str,
        details: Dict[str, Any],
        caller: str = "agent",
    ) -> Dict[str, Any]:
        """Record an execution or policy event."""
        event = {
            "timestamp": time.time(),
            "event_type": event_type,
            "caller": caller,
            "details": details,
        }
        self.events.append(event)

        if self.log_file:
            with open(self.log_file, "a", encoding="utf-8") as f:
                f.write(json.dumps(event) + "\n")

        return event

    def get_events(self, event_type: Optional[str] = None) -> List[Dict[str, Any]]:
        """Filter and retrieve logged events."""
        if event_type:
            return [e for e in self.events if e["event_type"] == event_type]
        return self.events.copy()

    def clear(self) -> None:
        """Clear in-memory events."""
        self.events.clear()
