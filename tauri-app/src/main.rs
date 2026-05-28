//! Tauri Fourier Visualizer Application

#[cfg(feature = "tauri")]
mod commands;

#[cfg(feature = "tauri")]
fn run_tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::drawing::process_drawing,
            commands::svg::parse_svg_file,
            commands::svg::process_svg_path,
            commands::export_cmd::export_fourier_data,
            commands::export_cmd::export_as_gif,
            commands::export_cmd::export_as_html,
            commands::files::save_canvas_as_png,
            commands::files::open_file_dialog,
            commands::files::save_file_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(feature = "tauri"))]
fn run_tauri_app() {
    eprintln!("Tauri visualizer requires the 'tauri' feature to be enabled.");
    eprintln!("Run with: cargo run --features tauri");
    eprintln!();
    eprintln!("Build requirements on Linux:");
    eprintln!("  sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev");
}

fn main() {
    run_tauri_app();
}
