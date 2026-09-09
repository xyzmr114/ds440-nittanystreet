use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

use crate::models::{
    AuditEvent, Observation, ProvenanceRecord, ProvenanceTag, SnapshotMetadata,
    ToolResult, TrustLevel,
};
use crate::runtime::{LocalIsolatedRuntime, SandboxRuntime};
use crate::taint::TaintEngine;

pub const NETWORK_PROGRAMS: &[&str] = &["curl", "wget", "nc", "ncat", "ssh", "scp", "ftp"];
pub const DELETE_PROGRAMS: &[&str] = &["rm", "del", "unlink", "shred"];

pub struct ACIHarness {
    pub runtime: Box<dyn SandboxRuntime>,
    pub taint_engine: TaintEngine,
    snapshots: HashMap<String, SnapshotMetadata>,
    step_counter: usize,
    audit_events: Vec<AuditEvent>,
}

impl ACIHarness {
    pub fn new_with_temp_dir() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let runtime = LocalIsolatedRuntime::new(temp_dir.path())?;
        Ok(Self {
            runtime: Box::new(runtime),
            taint_engine: TaintEngine::new(),
            snapshots: HashMap::new(),
            step_counter: 0,
            audit_events: Vec::new(),
        })
    }

    pub fn new_with_dir<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let runtime = LocalIsolatedRuntime::new(dir)?;
        Ok(Self {
            runtime: Box::new(runtime),
            taint_engine: TaintEngine::new(),
            snapshots: HashMap::new(),
            step_counter: 0,
            audit_events: Vec::new(),
        })
    }

    fn log_event(&mut self, event_type: &str, action: &str, details: serde_json::Value) {
        let event = AuditEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
            event_type: event_type.to_string(),
            action: action.to_string(),
            caller: "agent".to_string(),
            details,
        };
        self.audit_events.push(event);
    }

    pub fn read(&mut self, path: &str) -> ToolResult {
        let call_id = Uuid::new_v4().to_string();
        let decision = self.taint_engine.evaluate_path_policy("read", path, &[]);
        if !decision.allowed {
            self.log_event("POLICY_BLOCK", "read", serde_json::json!({ "path": path, "reason": decision.reason }));
            return ToolResult {
                call_id,
                tool_name: "read".to_string(),
                status: "BLOCKED_BY_POLICY".to_string(),
                output: serde_json::Value::Null,
                error: Some(format!("Policy violation: {}", decision.reason)),
                provenance: None,
                policy_decision: Some(decision),
            };
        }

        match self.runtime.read_file(path) {
            Ok(content) => {
                let rec = self
                    .taint_engine
                    .get_provenance(path)
                    .cloned()
                    .unwrap_or_else(|| {
                        let new_rec = ProvenanceRecord {
                            source_id: path.to_string(),
                            tag: ProvenanceTag::System,
                            trust_level: TrustLevel::Internal,
                            chain_of_custody: vec![],
                            timestamp: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
                            metadata: serde_json::json!({}),
                        };
                        self.taint_engine.record_provenance(path, new_rec.clone());
                        new_rec
                    });

                self.log_event("TOOL_READ", "read", serde_json::json!({ "path": path, "size": content.len() }));

                ToolResult {
                    call_id,
                    tool_name: "read".to_string(),
                    status: "SUCCESS".to_string(),
                    output: serde_json::Value::String(content),
                    error: None,
                    provenance: Some(rec),
                    policy_decision: None,
                }
            }
            Err(e) => ToolResult {
                call_id,
                tool_name: "read".to_string(),
                status: "ERROR".to_string(),
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
                provenance: None,
                policy_decision: None,
            },
        }
    }

    pub fn write(&mut self, path: &str, content: &str, source_ids: Option<Vec<String>>) -> ToolResult {
        let call_id = Uuid::new_v4().to_string();
        let sources = source_ids.as_deref().unwrap_or(&[]);
        let decision = self.taint_engine.evaluate_path_policy("write", path, sources);
        if !decision.allowed {
            self.log_event("POLICY_BLOCK", "write", serde_json::json!({ "path": path, "reason": decision.reason }));
            return ToolResult {
                call_id,
                tool_name: "write".to_string(),
                status: "BLOCKED_BY_POLICY".to_string(),
                output: serde_json::Value::Null,
                error: Some(format!("Policy violation: {}", decision.reason)),
                provenance: None,
                policy_decision: Some(decision),
            };
        }
        match self.runtime.write_file(path, content) {
            Ok(_) => {
                let rec = if let Some(sources) = &source_ids {
                    self.taint_engine.propagate(sources, path, None)
                } else {
                    let new_rec = ProvenanceRecord {
                        source_id: path.to_string(),
                        tag: ProvenanceTag::User,
                        trust_level: TrustLevel::Internal,
                        chain_of_custody: vec![],
                        timestamp: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
                        metadata: serde_json::json!({}),
                    };
                    self.taint_engine.record_provenance(path, new_rec.clone());
                    new_rec
                };

                self.log_event(
                    "TOOL_WRITE",
                    "write",
                    serde_json::json!({ "path": path, "size": content.len(), "sources": source_ids }),
                );

                ToolResult {
                    call_id,
                    tool_name: "write".to_string(),
                    status: "SUCCESS".to_string(),
                    output: serde_json::json!(format!("Successfully wrote {} bytes to {}", content.len(), path)),
                    error: None,
                    provenance: Some(rec),
                    policy_decision: None,
                }
            }
            Err(e) => ToolResult {
                call_id,
                tool_name: "write".to_string(),
                status: "ERROR".to_string(),
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
                provenance: None,
                policy_decision: None,
            },
        }
    }

    pub fn edit_block(
        &mut self,
        path: &str,
        target_content: &str,
        replacement_content: &str,
        source_ids: Option<Vec<String>>,
    ) -> ToolResult {
        let call_id = Uuid::new_v4().to_string();
        let sources = source_ids.as_deref().unwrap_or(&[]);
        let decision = self.taint_engine.evaluate_path_policy("write", path, sources);
        if !decision.allowed {
            self.log_event("POLICY_BLOCK", "edit_block", serde_json::json!({ "path": path, "reason": decision.reason }));
            return ToolResult {
                call_id,
                tool_name: "edit_block".to_string(),
                status: "BLOCKED_BY_POLICY".to_string(),
                output: serde_json::Value::Null,
                error: Some(format!("Policy violation: {}", decision.reason)),
                provenance: None,
                policy_decision: Some(decision),
            };
        }

        let current = match self.runtime.read_file(path) {
            Ok(c) => c,
            Err(e) => {
                return ToolResult {
                    call_id,
                    tool_name: "edit_block".to_string(),
                    status: "ERROR".to_string(),
                    output: serde_json::Value::Null,
                    error: Some(e.to_string()),
                    provenance: None,
                    policy_decision: None,
                };
            }
        };

        if !current.contains(target_content) {
            return ToolResult {
                call_id,
                tool_name: "edit_block".to_string(),
                status: "ERROR".to_string(),
                output: serde_json::Value::Null,
                error: Some(format!("Target content not found in file: {}", path)),
                provenance: None,
                policy_decision: None,
            };
        }

        let modified = current.replacen(target_content, replacement_content, 1);
        match self.runtime.write_file(path, &modified) {
            Ok(_) => {
                let rec = if let Some(sources) = &source_ids {
                    self.taint_engine.propagate(sources, path, None)
                } else {
                    let prev_rec = self.taint_engine.get_provenance(path).cloned();
                    prev_rec.unwrap_or_else(|| {
                        let new_rec = ProvenanceRecord {
                            source_id: path.to_string(),
                            tag: ProvenanceTag::User,
                            trust_level: TrustLevel::Internal,
                            chain_of_custody: vec![],
                            timestamp: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
                            metadata: serde_json::json!({}),
                        };
                        self.taint_engine.record_provenance(path, new_rec.clone());
                        new_rec
                    })
                };

                self.log_event("TOOL_EDIT_BLOCK", "edit_block", serde_json::json!({ "path": path }));

                ToolResult {
                    call_id,
                    tool_name: "edit_block".to_string(),
                    status: "SUCCESS".to_string(),
                    output: serde_json::json!(format!("Successfully replaced block in {}", path)),
                    error: None,
                    provenance: Some(rec),
                    policy_decision: None,
                }
            }
            Err(e) => ToolResult {
                call_id,
                tool_name: "edit_block".to_string(),
                status: "ERROR".to_string(),
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
                provenance: None,
                policy_decision: None,
            },
        }
    }

    pub fn fold_output(&self, text: &str, head_lines: usize, tail_lines: usize) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() <= head_lines + tail_lines {
            return text.to_string();
        }

        let head = &lines[..head_lines];
        let tail = &lines[lines.len() - tail_lines..];
        let truncated_count = lines.len() - head_lines - tail_lines;

        format!(
            "{}\n\n... [{} lines truncated to preserve model context] ...\n\n{}",
            head.join("\n"),
            truncated_count,
            tail.join("\n")
        )
    }

    pub fn view_lines(&mut self, path: &str, start_line: usize, end_line: usize) -> ToolResult {
        let call_id = Uuid::new_v4().to_string();
        match self.runtime.read_file(path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                if lines.is_empty() {
                    return ToolResult {
                        call_id,
                        tool_name: "view_lines".to_string(),
                        status: "SUCCESS".to_string(),
                        output: serde_json::Value::String(String::new()),
                        error: None,
                        provenance: self.taint_engine.get_provenance(path).cloned(),
                        policy_decision: None,
                    };
                }

                let start_idx = if start_line == 0 { 0 } else { start_line - 1 };
                let end_idx = std::cmp::min(end_line, lines.len());

                let mut formatted = String::new();
                for (idx, line) in lines.iter().enumerate().take(end_idx).skip(start_idx) {
                    formatted.push_str(&format!("{}: {}\n", idx + 1, line));
                }

                self.log_event("TOOL_VIEW_LINES", "view_lines", serde_json::json!({ "path": path, "start": start_line, "end": end_line }));

                ToolResult {
                    call_id,
                    tool_name: "view_lines".to_string(),
                    status: "SUCCESS".to_string(),
                    output: serde_json::Value::String(formatted),
                    error: None,
                    provenance: self.taint_engine.get_provenance(path).cloned(),
                    policy_decision: None,
                }
            }
            Err(e) => ToolResult {
                call_id,
                tool_name: "view_lines".to_string(),
                status: "ERROR".to_string(),
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
                provenance: None,
                policy_decision: None,
            },
        }
    }

    pub fn search_files(&mut self, pattern: &str) -> ToolResult {
        let call_id = Uuid::new_v4().to_string();
        let all_files = self.runtime.list_files();

        let clean_pattern = pattern.trim_start_matches('*').trim_end_matches('*');
        let matched: Vec<String> = all_files
            .into_iter()
            .filter(|f| {
                if pattern == "*" || pattern == "**/*" {
                    true
                } else if pattern.starts_with('*') {
                    f.ends_with(clean_pattern)
                } else {
                    f.contains(clean_pattern)
                }
            })
            .collect();

        self.log_event("TOOL_SEARCH_FILES", "search_files", serde_json::json!({ "pattern": pattern, "matches": matched.len() }));

        ToolResult {
            call_id,
            tool_name: "search_files".to_string(),
            status: "SUCCESS".to_string(),
            output: serde_json::to_value(&matched).unwrap_or(serde_json::json!([])),
            error: None,
            provenance: None,
            policy_decision: None,
        }
    }

    pub fn grep(&mut self, query: &str) -> ToolResult {
        let call_id = Uuid::new_v4().to_string();
        let all_files = self.runtime.list_files();
        let mut results = String::new();

        for file in all_files {
            if let Ok(content) = self.runtime.read_file(&file) {
                for (idx, line) in content.lines().enumerate() {
                    if line.contains(query) {
                        results.push_str(&format!("{}:{}: {}\n", file, idx + 1, line));
                    }
                }
            }
        }

        self.log_event("TOOL_GREP", "grep", serde_json::json!({ "query": query }));

        ToolResult {
            call_id,
            tool_name: "grep".to_string(),
            status: "SUCCESS".to_string(),
            output: serde_json::Value::String(results),
            error: None,
            provenance: None,
            policy_decision: None,
        }
    }

    pub fn fetch(&mut self, url: &str, save_as: Option<&str>, mock_content: Option<&str>) -> ToolResult {
        let call_id = Uuid::new_v4().to_string();
        let target_path = save_as.map(|s| s.to_string()).unwrap_or_else(|| {
            format!("downloads/{}.txt", Uuid::new_v4().simple())
        });
        let payload = mock_content.unwrap_or("Simulated untrusted content");

        if let Err(e) = self.runtime.write_file(&target_path, payload) {
            return ToolResult {
                call_id,
                tool_name: "fetch".to_string(),
                status: "ERROR".to_string(),
                output: serde_json::Value::Null,
                error: Some(e.to_string()),
                provenance: None,
                policy_decision: None,
            };
        }

        let record = ProvenanceRecord {
            source_id: target_path.clone(),
            tag: ProvenanceTag::UntrustedWeb,
            trust_level: TrustLevel::Untrusted,
            chain_of_custody: vec![url.to_string()],
            timestamp: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
            metadata: serde_json::json!({ "origin_url": url }),
        };
        self.taint_engine.record_provenance(&target_path, record.clone());

        self.log_event("TOOL_FETCH", "fetch", serde_json::json!({ "url": url, "saved_to": target_path }));

        ToolResult {
            call_id,
            tool_name: "fetch".to_string(),
            status: "SUCCESS".to_string(),
            output: serde_json::Value::String(payload.to_string()),
            error: None,
            provenance: Some(record),
            policy_decision: None,
        }
    }

    pub fn exec(&mut self, program: &str, args: &[String]) -> ToolResult {
        let call_id = Uuid::new_v4().to_string();

        let action = if NETWORK_PROGRAMS.iter().any(|&p| p.eq_ignore_ascii_case(program)) {
            "network_egress"
        } else if DELETE_PROGRAMS.iter().any(|&p| p.eq_ignore_ascii_case(program)) {
            "file_delete"
        } else {
            "exec"
        };

        // Scan args for referenced files
        let mut referenced_files = Vec::new();
        for arg in args {
            let clean = arg.trim_start_matches('@').trim();
            if self.runtime.file_exists(clean) {
                referenced_files.push(clean.to_string());
            }
        }

        if action == "network_egress" {
            let target_url = args
                .iter()
                .find(|a| a.starts_with("http://") || a.starts_with("https://") || a.contains("://"))
                .map(|s| s.as_str())
                .unwrap_or("https://unknown-egress");
            let net_decision =
                self.taint_engine.evaluate_network_egress(target_url, &referenced_files);
            if !net_decision.allowed {
                self.log_event(
                    "POLICY_BLOCK",
                    action,
                    serde_json::json!({
                        "program": program,
                        "args": args,
                        "reason": net_decision.reason
                    }),
                );
                return ToolResult {
                    call_id,
                    tool_name: "exec".to_string(),
                    status: "BLOCKED_BY_POLICY".to_string(),
                    output: serde_json::Value::Null,
                    error: Some(net_decision.reason.clone()),
                    provenance: None,
                    policy_decision: Some(net_decision),
                };
            }
        }

        let decision = self.taint_engine.evaluate_policy(action, &referenced_files);

        if !decision.allowed {
            self.log_event(
                "POLICY_BLOCK",
                action,
                serde_json::json!({
                    "program": program,
                    "args": args,
                    "reason": decision.reason
                }),
            );

            return ToolResult {
                call_id,
                tool_name: "exec".to_string(),
                status: "BLOCKED_BY_POLICY".to_string(),
                output: serde_json::Value::Null,
                error: Some(decision.reason.clone()),
                provenance: None,
                policy_decision: Some(decision),
            };
        }

        let (exit_code, stdout, stderr) = self.runtime.execute_command(program, args);

        self.log_event(
            "TOOL_EXEC",
            action,
            serde_json::json!({ "program": program, "args": args, "exit_code": exit_code }),
        );

        ToolResult {
            call_id,
            tool_name: "exec".to_string(),
            status: if exit_code == 0 { "SUCCESS".to_string() } else { "ERROR".to_string() },
            output: serde_json::json!({ "exit_code": exit_code, "stdout": stdout, "stderr": stderr }),
            error: if exit_code != 0 { Some(stderr) } else { None },
            provenance: None,
            policy_decision: Some(decision),
        }
    }

    pub fn snapshot(&mut self, description: &str) -> anyhow::Result<SnapshotMetadata> {
        let snap_id = format!("snap_{}", chrono::Utc::now().timestamp_millis());
        let file_hashes = self.runtime.create_snapshot(&snap_id)?;
        let taint_state = self.taint_engine.export_state();

        let meta = SnapshotMetadata {
            snapshot_id: snap_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
            description: description.to_string(),
            file_hashes,
            taint_ledger_state: taint_state,
        };

        self.snapshots.insert(snap_id.clone(), meta.clone());
        self.log_event("SNAPSHOT_CREATED", "snapshot", serde_json::json!({ "snapshot_id": snap_id, "description": description }));

        Ok(meta)
    }

    pub fn rewind(&mut self, snapshot_id: &str) -> anyhow::Result<()> {
        let meta = self
            .snapshots
            .get(snapshot_id)
            .ok_or_else(|| anyhow::anyhow!("Snapshot not found: {}", snapshot_id))?
            .clone();

        self.runtime.restore_snapshot(snapshot_id)?;
        self.taint_engine.restore_state(meta.taint_ledger_state);

        self.log_event("SNAPSHOT_REWOUND", "rewind", serde_json::json!({ "snapshot_id": snapshot_id }));
        Ok(())
    }

    pub fn observe(&mut self) -> Observation {
        self.step_counter += 1;
        let files = self.runtime.list_files();
        let tainted = self.taint_engine.list_tainted_resources();

        let mut prov_context = HashMap::new();
        for t in &tainted {
            if let Some(rec) = self.taint_engine.get_provenance(t) {
                prov_context.insert(
                    t.clone(),
                    serde_json::json!({
                        "tag": rec.tag,
                        "trust_level": rec.trust_level,
                        "chain_of_custody": rec.chain_of_custody,
                    }),
                );
            }
        }

        Observation {
            step_id: self.step_counter,
            modified_files: vec![],
            created_files: files,
            deleted_files: vec![],
            active_taint_count: tainted.len(),
            tainted_resources: tainted,
            provenance_context: prov_context,
        }
    }

    pub fn get_audit_events(&self) -> Vec<AuditEvent> {
        self.audit_events.clone()
    }
}
