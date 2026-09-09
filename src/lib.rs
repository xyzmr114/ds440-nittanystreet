//! TaintBox: A Taint-Tracked Sandbox Runtime and Agent-Computer Interface for AI Agents.

pub mod aci;
pub mod api;
pub mod cli;
pub mod models;
pub mod runtime;
pub mod store;
pub mod taint;
pub mod walls;

pub use aci::ACIHarness;
pub use models::*;
pub use runtime::{LocalIsolatedRuntime, SandboxRuntime};
pub use store::{PostgresStore, SessionManager};
pub use taint::TaintEngine;
pub use walls::{EmergencyStop, HalluScan, OuroborosWall, PromptInjectScanner};
