use std::collections::{HashMap, HashSet};
use crate::models::{PolicyDecision, ProvenanceRecord, ProvenanceTag, TrustLevel};

pub const DEFAULT_PRIVILEGED_ACTIONS: &[&str] = &[
    "network_egress",
    "exec_privileged",
    "file_delete",
    "send_email",
    "exfiltrate",
    "curl",
    "wget",
    "rm",
];

#[derive(Debug, Clone)]
pub struct TaintEngine {
    ledger: HashMap<String, ProvenanceRecord>,
    privileged_actions: HashSet<String>,
}

impl Default for TaintEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TaintEngine {
    pub fn new() -> Self {
        let mut privileged_actions = HashSet::new();
        for action in DEFAULT_PRIVILEGED_ACTIONS {
            privileged_actions.insert(action.to_string());
        }
        Self {
            ledger: HashMap::new(),
            privileged_actions,
        }
    }

    pub fn record_provenance(&mut self, resource_id: &str, record: ProvenanceRecord) {
        self.ledger.insert(resource_id.to_string(), record);
    }

    pub fn get_provenance(&self, resource_id: &str) -> Option<&ProvenanceRecord> {
        self.ledger.get(resource_id)
    }

    pub fn is_tainted(&self, resource_id: &str) -> bool {
        self.get_provenance(resource_id)
            .map(|r| r.trust_level.is_tainted())
            .unwrap_or(false)
    }

    pub fn propagate(
        &mut self,
        source_ids: &[String],
        target_id: &str,
        tag: Option<ProvenanceTag>,
    ) -> ProvenanceRecord {
        let mut highest_level = TrustLevel::Internal;
        let mut combined_custody = Vec::new();
        let mut found_sources = false;

        for sid in source_ids {
            if let Some(rec) = self.ledger.get(sid) {
                found_sources = true;
                if rec.trust_level > highest_level {
                    highest_level = rec.trust_level;
                }
                for c in &rec.chain_of_custody {
                    if !combined_custody.contains(c) {
                        combined_custody.push(c.clone());
                    }
                }
                if !combined_custody.contains(&rec.source_id) {
                    combined_custody.push(rec.source_id.clone());
                }
            } else if !combined_custody.contains(sid) {
                combined_custody.push(sid.clone());
            }
        }

        let effective_tag = tag.unwrap_or_else(|| {
            if found_sources {
                ProvenanceTag::Derived
            } else {
                ProvenanceTag::Internal
            }
        });

        let new_record = ProvenanceRecord {
            source_id: target_id.to_string(),
            tag: effective_tag,
            trust_level: highest_level,
            chain_of_custody: combined_custody,
            timestamp: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
            metadata: serde_json::json!({ "derived_from": source_ids }),
        };

        self.ledger.insert(target_id.to_string(), new_record.clone());
        new_record
    }

    pub fn evaluate_policy(&self, action: &str, resource_ids: &[String]) -> PolicyDecision {
        let mut tainted_records = Vec::new();
        for rid in resource_ids {
            if let Some(rec) = self.get_provenance(rid) {
                if rec.trust_level.is_tainted() {
                    tainted_records.push(rec.clone());
                }
            }
        }

        let is_privileged = self.privileged_actions.contains(action);

        if is_privileged && !tainted_records.is_empty() {
            let tainted_names: Vec<String> = tainted_records.iter().map(|r| r.source_id.clone()).collect();
            PolicyDecision {
                allowed: false,
                rule_id: Some("RULE-001".to_string()),
                action: action.to_string(),
                reason: format!(
                    "Action '{}' is privileged and blocked because referenced inputs contain untrusted data: {:?}",
                    action, tainted_names
                ),
                taint_records: tainted_records,
            }
        } else {
            PolicyDecision {
                allowed: true,
                rule_id: None,
                action: action.to_string(),
                reason: "Allowed by boundary policy".to_string(),
                taint_records: tainted_records,
            }
        }
    }

    pub fn list_tainted_resources(&self) -> Vec<String> {
        let mut list: Vec<String> = self
            .ledger
            .iter()
            .filter(|(_, rec)| rec.trust_level.is_tainted())
            .map(|(k, _)| k.clone())
            .collect();
        list.sort();
        list
    }

    pub fn export_state(&self) -> serde_json::Value {
        serde_json::to_value(&self.ledger).unwrap_or(serde_json::json!({}))
    }

    pub fn restore_state(&mut self, state: serde_json::Value) {
        if let Ok(ledger) = serde_json::from_value::<HashMap<String, ProvenanceRecord>>(state) {
            self.ledger = ledger;
        }
    }
}
