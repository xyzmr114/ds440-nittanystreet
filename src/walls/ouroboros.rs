pub struct OuroborosWall {
    protected_patterns: Vec<&'static str>,
}

impl Default for OuroborosWall {
    fn default() -> Self {
        Self::new()
    }
}

impl OuroborosWall {
    pub fn new() -> Self {
        Self {
            protected_patterns: vec![
                "tests/",
                "src/taint/",
                "src/walls/",
                "migrations/",
                ".git/",
                "Cargo.toml",
            ],
        }
    }

    pub fn check_write(&self, target_path: &str, _content: &str) -> anyhow::Result<()> {
        let normalized = target_path.replace('\\', "/");
        for pattern in &self.protected_patterns {
            if normalized.contains(pattern) || normalized.starts_with(pattern) {
                return Err(anyhow::anyhow!(
                    "Ouroboros Wall Block: Modification of security-critical infrastructure '{}' is prohibited",
                    target_path
                ));
            }
        }
        Ok(())
    }
}
