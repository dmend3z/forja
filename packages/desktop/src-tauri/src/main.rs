#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::sparks::SparkStore::default())
        .manage(commands::sparks::ChildPidStore::default())
        .invoke_handler(tauri::generate_handler![
            commands::projects::list_projects,
            commands::projects::add_project,
            commands::sparks::start_spark,
            commands::sparks::list_sparks,
            commands::sparks::stop_spark,
            commands::marketplace::get_forja_paths,
            commands::marketplace::list_skills,
            commands::marketplace::search_skills,
            commands::marketplace::get_skill_detail,
            commands::marketplace::install_skill,
            commands::marketplace::uninstall_skill,
            commands::marketplace::create_skill,
            commands::specs::list_specs,
            commands::specs::get_spec,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
