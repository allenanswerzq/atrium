//! Agent discovery — find running agent processes on the local machine.
//!
//! Scans the system process table for known agent binaries (copilot, claude,
//! codex, etc.), captures their PID, command line, working directory, and
//! infers capabilities from arguments.

use std::path::PathBuf;

use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};

use crate::kind::AgentKind;

/// A running agent process discovered on the local machine.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredAgent {
    /// Which agent this is.
    pub kind: AgentKind,
    /// OS process ID.
    pub pid: u32,
    /// Process name (e.g. `copilot.exe`).
    pub name: String,
    /// Full command line, if available.
    pub cmd: Vec<String>,
    /// Working directory of the process, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Whether this instance is running in ACP mode (e.g. `--acp` flag).
    pub is_acp: bool,
}

/// Known agent binary names (without extension) → AgentKind mapping.
fn agent_binary_map() -> &'static [(&'static str, AgentKind)] {
    &[
        ("copilot", AgentKind::Copilot),
        ("claude", AgentKind::Claude),
        ("codex", AgentKind::Codex),
        ("cursor", AgentKind::Cursor),
        ("gemini", AgentKind::Gemini),
        ("opencode", AgentKind::OpenCode),
        ("pi", AgentKind::Pi),
    ]
}

/// Discover all running agent processes on the local machine.
///
/// Scans the process table for known agent binary names and returns
/// information about each running instance.
pub fn discover_agents() -> Vec<DiscoveredAgent> {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(
            ProcessRefreshKind::nothing()
                .with_cmd(UpdateKind::Always)
                .with_cwd(UpdateKind::Always)
                .with_exe(UpdateKind::Always),
        ),
    );

    let map = agent_binary_map();
    let mut discovered = Vec::new();

    for (pid, process) in sys.processes() {
        let exe_name = process
            .exe()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_lowercase();

        let Some(&(_, kind)) = map.iter().find(|&&(name, _)| exe_name == name) else {
            continue;
        };

        let cmd: Vec<String> = process
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        let is_acp = cmd.iter().any(|arg| arg == "--acp");
        let cwd = process.cwd().map(|p| p.to_path_buf());

        discovered.push(DiscoveredAgent {
            kind,
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().into_owned(),
            cmd,
            cwd,
            is_acp,
        });
    }

    tracing::info!(count = discovered.len(), "running agents discovered");
    discovered
}

/// Discover running processes for a specific agent kind.
pub fn discover_agent(kind: AgentKind) -> Vec<DiscoveredAgent> {
    discover_agents()
        .into_iter()
        .filter(|a| a.kind == kind)
        .collect()
}

/// Check if a specific agent binary is installed on PATH (does not check running).
pub fn is_installed(kind: AgentKind) -> Option<PathBuf> {
    let map = agent_binary_map();
    let (binary, _) = map.iter().find(|&&(_, k)| k == kind)?;
    which::which(binary).ok()
}
