use std::path::{Path, PathBuf};

use zeroize::Zeroize;

use crate::core::model::{Entry, EntryInput};
use crate::core::vault::Vault;
use crate::storage::local::{LocalVaultStore, StorageError};

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("vault is locked")]
    Locked,
    #[error("vault path is not set")]
    PathNotSet,
    #[error(transparent)]
    Storage(#[from] StorageError),
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultSnapshot {
    pub path: String,
    pub name: String,
    pub entry_count: usize,
    pub group_count: usize,
    pub sync_status: String,
}

pub struct VaultSession {
    path: Option<PathBuf>,
    password: Vec<u8>,
    vault: Option<Vault>,
}

impl VaultSession {
    pub fn new() -> Self {
        Self {
            path: None,
            password: Vec::new(),
            vault: None,
        }
    }

    pub fn create(&mut self, path: impl AsRef<Path>, password: &[u8]) -> Result<(), SessionError> {
        let path = path.as_ref().to_path_buf();
        let vault = Vault::new();
        LocalVaultStore::new(&path).save(&vault, password)?;
        self.replace_unlocked(path, password, vault);
        Ok(())
    }

    pub fn unlock(&mut self, path: impl AsRef<Path>, password: &[u8]) -> Result<(), SessionError> {
        let path = path.as_ref().to_path_buf();
        let vault = LocalVaultStore::new(&path).load(password)?;
        self.replace_unlocked(path, password, vault);
        Ok(())
    }

    pub fn lock(&mut self) {
        self.password.zeroize();
        self.password.clear();
        self.vault = None;
        self.path = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.vault.is_some()
    }

    pub fn current_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn snapshot(&self) -> Result<VaultSnapshot, SessionError> {
        let vault = self.vault.as_ref().ok_or(SessionError::Locked)?;
        let path = self.path.as_ref().ok_or(SessionError::PathNotSet)?;
        Ok(VaultSnapshot {
            path: path.display().to_string(),
            name: path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Vault")
                .to_string(),
            entry_count: vault.entries.len(),
            group_count: vault.groups.len(),
            sync_status: "offline".into(),
        })
    }

    pub fn with_vault<T>(&self, operation: impl FnOnce(&Vault) -> T) -> Result<T, SessionError> {
        self.vault
            .as_ref()
            .map(operation)
            .ok_or(SessionError::Locked)
    }

    pub fn data(&self) -> Result<Vault, SessionError> {
        self.vault.clone().ok_or(SessionError::Locked)
    }

    pub fn save(&self) -> Result<(), SessionError> {
        let path = self.path.as_ref().ok_or(SessionError::PathNotSet)?;
        let vault = self.vault.as_ref().ok_or(SessionError::Locked)?;
        LocalVaultStore::new(path).save(vault, &self.password)?;
        Ok(())
    }

    pub fn create_entry(&mut self, input: EntryInput) -> Result<Entry, SessionError> {
        let entry_id = self
            .vault
            .as_mut()
            .ok_or(SessionError::Locked)?
            .create_entry(input)
            .map_err(|error| StorageError::Io(std::io::Error::other(error.to_string())))?;
        self.save()?;
        self.vault
            .as_ref()
            .and_then(|vault| vault.entries.iter().find(|entry| entry.id == entry_id))
            .cloned()
            .ok_or(SessionError::Locked)
    }

    fn replace_unlocked(&mut self, path: PathBuf, password: &[u8], vault: Vault) {
        self.password.zeroize();
        self.password = password.to_vec();
        self.path = Some(path);
        self.vault = Some(vault);
    }
}

impl Default for VaultSession {
    fn default() -> Self {
        Self::new()
    }
}
