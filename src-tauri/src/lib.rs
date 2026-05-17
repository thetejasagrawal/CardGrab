mod camera;
mod commands;
mod db;
mod device;
mod error;
mod journal;
mod scanner;
mod settings;
mod state;
mod template;
mod thumbs;
mod transfer;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let app_state = state::AppState::initialize(&handle)?;
            app.manage(app_state);

            // Start the device watcher loop in the background.
            let watch_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                device::watch_loop(watch_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_cards,
            commands::scan_card,
            commands::tool_status,
            commands::render_template_preview,
            commands::start_import,
            commands::cancel_import,
            commands::list_imports,
            commands::get_import_files,
            commands::get_import_events,
            commands::reveal_in_finder,
            commands::eject_card,
            commands::pick_destination_dir,
            commands::get_settings,
            commands::set_settings,
            commands::list_templates,
            commands::save_template,
            commands::delete_template,
            commands::get_thumbnail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
