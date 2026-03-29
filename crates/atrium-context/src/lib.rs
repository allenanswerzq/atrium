//! # atrium-context
//!
//! Global environment context for Atrium applications.
//!
//! Provides a cheap-to-clone [`GlobalCtxt`] (Arc-wrapped) holding resolved
//! platform info, config paths, and environment variables. Built via
//! [`GlobalCtxtBuilder`] following the pilot-context pattern.

pub mod env_var;

use std::{path::PathBuf, sync::Arc};

use atrium_error::{Error, ErrorKind};

/// Shared application context — cheap to clone (Arc).
#[derive(Debug, Clone)]
pub struct GlobalCtxt(Arc<Inner>);

#[derive(Debug)]
struct Inner {
    /// User's home directory.
    home_dir: PathBuf,
    /// Atrium config directory (`~/.config/atrium/`).
    config_dir: PathBuf,
    /// Atrium data directory (`~/.atrium/`).
    data_dir: PathBuf,
    /// Hostname.
    hostname: String,
    /// Custom key-value data.
    custom: std::collections::HashMap<String, String>,
}

impl GlobalCtxt {
    /// Build a new context.
    pub fn builder() -> GlobalCtxtBuilder {
        GlobalCtxtBuilder::default()
    }

    /// User's home directory.
    pub fn home_dir(&self) -> &PathBuf {
        &self.0.home_dir
    }

    /// Atrium config directory.
    pub fn config_dir(&self) -> &PathBuf {
        &self.0.config_dir
    }

    /// Atrium data directory.
    pub fn data_dir(&self) -> &PathBuf {
        &self.0.data_dir
    }

    /// Machine hostname.
    pub fn hostname(&self) -> &str {
        &self.0.hostname
    }

    /// Resolve a path relative to the config directory.
    pub fn config_path(&self, relative: impl AsRef<std::path::Path>) -> PathBuf {
        self.0.config_dir.join(relative)
    }

    /// Resolve a path relative to the data directory.
    pub fn data_path(&self, relative: impl AsRef<std::path::Path>) -> PathBuf {
        self.0.data_dir.join(relative)
    }

    /// Get a custom key-value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.custom.get(key).map(|s| s.as_str())
    }
}

/// Builder for [`GlobalCtxt`].
#[derive(Debug, Default)]
pub struct GlobalCtxtBuilder {
    config_dir_override: Option<PathBuf>,
    data_dir_override: Option<PathBuf>,
    custom: std::collections::HashMap<String, String>,
}

impl GlobalCtxtBuilder {
    /// Override the config directory (default: `~/.config/atrium/`).
    pub fn config_dir(mut self, path: PathBuf) -> Self {
        self.config_dir_override = Some(path);
        self
    }

    /// Override the data directory (default: `~/.atrium/`).
    pub fn data_dir(mut self, path: PathBuf) -> Self {
        self.data_dir_override = Some(path);
        self
    }

    /// Add a custom key-value pair.
    pub fn custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }

    /// Build the context. Fails if the home directory cannot be resolved.
    pub fn build(self) -> Result<GlobalCtxt, Error> {
        let home_dir = atrium_fs::home_dir().ok_or_else(|| {
            Error::new(ErrorKind::ConfigInvalid, "home directory not found")
                .with_operation("GlobalCtxt::build")
        })?;

        let config_dir = self
            .config_dir_override
            .unwrap_or_else(|| home_dir.join(".config").join("atrium"));

        let data_dir = self
            .data_dir_override
            .unwrap_or_else(|| home_dir.join(".atrium"));

        let hostname = hostname_or_unknown();

        Ok(GlobalCtxt(Arc::new(Inner {
            home_dir,
            config_dir,
            data_dir,
            hostname,
            custom: self.custom,
        })))
    }
}

fn hostname_or_unknown() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_owned())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn build_context_with_defaults() {
        let ctx = GlobalCtxt::builder().build().unwrap();
        assert!(!ctx.hostname().is_empty());
        assert!(ctx.home_dir().exists());
    }

    #[test]
    fn build_context_with_overrides() {
        let ctx = GlobalCtxt::builder()
            .config_dir(PathBuf::from("/tmp/atrium-test-config"))
            .data_dir(PathBuf::from("/tmp/atrium-test-data"))
            .custom("version", "0.1.0")
            .build()
            .unwrap();

        assert_eq!(ctx.config_dir(), &PathBuf::from("/tmp/atrium-test-config"));
        assert_eq!(ctx.data_dir(), &PathBuf::from("/tmp/atrium-test-data"));
        assert_eq!(ctx.get("version"), Some("0.1.0"));
    }

    #[test]
    fn context_is_cheap_to_clone() {
        let ctx = GlobalCtxt::builder().build().unwrap();
        let clone = ctx.clone();
        assert_eq!(ctx.hostname(), clone.hostname());
    }
}
