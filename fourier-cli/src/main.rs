#![allow(non_upper_case_globals)]

use fourier_svg::{
    build_path_from_svg, export_to_draw_data, load_fourier_export, path_to_fft, DrawData,
    ExportVisualizer, GIFVisualizer, HTMLVisualizer, Visualizer,
};

use clap::Parser;
use svg::node::element::tag::Path;
use svg::parser::Event;

/// Draw a path in SVG format using Fourier Transform
#[derive(Parser, Debug)]
#[command(author = "Inoki <veyx.shaw@gmail.com>", version = "1.0.0", about)]
#[command(arg_required_else_help = true)]
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

    /// Rendering backend: html, gif, export
    #[arg(short = 'b', long = "backend", default_value = "html")]
    backend: String,

    /// Output file name (without extension)
    #[arg(short = 'o', long = "output", default_value = "output")]
    output: String,

    /// Number of frames for GIF output
    #[arg(long = "frames", default_value = "100")]
    frames: usize,
}

fn main() {
    let args = Args::parse();

    // SVG source args
    let arg_path = args.svg_path.as_deref().unwrap_or_default();
    let arg_svg_file = args.svg_file.as_deref().unwrap_or_default();
    let input_file = args.input_file.clone();

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
    } else {
        // Compute from SVG
        let mut svg_string: String = String::new();
        if !arg_svg_file.is_empty() {
            // Read path from svg file
            let mut content = String::new();
            for event in svg::open(arg_svg_file, &mut content).unwrap() {
                match event {
                    Event::Tag(Path, _, attributes) => {
                        svg_string = attributes.get("d").unwrap().to_string();
                        break;
                    }
                    _ => {}
                }
            }
        } else if !arg_path.is_empty() {
            // Read path from svg path string
            svg_string = arg_path.to_string();
        } else {
            println!("No SVG path provided.");
            return;
        }

        let num_sample = args.num_sample;
        let mut num_wave = args.num_wave;

        // Make sure num_sample >= num_wave
        if num_sample < num_wave {
            num_wave = num_sample;
        }

        let path = build_path_from_svg(&svg_string);

        let fft_size = num_sample;
        let fft_result = path_to_fft(path, fft_size);

        // Build DrawData from FFT result
        let mut result = Vec::new();
        result.push(DrawData::new_from_complex(0 as f32, fft_result[0]));
        for i in 1..((num_wave + 1) / 2) {
            result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
            result.push(DrawData::new_from_complex(
                (0 - i as i32) as f32,
                fft_result[fft_size - i],
            ));
        }

        result
    };

    // Select visualizer based on backend
    let success =
        match args.backend.as_str() {
            "html" => {
                let visualizer = HTMLVisualizer::new(format!("{}.html", args.output));
                visualizer.render(data)
            }
            "gif" => {
                let visualizer =
                    GIFVisualizer::new(format!("{}.gif", args.output)).with_frames(args.frames);
                visualizer.render(data)
            }
            "export" => {
                let visualizer = ExportVisualizer::new(format!("{}.json", args.output))
                    .with_metadata(args.svg_path.clone(), args.num_sample, args.num_wave);
                visualizer.render(data)
            }
            _ => {
                eprintln!(
                    "Unknown backend: {}. Available options: html, gif, export",
                    args.backend
                );
                eprintln!(
                "For gpui and tauri applications, see src/bin/gpui-app.rs and src/bin/tauri-app.rs"
            );
                false
            }
        };

    if !success {
        eprintln!("Rendering failed!");
    }
}
