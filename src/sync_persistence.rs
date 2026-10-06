use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::taskchampion::SyncSettings;

const SYNC_CONFIG_FILE: &str = "sync.toml";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct PersistedSyncSettings {
    server_url: String,
    client_id: String,
    encryption_secret: String,
}

impl From<&SyncSettings> for PersistedSyncSettings {
    fn from(settings: &SyncSettings) -> Self {
        Self {
            server_url: settings.server_url.clone(),
            client_id: settings.client_id.clone(),
            encryption_secret: settings.encryption_secret.clone(),
        }
    }
}

impl From<PersistedSyncSettings> for SyncSettings {
    fn from(settings: PersistedSyncSettings) -> Self {
        Self {
            server_url: settings.server_url,
            client_id: settings.client_id,
            encryption_secret: settings.encryption_secret,
            local_server_dir: None,
        }
    }
}

pub fn sync_config_path(data_dir: &Path) -> PathBuf {
    data_dir.join(SYNC_CONFIG_FILE)
}

pub fn load_sync_settings(data_dir: &Path) -> Result<Option<SyncSettings>> {
    let path = sync_config_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read saved sync config: {:?}", path))?;
    let saved: PersistedSyncSettings = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse saved sync config: {:?}", path))?;

    Ok(Some(saved.into()))
}

pub fn save_sync_settings(data_dir: &Path, settings: &SyncSettings) -> Result<()> {
    fs::create_dir_all(data_dir)
        .with_context(|| format!("Failed to create data directory: {:?}", data_dir))?;

    let path = sync_config_path(data_dir);
    let contents = toml::to_string_pretty(&PersistedSyncSettings::from(settings))
        .context("Failed to serialize sync credentials")?;

    fs::write(&path, contents)
        .with_context(|| format!("Failed to save sync credentials: {:?}", path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_file_returns_none() -> Result<()> {
        let dir = tempdir()?;
        assert!(load_sync_settings(dir.path())?.is_none());
        Ok(())
    }

    #[test]
    fn credentials_round_trip() -> Result<()> {
        let dir = tempdir()?;
        let settings = SyncSettings {
            server_url: "https://tasks.example.test".to_string(),
            client_id: "01234567-89ab-cdef-0123-456789abcdef".to_string(),
            encryption_secret: "test-secret".to_string(),
            local_server_dir: None,
        };

        save_sync_settings(dir.path(), &settings)?;
        let loaded = load_sync_settings(dir.path())?.expect("saved settings should load");

        assert_eq!(loaded.server_url, settings.server_url);
        assert_eq!(loaded.client_id, settings.client_id);
        assert_eq!(loaded.encryption_secret, settings.encryption_secret);
        assert!(loaded.local_server_dir.is_none());
        Ok(())
    }
}
