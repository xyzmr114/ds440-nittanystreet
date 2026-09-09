"""Sandbox Runtime abstraction layer."""

from taintbox.runtime.base import LocalIsolatedRuntime, SandboxRuntime

__all__ = ["SandboxRuntime", "LocalIsolatedRuntime"]
