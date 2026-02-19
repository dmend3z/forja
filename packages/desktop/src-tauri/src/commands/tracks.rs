use forja_core::models::track::{self, TrackFile};
use std::path::PathBuf;

#[tauri::command]
pub fn list_tracks(project_path: String) -> Result<Vec<TrackFile>, String> {
    let tracks_dir = PathBuf::from(&project_path).join(".forja").join("tracks");
    track::discover_tracks(&tracks_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_track(project_path: String, track_id: String) -> Result<TrackFile, String> {
    let tracks_dir = PathBuf::from(&project_path).join(".forja").join("tracks");
    track::find_track(&tracks_dir, &track_id).map_err(|e| e.to_string())
}
