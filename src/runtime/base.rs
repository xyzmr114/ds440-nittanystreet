use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub trait SandboxRuntime: Send + Sync {
    fn read_file(&self, rel_path: &str) -> anyhow::Result<String>;
    fn write_file(&self, rel_path: &str, content: &str) -> anyhow::Result<()>;
    fn file_exists(&self, rel_path: &str) -> bool;
    fn list_files(&self) -> Vec<String>;
    fn execute_command(&self, program: &str, args: &[String]) -> (i32, String, String);
    fn create_snapshot(&self, snapshot_id: &str) -> anyhow::Result<HashMap<String, String>>;
    fn restore_snapshot(&self, snapshot_id: &str) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct LocalIsolatedRuntime {
    root_path: PathBuf,
    snapshots_dir: PathBuf,
}

impl LocalIsolatedRuntime {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> anyhow::Result<Self> {
        let root = root_dir.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        let snapshots = root.join(".taintbox_snapshots");
        fs::create_dir_all(&snapshots)?;
        Ok(Self {
            root_path: root,
            snapshots_dir: snapshots,
        })
    }

    fn resolve_path(&self, rel_path: &str) -> anyhow::Result<PathBuf> {
        let cleaned = Path::new(rel_path);
        if cleaned.is_absolute() {
            return Err(anyhow::anyhow!("Absolute paths not allowed in sandbox: {}", rel_path));
        }
        let resolved = self.root_path.join(cleaned);
        // Ensure path stays within root
        if !resolved.starts_with(&self.root_path) {
            return Err(anyhow::anyhow!("Path escape detected: {}", rel_path));
        }
        Ok(resolved)
    }

    fn collect_files_recursive(&self, dir: &Path, rel_prefix: &str, files: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path == self.snapshots_dir {
                    continue;
                }
                let file_name = entry.file_name().to_string_lossy().to_string();
                let rel = if rel_prefix.is_empty() {
                    file_name
                } else {
                    format!("{}/{}", rel_prefix, file_name)
                };

                if path.is_dir() {
                    self.collect_files_recursive(&path, &rel, files);
                } else if path.is_file() {
                    files.push(rel.replace('\\', "/"));
                }
            }
        }
    }
}

impl SandboxRuntime for LocalIsolatedRuntime {
    fn read_file(&self, rel_path: &str) -> anyhow::Result<String> {
        let path = self.resolve_path(rel_path)?;
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", rel_path));
        }
        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    fn write_file(&self, rel_path: &str, content: &str) -> anyhow::Result<()> {
        let path = self.resolve_path(rel_path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn file_exists(&self, rel_path: &str) -> bool {
        self.resolve_path(rel_path).map(|p| p.exists()).unwrap_or(false)
    }

    fn list_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        self.collect_files_recursive(&self.root_path, "", &mut files);
        files.sort();
        files
    }

    fn execute_command(&self, program: &str, args: &[String]) -> (i32, String, String) {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.current_dir(&self.root_path);

        match cmd.output() {
            Ok(output) => {
                let code = output.status.code().unwrap_or(1);
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                (code, stdout, stderr)
            }
            Err(e) => (1, String::new(), e.to_string()),
        }
    }

    fn create_snapshot(&self, snapshot_id: &str) -> anyhow::Result<HashMap<String, String>> {
        let target_dir = self.snapshots_dir.join(snapshot_id);
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
        }
        fs::create_dir_all(&target_dir)?;

        let mut hashes = HashMap::new();
        for rel in self.list_files() {
            let src = self.resolve_path(&rel)?;
            let bytes = fs::read(&src)?;
            let hash = hex::encode(Sha256::digest(&bytes));
            hashes.insert(rel.clone(), hash);

            let dest = target_dir.join(&rel);
            if let Some(p) = dest.parent() {
                fs::create_dir_all(p)?;
            }
            fs::copy(&src, &dest)?;
        }

        Ok(hashes)
    }

    fn restore_snapshot(&self, snapshot_id: &str) -> anyhow::Result<()> {
        let source_dir = self.snapshots_dir.join(snapshot_id);
        if !source_dir.exists() {
            return Err(anyhow::anyhow!("Snapshot not found: {}", snapshot_id));
        }

        // Clean current files (except snapshots directory)
        for entry in fs::read_dir(&self.root_path)?.flatten() {
            let path = entry.path();
            if path == self.snapshots_dir {
                continue;
            }
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }

        // Restore snapshot files
        let mut files = Vec::new();
        self.collect_files_recursive(&source_dir, "", &mut files);
        for rel in files {
            let src = source_dir.join(&rel);
            let dest = self.resolve_path(&rel)?;
            if let Some(p) = dest.parent() {
                fs::create_dir_all(p)?;
            }
            fs::copy(src, dest)?;
        }

        Ok(())
    }
}
