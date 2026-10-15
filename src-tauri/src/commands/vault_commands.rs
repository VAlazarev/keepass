use std::sync::Mutex;

use tauri::State;

use crate::session::{SessionError, VaultSession, VaultSnapshot};

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
        SessionError::Storage(_) | SessionError::PathNotSet => CommandError {
            code: "VAULT_ERROR",
            message: "Не удалось открыть хранилище",
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
