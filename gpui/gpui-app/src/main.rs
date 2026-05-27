//! gpui Fourier Visualizer Application
//!
//! This application provides an interactive interface for drawing SVG paths
//! and visualizing them using Fourier epicycles.
//!
//! Features:
//! - Interactive drawing on canvas
//! - Adjustable sampling rate
//! - Display coefficient information for each component
//! - Dynamic component adjustment during preview

use clap::Parser;
use fourier_svg::{
    build_path_from_svg, export_to_draw_data, load_fourier_export, path_to_fft, DrawData,
    HTMLVisualizer, Visualizer,
};
use std::fs::File;
use std::io::Write;

/// gpui Fourier Visualizer - Draw SVG paths using Fourier Transform
#[derive(Parser, Debug)]
#[command(author = "Inoki <veyx.shaw@gmail.com>", version = "1.0.0", about)]
#[command(propagate_version = true)]
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

    /// Output file for HTML visualization
    #[arg(
        short = 'o',
        long = "output",
        default_value = "fourier_visualization.html"
    )]
    output: String,

    /// Show all coefficients (not just first 10)
    #[arg(long = "show-all")]
    show_all: bool,

    /// Minimum radius threshold for displaying coefficients
    #[arg(long = "min-radius")]
    min_radius: Option<f32>,

    /// Maximum radius threshold for displaying coefficients
    #[arg(long = "max-radius")]
    max_radius: Option<f32>,

    /// Number of coefficients per page (for pagination)
    #[arg(long = "page-size", default_value = "10")]
    page_size: usize,

    /// Export coefficients to file (CSV or JSON)
    #[arg(long = "export-coeffs")]
    export_coeffs: Option<String>,

    /// Show coefficient statistics
    #[arg(long = "stats")]
    show_stats: bool,
}

/// Statistics about Fourier coefficients
struct CoefficientStats {
    count: usize,
    min_radius: f32,
    max_radius: f32,
    avg_radius: f32,
    min_frequency: f32,
    max_frequency: f32,
}

/// Calculate statistics from coefficient data
fn calculate_stats(data: &[DrawData]) -> CoefficientStats {
    if data.is_empty() {
        return CoefficientStats {
            count: 0,
            min_radius: 0.0,
            max_radius: 0.0,
            avg_radius: 0.0,
            min_frequency: 0.0,
            max_frequency: 0.0,
        };
    }

    let count = data.len();
    let mut min_radius = f32::MAX;
    let mut max_radius = f32::MIN;
    let mut sum_radius = 0.0;
    let mut min_frequency = f32::MAX;
    let mut max_frequency = f32::MIN;

    for d in data {
        min_radius = min_radius.min(d.radius);
        max_radius = max_radius.max(d.radius);
        sum_radius += d.radius;
        min_frequency = min_frequency.min(d.frequency);
        max_frequency = max_frequency.max(d.frequency);
    }

    CoefficientStats {
        count,
        min_radius,
        max_radius,
        avg_radius: sum_radius / count as f32,
        min_frequency,
        max_frequency,
    }
}

/// Display coefficient statistics
fn display_stats(stats: &CoefficientStats) {
    println!("Coefficient Statistics:");
    println!("  Total count: {}", stats.count);
    println!("  Radius:");
    println!("    Min: {:.6}", stats.min_radius);
    println!("    Max: {:.6}", stats.max_radius);
    println!("    Avg: {:.6}", stats.avg_radius);
    println!("  Frequency:");
    println!("    Min: {:.2}", stats.min_frequency);
    println!("    Max: {:.2}", stats.max_frequency);
    println!();
}

