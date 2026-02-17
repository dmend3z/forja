#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::sparks::SparkStore::default())
        .invoke_handler(tauri::generate_handler![
            commands::projects::list_projects,
            commands::projects::add_project,
            commands::sparks::start_spark,
            commands::sparks::list_sparks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
