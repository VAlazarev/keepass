use std::sync::Mutex;

use tauri::State;

use crate::core::model::{Entry, EntryInput};
use crate::core::vault::Vault;
use crate::session::{SessionError, VaultSession, VaultSnapshot};
use crate::storage::local::StorageError;

pub struct AppState {
    pub session: Mutex<VaultSession>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session: Mutex::new(VaultSession::new()),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: &'static str,
}

pub type CommandResult<T> = Result<T, CommandError>;

fn map_error(error: SessionError) -> CommandError {
    match error {
        SessionError::Locked => CommandError {
            code: "LOCKED",
            message: "Хранилище заблокировано",
        },
        SessionError::PathNotSet => CommandError {
            code: "PATH_NOT_SET",
            message: "Не указан путь к файлу хранилища",
        },
        SessionError::Storage(StorageError::Io(_)) => CommandError {
            code: "STORAGE_IO_ERROR",
            message: "Не удалось прочитать или записать файл хранилища",
        },
        SessionError::Storage(StorageError::Serialization(_)) => CommandError {
            code: "VAULT_FORMAT_ERROR",
            message: "Файл хранилища имеет повреждённый формат",
        },
        SessionError::Storage(StorageError::Crypto(_)) => CommandError {
            code: "VAULT_AUTH_ERROR",
            message: "Неверный мастер-пароль или файл хранилища повреждён",
        },
    }
}

#[tauri::command]
pub fn create_vault(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> CommandResult<VaultSnapshot> {
    let mut session = state.session.lock().map_err(|_| CommandError {
        code: "SESSION_ERROR",
        message: "Не удалось получить состояние приложения",
    })?;
    session
        .create(path, password.as_bytes())
        .map_err(map_error)?;
    session.snapshot().map_err(map_error)
}

#[tauri::command]
pub fn unlock_vault(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> CommandResult<VaultSnapshot> {
    let mut session = state.session.lock().map_err(|_| CommandError {
        code: "SESSION_ERROR",
        message: "Не удалось получить состояние приложения",
    })?;
    session
        .unlock(path, password.as_bytes())
        .map_err(map_error)?;
    session.snapshot().map_err(map_error)
}

#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> CommandResult<()> {
    let mut session = state.session.lock().map_err(|_| CommandError {
        code: "SESSION_ERROR",
        message: "Не удалось получить состояние приложения",
    })?;
    session.lock();
    Ok(())
}

#[tauri::command]
pub fn get_vault_snapshot(state: State<'_, AppState>) -> CommandResult<VaultSnapshot> {
    let session = state.session.lock().map_err(|_| CommandError {
        code: "SESSION_ERROR",
        message: "Не удалось получить состояние приложения",
    })?;
    session.snapshot().map_err(map_error)
}

#[tauri::command]
pub fn get_vault_data(state: State<'_, AppState>) -> CommandResult<Vault> {
    let session = state.session.lock().map_err(|_| CommandError {
        code: "SESSION_ERROR",
        message: "Не удалось получить состояние приложения",
    })?;
    session.data().map_err(map_error)
}

#[tauri::command]
pub fn save_vault(state: State<'_, AppState>) -> CommandResult<()> {
    let session = state.session.lock().map_err(|_| CommandError {
        code: "SESSION_ERROR",
        message: "Не удалось получить состояние приложения",
    })?;
    session.save().map_err(map_error)
}

#[tauri::command]
pub fn create_entry(state: State<'_, AppState>, input: EntryInput) -> CommandResult<Entry> {
    let mut session = state.session.lock().map_err(|_| CommandError {
        code: "SESSION_ERROR",
        message: "Не удалось получить состояние приложения",
    })?;
    session.create_entry(input).map_err(map_error)
}

#[cfg(test)]
mod tests {
    use super::map_error;
    use crate::session::SessionError;
    use crate::storage::local::StorageError;

    #[test]
    fn io_failure_reports_a_specific_safe_reason() {
        let error = map_error(SessionError::Storage(StorageError::Io(
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        )));

        assert_eq!(error.code, "STORAGE_IO_ERROR");
        assert_eq!(error.message, "Не удалось прочитать или записать файл хранилища");
    }
}
