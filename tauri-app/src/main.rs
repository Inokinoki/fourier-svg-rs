//! Tauri Fourier Visualizer Application
//!
//! This application provides an interactive interface for drawing SVG paths
//! and visualizing them using Fourier epicycles.
//!
//! Features:
//! - Interactive drawing on canvas
//! - Adjustable sampling rate
//! - Display coefficient information for each component
//! - Dynamic component adjustment during preview
//!
//! Build requirements on Linux:
//!   sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

#[cfg(feature = "tauri")]
mod commands;

use clap::Parser;
use fourier_svg::{
    export_to_draw_data, load_fourier_export, process_source, DrawData, FourierConfig,
    FourierSource,
};

/// Tauri Fourier Visualizer
#[derive(Parser, Debug)]
#[command(author = "Inoki <veyx.shaw@gmail.com>", version = "1.0.0", about)]
struct Args {
    /// Draw an SVG path in string
    #[arg(short = 'p', long = "path")]
    svg_path: Option<String>,

    /// Draw the first SVG path in file
    #[arg(short = 'f', long = "file")]
    svg_file: Option<String>,

    /// Load from exported Fourier data JSON file
    #[arg(short = 'i', long = "input")]
    input_file: Option<String>,

    /// Use how many sample points to draw the path
    #[arg(short = 's', long = "sample", default_value = "10240")]
    num_sample: usize,

    /// Use how many waves to draw the path
    #[arg(short = 'w', long = "wave", default_value = "201")]
    num_wave: usize,
}

#[cfg(feature = "tauri")]
fn run_tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::drawing::process_drawing,
            commands::svg::parse_svg_file,
            commands::svg::process_svg_path,
            commands::svg::get_svg_paths,
            commands::export_cmd::export_fourier_data,
            commands::export_cmd::export_as_gif,
            commands::export_cmd::export_as_html,
            commands::files::save_canvas_as_png,
            commands::files::add_recent_file,
            commands::files::get_recent_files,
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
    let args = Args::parse();

    let num_sample = args.num_sample;
    let num_wave = args.num_wave;
    let config = FourierConfig::new(num_sample, num_wave);

    let _initial_data: Option<Vec<DrawData>> = if let Some(input_path) = args.input_file.clone() {
        match load_fourier_export(&input_path) {
            Ok(export) => {
                println!(
                    "Loaded Fourier data from {} ({} coefficients, {} samples)",
                    input_path, export.metadata.wave_count, export.metadata.sample_count
                );
                Some(export_to_draw_data(&export))
            }
            Err(e) => {
                eprintln!("Failed to load Fourier data: {}", e);
                None
            }
        }
    } else if args.svg_file.is_some() || args.svg_path.is_some() {
        let source = if let Some(ref file) = args.svg_file {
            FourierSource::SvgFile(file)
        } else {
            FourierSource::SvgPath(args.svg_path.as_deref().unwrap())
        };

        match process_source(source, &config) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("Error: {}", e);
                None
            }
        }
    } else {
        println!("No SVG path provided - launching in interactive drawing mode");
        None
    };

    // For now, just run the Tauri app
    // TODO: Pass initial_data to the Tauri app
    run_tauri_app();
}
