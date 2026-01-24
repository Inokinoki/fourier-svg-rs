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

    // Display coefficient information
    println!("Fourier Coefficients:");
    println!("  Index | Frequency |     Radius |     Angle (rad)");
    println!("  ------|-----------|-----------|------------------");
    for (i, d) in data.iter().enumerate().take(10) {
        println!(
            "  {:5} | {:9.2} | {:9.2} | {:15.6}",
            i, d.frequency, d.radius, d.angle
        );
    }
    if data.len() > 10 {
        println!("  ... ({} total coefficients)", data.len());
    }
    println!();

    // Generate HTML visualization
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
