//! Repository-local configuration (`atrium.toml`).
//!
//! Defines the schema for per-repo configuration including presets,
//! managed processes, tasks, branch naming rules, and notifications.

use atrium_error::{Error, ErrorKind, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE_NAME: &str = "atrium.toml";

/// Top-level repo configuration from `atrium.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RepoConfig {
    #[serde(rename = "presets", default)]
    pub presets: Vec<PresetConfig>,
    #[serde(rename = "processes", default)]
    pub processes: Vec<ProcessConfig>,
    pub scripts: Option<ScriptsConfig>,
    #[serde(rename = "tasks", default)]
    pub tasks: Vec<TaskConfig>,
    pub branch: Option<BranchConfig>,
    pub agent: Option<AgentConfig>,
    pub notifications: Option<NotificationsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresetConfig {
    pub name: String,
    pub icon: Option<String>,
    pub command: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessConfig {
    pub name: String,
    pub command: String,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub auto_restart: bool,
    pub restart_delay_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptsConfig {
    #[serde(default)]
    pub setup: Vec<String>,
    #[serde(default)]
    pub teardown: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskConfig {
    pub name: String,
    pub schedule: Option<String>,
    pub command: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub trigger: Option<TaskTriggerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskTriggerConfig {
    pub on_exit_code: Option<i32>,
    #[serde(default)]
    pub on_stdout: bool,
    pub agent: Option<String>,
    pub prompt_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BranchConfig {
    pub prefix_mode: Option<BranchPrefixMode>,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, strum::Display, strum::IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "snake_case")]
pub enum BranchPrefixMode {
    None,
    GitAuthor,
    GithubUser,
    Custom,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub default_preset: Option<String>,
    #[serde(default)]
    pub auto_checkpoint: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationsConfig {
    #[serde(default)]
    pub desktop: bool,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub webhook_urls: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Constructs the path to the config file in a repo root.
pub fn config_path(repo_root: &Path) -> PathBuf {
    repo_root.join(CONFIG_FILE_NAME)
}

/// Reads and parses `atrium.toml` from a repo root. Returns `None` if not found.
pub fn read_config(repo_root: &Path) -> Result<Option<RepoConfig>> {
    let path = config_path(repo_root);
    if !path.exists() {
        return Ok(None);
    }

    let content = atrium_fs::read_to_string(&path)?;
    let config: RepoConfig = toml::from_str(&content).map_err(|e| {
        Error::new(ErrorKind::ConfigInvalid, "failed to parse config")
            .with_operation("read_config")
            .with_context("path", path.display().to_string())
            .set_source(e)
    })?;

    Ok(Some(config))
}
