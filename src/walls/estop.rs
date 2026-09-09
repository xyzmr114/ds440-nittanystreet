use crate::walls::promptinject::Severity;

pub struct EmergencyStop {
    tripped: bool,
    reason: Option<String>,
}

impl Default for EmergencyStop {
    fn default() -> Self {
        Self::new()
    }
}

impl EmergencyStop {
    pub fn new() -> Self {
        Self {
            tripped: false,
            reason: None,
        }
    }

    pub fn is_tripped(&self) -> bool {
        self.tripped
    }

    pub fn trip_reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn record_violation(&mut self, reason: &str, severity: Severity) {
        if severity == Severity::High {
            self.tripped = true;
            self.reason = Some(reason.to_string());
        }
    }

    pub fn reset(&mut self) {
        self.tripped = false;
        self.reason = None;
    }
}
