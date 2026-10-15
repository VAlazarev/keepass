pub mod commands;
pub mod core;
pub mod session;
pub mod storage;
pub mod sync;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(commands::vault_commands::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::vault_commands::create_vault,
            commands::vault_commands::unlock_vault,
            commands::vault_commands::lock_vault,
            commands::vault_commands::get_vault_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running Keepass Drive Vault");
}
