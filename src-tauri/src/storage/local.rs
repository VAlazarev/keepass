use std::fs;
use std::path::{Path, PathBuf};

use crate::core::crypto::{decrypt_container, encrypt_container, CryptoError};
use crate::core::vault::Vault;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("storage I/O failed")]
    Io(#[from] std::io::Error),
    #[error("vault serialization failed")]
    Serialization(#[from] serde_json::Error),
    #[error("vault encryption failed")]
    Crypto(#[from] CryptoError),
}

pub struct LocalVaultStore {
    path: PathBuf,
}

impl LocalVaultStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn save(&self, vault: &Vault, password: &[u8]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(vault)?;
        let encrypted = encrypt_container(password, &payload)?;
        let temp_path = self.path.with_extension("kdv.tmp");
        let backup_path = self.path.with_extension("kdv.bak");

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&temp_path, encrypted)?;
        if self.path.exists() {
            fs::copy(&self.path, &backup_path)?;
            fs::remove_file(&self.path)?;
        }
        fs::rename(&temp_path, &self.path)?;
        Ok(())
    }

    pub fn load(&self, password: &[u8]) -> Result<Vault, StorageError> {
        let encrypted = fs::read(&self.path)?;
        let payload = decrypt_container(password, &encrypted)?;
        Ok(serde_json::from_slice(&payload)?)
    }
}