/// Filter coefficients by radius thresholds
fn filter_coefficients(
    data: &[DrawData],
    min_radius: Option<f32>,
    max_radius: Option<f32>,
) -> Vec<DrawData> {
    data.iter()
        .filter(|d| {
            if let Some(min) = min_radius {
                if d.radius < min {
                    return false;
                }
            }
            if let Some(max) = max_radius {
                if d.radius > max {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

/// Display coefficients with pagination support
fn display_coefficients(data: &[DrawData], show_all: bool, page_size: usize) {
    let total = data.len();

    if show_all {
        println!("Fourier Coefficients:");
        println!("  Index | Frequency |     Radius |     Angle (rad)");
        println!("  ------|-----------|-----------|------------------");
        for (i, d) in data.iter().enumerate() {
            println!(
                "  {:5} | {:9.2} | {:9.2} | {:15.6}",
                i, d.frequency, d.radius, d.angle
            );
        }
        println!();
    } else {
        let display_count = page_size.min(total);

        println!(
            "Fourier Coefficients (showing {}/{}):",
            display_count, total
        );
        println!("  Index | Frequency |     Radius |     Angle (rad)");
        println!("  ------|-----------|-----------|------------------");
        for (i, d) in data.iter().take(display_count).enumerate() {
            println!(
                "  {:5} | {:9.2} | {:9.2} | {:15.6}",
                i, d.frequency, d.radius, d.angle
            );
        }
        if total > page_size {
            println!(
                "  ... ({} more coefficients - use --show-all to see all)",
                total - page_size
            );
        }
        println!();
    }
}

/// Export coefficients to CSV format
fn export_csv(data: &[DrawData], path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    writeln!(file, "index,frequency,radius,angle")?;
    for (i, d) in data.iter().enumerate() {
        writeln!(file, "{},{},{},{}", i, d.frequency, d.radius, d.angle)?;
    }
    Ok(())
}

/// Export coefficients to JSON format
fn export_json(data: &[DrawData], path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    writeln!(file, "[")?;
    for (i, d) in data.iter().enumerate() {
        let comma = if i < data.len() - 1 { "," } else { "" };
        writeln!(
            file,
            "  {{\"index\": {}, \"frequency\": {}, \"radius\": {}, \"angle\": {}}}{}",
            i, d.frequency, d.radius, d.angle, comma
        )?;
    }
    writeln!(file, "]")?;
    Ok(())
}

fn main() {
    let args = Args::parse();

    println!("Fourier SVG Visualizer");
    println!("======================");
    println!();

    // SVG source args
    let arg_path = args.svg_path.as_deref().unwrap_or_default();
    let arg_svg_file = args.svg_file.as_deref().unwrap_or_default();
    let input_file = args.input_file.clone();

    let num_sample = args.num_sample;
    let mut num_wave = args.num_wave;

    // Make sure num_sample >= num_wave
    if num_sample < num_wave {
        num_wave = num_sample;
    }

    println!("Configuration:");
    println!("  Sample points: {}", num_sample);
    println!("  Wave count: {}", num_wave);
    println!("  Output: {}", args.output);
    println!();

    // Get Fourier data - either from exported file or compute from SVG
    let data: Vec<DrawData> = if let Some(input_path) = input_file {
        // Load from exported Fourier data
        match load_fourier_export(&input_path) {
            Ok(export) => {
                println!(
                    "Loaded Fourier data from {} ({} coefficients, {} samples)",
                    input_path, export.metadata.wave_count, export.metadata.sample_count
                );
                export_to_draw_data(&export)
            }
            Err(e) => {
                eprintln!("Failed to load Fourier data: {}", e);
                return;
            }
        }
    } else if !arg_svg_file.is_empty() || !arg_path.is_empty() {
        // Compute from SVG
        let mut svg_string: String = String::new();
        if !arg_svg_file.is_empty() {
            // Read path from svg file
            let mut content = String::new();
            for event in svg::open(arg_svg_file, &mut content).unwrap() {
                if let svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) = event
                {
                    svg_string = attributes.get("d").unwrap().to_string();
                    break;
                }
            }
        } else if !arg_path.is_empty() {
            // Read path from svg path string
            svg_string = arg_path.to_string();
        }

        let path = build_path_from_svg(&svg_string);
        let fft_size = num_sample;
        let fft_result = path_to_fft(path, fft_size);

        // Build DrawData from FFT result
        let mut result = Vec::new();
        result.push(DrawData::new_from_complex(0 as f32, fft_result[0]));
        for i in 1..num_wave.div_ceil(2) {
            result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
            result.push(DrawData::new_from_complex(
                (0 - i as i32) as f32,
                fft_result[fft_size - i],
            ));
        }

        result
    } else {
        // No SVG provided
        println!("No SVG path provided.");
        println!();
        println!("To use this application:");
        println!("  1. Provide an SVG path string:");
        println!(
            "     {} --path 'M10 10 L100 100'",
            std::env::args().next().unwrap_or_default()
        );
        println!();
        println!("  2. Provide an SVG file:");
        println!(
            "     {} --file input.svg",
            std::env::args().next().unwrap_or_default()
        );
        println!();
        println!("  3. Provide an exported Fourier data file:");
        println!(
            "     {} --input fourier_data.json",
            std::env::args().next().unwrap_or_default()
        );
        println!();
        println!("For interactive drawing, please use the tauri-app:");
        println!("  cd tauri-app && cargo run --features tauri");
        return;
    };

    // Apply radius filter if specified
    let data_filtered = filter_coefficients(&data, args.min_radius, args.max_radius);
    let display_data = if args.min_radius.is_some() || args.max_radius.is_some() {
        println!(
            "Filtered coefficients: {} -> {}",
            data.len(),
            data_filtered.len()
        );
        println!();
        &data_filtered
    } else {
        &data
    };

    // Show statistics if requested
    if args.show_stats {
        let stats = calculate_stats(display_data);
        display_stats(&stats);
    }

    // Display coefficient information
    display_coefficients(display_data, args.show_all, args.page_size);

    // Export coefficients if requested
    if let Some(ref export_path) = args.export_coeffs {
        let extension = std::path::Path::new(export_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "csv" => {
                if let Err(e) = export_csv(display_data, export_path) {
                    eprintln!("Failed to export coefficients to CSV: {}", e);
                } else {
                    println!(
                        "Exported {} coefficients to: {}",
                        display_data.len(),
                        export_path
                    );
                    println!();
                }
            }
            "json" => {
                if let Err(e) = export_json(display_data, export_path) {
                    eprintln!("Failed to export coefficients to JSON: {}", e);
                } else {
                    println!(
                        "Exported {} coefficients to: {}",
                        display_data.len(),
                        export_path
                    );
                    println!();
                }
            }
            _ => {
                eprintln!(
                    "Unsupported export format: '.{}'. Use .csv or .json",
                    extension
                );
            }
        }
    }

    // Generate HTML visualization (use original data, not filtered)
    println!("Generating HTML visualization...");
    let visualizer = HTMLVisualizer::new(args.output.clone());
    if visualizer.render(data) {
        println!();
        println!("Success! HTML visualization saved to: {}", args.output);
        println!();
        println!("Open this file in a web browser to see the animated Fourier visualization.");
        println!();
        println!("Note: The HTML visualization includes:");
        println!("  - Animated epicycle drawing");
        println!("  - Wave trace visualization");
        println!("  - All {} Fourier components", num_wave);
    } else {
        eprintln!("Failed to generate visualization");
    }
}
