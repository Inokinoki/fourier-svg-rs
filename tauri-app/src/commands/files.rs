use std::fs::File;
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn open_file_dialog(
    filters: Vec<FileFilter>,
    window: tauri::Window,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut dialog = window.dialog().file();
    for filter in &filters {
        let ext_refs: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&filter.name, &ext_refs);
    }

    let result = dialog.blocking_pick_file();
    Ok(result.map(|p| p.to_string()))
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn save_file_dialog(
    default_name: String,
    filters: Vec<FileFilter>,
    window: tauri::Window,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut dialog = window.dialog().file().set_file_name(&default_name);
    for filter in &filters {
        let ext_refs: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&filter.name, &ext_refs);
    }

    let result = dialog.blocking_save_file();
    Ok(result.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn save_canvas_as_png(data_url: String, file_path: String) -> Result<(), String> {
    let base64_data = data_url
        .strip_prefix("data:image/png;base64,")
        .ok_or("Invalid data URL")?;

    let image_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        base64_data,
    )
    .map_err(|e| e.to_string())?;

    let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
    file.write_all(&image_bytes).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(not(feature = "tauri"))]
#[tauri::command]
pub async fn open_file_dialog(_filters: Vec<FileFilter>) -> Result<Option<String>, String> {
    Err("Tauri not enabled".to_string())
}

#[cfg(not(feature = "tauri"))]
#[tauri::command]
pub async fn save_file_dialog(
    _default_name: String,
    _filters: Vec<FileFilter>,
) -> Result<Option<String>, String> {
    Err("Tauri not enabled".to_string())
}
