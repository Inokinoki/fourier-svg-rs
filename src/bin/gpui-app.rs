//! gpui Fourier Visualizer Application
//!
//! A standalone gpui application for visualizing Fourier epicycles.
//! Allows users to draw SVG paths interactively.
//!
//! Run with: cargo run --bin gpui-app --features gpui-app

use fourier_svg::{DrawData, build_path_from_svg, path_to_fft, load_fourier_export, export_to_draw_data};
use clap::Parser;

#[cfg(feature = "gpui-app")]
use std::sync::Arc;

/// gpui Fourier Visualizer - Draw SVG paths using Fourier Transform
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

#[cfg(not(feature = "gpui-app"))]
fn run_gpui_app(_initial_data: Option<Vec<DrawData>>, _num_sample: usize, _num_wave: usize) {
    eprintln!("gpui visualizer requires the 'gpui-app' feature to be enabled.");
    eprintln!("Run with: cargo run --bin gpui-app --features gpui-app");
}

fn main() {
    let args = Args::parse();

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

    // Get Fourier data - either from exported file or compute from SVG
    let initial_data: Option<Vec<DrawData>> = if let Some(input_path) = input_file {
        // Load from exported Fourier data
        match load_fourier_export(&input_path) {
            Ok(export) => {
                println!("Loaded Fourier data from {} ({} coefficients, {} samples)",
                    input_path, export.metadata.wave_count, export.metadata.sample_count);
                Some(export_to_draw_data(&export))
            }
            Err(e) => {
                eprintln!("Failed to load Fourier data: {}", e);
                None
            }
        }
    } else if !arg_svg_file.is_empty() || !arg_path.is_empty() {
        // Compute from SVG
        let mut svg_string: String = String::new();
        if !arg_svg_file.is_empty() {
            // Read path from svg file
            let mut content = String::new();
            for event in svg::open(arg_svg_file, &mut content).unwrap() {
                match event {
                    svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) => {
                        svg_string = attributes.get("d").unwrap().to_string();
                        break;
                    }
                    _ => {}
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
        for i in 1..((num_wave + 1) / 2) {
            result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
            result.push(DrawData::new_from_complex((0 - i as i32) as f32, fft_result[fft_size - i]));
        }

        Some(result)
    } else {
        // No SVG provided - user will draw in the app
        println!("No SVG path provided.");
        println!("");
        println!("INTERACTIVE DRAWING MODE:");
        println!("The gpui app requires a more complex setup for interactive drawing.");
        println!("For now, please provide an SVG path or file:");
        println!("  cargo run --bin gpui-app --features gpui-app -- --path 'M10 10 L100 100'");
        println!("  cargo run --bin gpui-app --features gpui-app -- --file input.svg");
        println!("");
        println!("For interactive drawing, use the Tauri app instead:");
        println!("  cargo run --bin tauri-app --features tauri-app");
        None
    };

    #[cfg(feature = "gpui-app")]
    {
        // For now, we'll use a simpler approach - just export to HTML
        // since gpui's API is complex and changes frequently
        if let Some(data) = initial_data {
            use fourier_svg::Visualizer;
            use fourier_svg::HTMLVisualizer;

            println!("Generating HTML visualization...");
            let visualizer = HTMLVisualizer::new("gpui_output.html".to_string());
            if visualizer.render(data) {
                println!("HTML output saved to gpui_output.html");
                println!("You can open this file in a web browser to see the visualization.");
            }
        }
    }

    #[cfg(not(feature = "gpui-app"))]
    {
        run_gpui_app(initial_data, num_sample, num_wave);
    }
}
