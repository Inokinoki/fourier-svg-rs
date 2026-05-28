use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub timestamp: i64,
}

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

#[tauri::command]
pub async fn add_recent_file(file_path: String, file_name: String) -> Result<Vec<RecentFile>, String> {
    let recent_files_path = get_recent_files_path()?;
    let mut recent_files = load_recent_files()?;

    recent_files.retain(|f| f.path != file_path);

    let timestamp = chrono::Utc::now().timestamp();
    recent_files.insert(
        0,
        RecentFile {
            path: file_path.clone(),
            name: file_name,
            timestamp,
        },
    );

    recent_files.truncate(10);

    let json_str = serde_json::to_string_pretty(&recent_files).map_err(|e| e.to_string())?;
    let mut file = File::create(&recent_files_path).map_err(|e| e.to_string())?;
    file.write_all(json_str.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(recent_files)
}

#[tauri::command]
pub async fn get_recent_files() -> Result<Vec<RecentFile>, String> {
    load_recent_files()
}

fn get_recent_files_path() -> Result<PathBuf, String> {
    let mut path = dirs::config_dir().ok_or("Failed to get config directory")?;
    path.push("fourier-svg");
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    path.push("recent_files.json");
    Ok(path)
}

fn load_recent_files() -> Result<Vec<RecentFile>, String> {
    let path = get_recent_files_path()?;
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let files: Vec<RecentFile> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(files)
    } else {
        Ok(Vec::new())
    }
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
