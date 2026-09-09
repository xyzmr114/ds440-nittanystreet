use std::collections::HashSet;

pub struct HalluScan {
    known_files: HashSet<String>,
}

impl HalluScan {
    pub fn new(files: &[String]) -> Self {
        let set = files.iter().map(|f| f.replace('\\', "/")).collect();
        Self { known_files: set }
    }

    pub fn validate_path(&self, path: &str) -> anyhow::Result<()> {
        let normalized = path.replace('\\', "/");
        if !self.known_files.contains(&normalized) {
            return Err(anyhow::anyhow!(
                "HalluScan Warning: Hallucinated path '{}' does not exist in workspace snapshot",
                path
            ));
        }
        Ok(())
    }
}
