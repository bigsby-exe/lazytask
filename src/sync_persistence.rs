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
