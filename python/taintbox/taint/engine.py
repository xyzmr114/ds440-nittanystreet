"""Taint Engine: Provenance Ledger, Propagation, and Boundary Policy Enforcer."""

from __future__ import annotations

import copy
import time
from typing import Any, Dict, List, Optional

from taintbox.models import (
    PolicyDecision,
    ProvenanceRecord,
    ProvenanceTag,
    TrustLevel,
)

# Trust ranking: higher number = more tainted / less trusted
TRUST_SEVERITY: Dict[TrustLevel, int] = {
    TrustLevel.TRUSTED: 0,
    TrustLevel.INTERNAL: 1,
    TrustLevel.UNTRUSTED: 2,
    TrustLevel.HOSTILE: 3,
}

PRIVILEGED_ACTIONS = {
    "network_egress",
    "exec_privileged",
    "file_delete",
    "send_email",
    "exfiltrate",
}


class TaintEngine:
    """Manages resource provenance and enforces boundary policies on tool invocations."""

    def __init__(self, policy_config: Optional[Dict[str, Any]] = None):
        self._ledger: Dict[str, ProvenanceRecord] = {}
        self._policy_config = policy_config or {}
        self._privileged_actions = set(
            self._policy_config.get("privileged_actions", PRIVILEGED_ACTIONS)
        )

    def record_provenance(self, resource_id: str, record: ProvenanceRecord) -> None:
        """Register or update the provenance record for a resource."""
        self._ledger[resource_id] = record

    def get_provenance(self, resource_id: str) -> Optional[ProvenanceRecord]:
        """Retrieve the provenance record for a given resource ID."""
        return self._ledger.get(resource_id)

    def is_tainted(self, resource_id: str) -> bool:
        """Return True if the resource exists and has UNTRUSTED or HOSTILE trust level."""
        record = self.get_provenance(resource_id)
        if not record:
            return False
        return TRUST_SEVERITY[record.trust_level] >= TRUST_SEVERITY[TrustLevel.UNTRUSTED]

    def propagate(
        self,
        source_ids: List[str],
        target_id: str,
        tag: ProvenanceTag = ProvenanceTag.DERIVED,
    ) -> ProvenanceRecord:
        """Propagate taint from source resources to a new target resource.
        
        The target inherits the highest taint severity and concatenates the custody chains.
        """
        records: List[ProvenanceRecord] = [
            self._ledger[sid] for sid in source_ids if sid in self._ledger
        ]

        if not records:
            # If no existing sources have records, default to INTERNAL
            new_record = ProvenanceRecord(
                source_id=target_id,
                tag=tag,
                trust_level=TrustLevel.INTERNAL,
                chain_of_custody=source_ids.copy(),
                timestamp=time.time(),
            )
        else:
            # Determine maximum taint severity among sources
            highest_record = max(records, key=lambda r: TRUST_SEVERITY[r.trust_level])
            combined_custody = []
            for r in records:
                for c in r.chain_of_custody:
                    if c not in combined_custody:
                        combined_custody.append(c)
                if r.source_id not in combined_custody:
                    combined_custody.append(r.source_id)

            new_record = ProvenanceRecord(
                source_id=target_id,
                tag=tag,
                trust_level=highest_record.trust_level,
                chain_of_custody=combined_custody,
                timestamp=time.time(),
                metadata={"derived_from": source_ids},
            )

        self._ledger[target_id] = new_record
        return new_record

    def evaluate_policy(self, action: str, resource_ids: List[str]) -> PolicyDecision:
        """Evaluate if an action on the specified resources violates boundary policies."""
        tainted_records: List[ProvenanceRecord] = []
        for rid in resource_ids:
            rec = self.get_provenance(rid)
            if rec and TRUST_SEVERITY[rec.trust_level] >= TRUST_SEVERITY[TrustLevel.UNTRUSTED]:
                tainted_records.append(rec)

        is_privileged = action in self._privileged_actions

        if is_privileged and tainted_records:
            return PolicyDecision(
                allowed=False,
                rule_id="RULE-001",
                action=action,
                reason=(
                    f"Action '{action}' is privileged and blocked because inputs contain "
                    f"untrusted data: {[r.source_id for r in tainted_records]}"
                ),
                taint_records=tainted_records,
            )

        return PolicyDecision(
            allowed=True,
            action=action,
            reason="Allowed by boundary policy",
            taint_records=tainted_records,
        )

    def list_tainted_resources(self) -> List[str]:
        """List all resource IDs currently classified as UNTRUSTED or HOSTILE."""
        return [
            rid
            for rid, record in self._ledger.items()
            if TRUST_SEVERITY[record.trust_level] >= TRUST_SEVERITY[TrustLevel.UNTRUSTED]
        ]

    def export_state(self) -> Dict[str, Any]:
        """Export ledger state for snapshotting."""
        return {k: v.model_dump() for k, v in self._ledger.items()}

    def restore_state(self, state: Dict[str, Any]) -> None:
        """Restore ledger state from snapshot."""
        self._ledger = {
            k: ProvenanceRecord.model_validate(v) for k, v in state.items()
        }
