//! Fourier SVG Painter
//!
//! A tool that uses the Fourier Transform to draw SVG paths with rotating
//! circles (epicycles). The program decomposes an SVG path into frequency
//! components and generates an animated visualization.

mod fft_drawer;
mod path_util;
mod visualizer;

use visualizer::html_visualizer::HTMLVisualizer;
use visualizer::Visualizer;

use path_util::{build_path_from_svg, path_to_fft};

use svg::node::element::tag::Path as SvgPath;
use svg::parser::Event;

use clap::{App, AppSettings, Arg};

/// Default number of sample points for path discretization.
const DEFAULT_SAMPLE_POINTS: usize = 10240;

/// Default number of Fourier components (waves) to use.
const DEFAULT_WAVE_COUNT: usize = 201;

/// Output filename for the generated HTML visualization.
const OUTPUT_FILE: &str = "output.html";

fn main() {
    // Configure command-line arguments
    let app = App::new("Fourier SVG Drawer")
        .version("1.0.0")
        .author("Inoki <veyx.shaw@gmail.com>")
        .about("Draw a path in SVG format using Fourier Transform epicycles")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("SVG Path")
                .short("p")
                .long("path")
                .help("SVG path string to draw (e.g., \"M 0 0 L 100 100\")")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("SVG file")
                .short("f")
                .long("file")
                .help("SVG file to read the first path from")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Number of sample points")
                .short("s")
                .long("sample")
                .help("Number of sample points for path discretization")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Number of waves")
                .short("w")
                .long("wave")
                .help("Number of Fourier components (epicycles) to use")
                .takes_value(true),
        );

    let matches = app.get_matches();

    // Parse SVG source arguments
    let arg_path = matches.value_of("SVG Path").unwrap_or("");
    let arg_svg_file = matches.value_of("SVG file").unwrap_or("");

    // Parse FFT configuration arguments
    let num_sample = matches
        .value_of("Number of sample points")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_SAMPLE_POINTS);

    let mut num_wave = matches
        .value_of("Number of waves")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WAVE_COUNT);

    // Extract SVG path string from file or command line
    let svg_string = if !arg_svg_file.is_empty() {
        extract_path_from_svg_file(arg_svg_file)
    } else if !arg_path.is_empty() {
        Some(arg_path.to_string())
    } else {
        None
    };

    let svg_string = match svg_string {
        Some(s) if !s.is_empty() => s,
        _ => {
            eprintln!("Error: No valid SVG path provided.");
            eprintln!("Use -f to specify an SVG file or -p to provide a path string.");
            return;
        }
    };

    // Ensure num_sample >= num_wave for valid FFT output
    if num_sample < num_wave {
        num_wave = num_sample;
    }

    // Build path and compute FFT
    let path = build_path_from_svg(&svg_string);
    let fft_result = path_to_fft(path, num_sample);

    // Extract Fourier components for visualization
    // Include DC component (frequency 0) and pairs of positive/negative frequencies
    let mut data = Vec::with_capacity(num_wave);
    data.push(fft_drawer::DrawData::new_from_complex(0.0, fft_result[0]));

    for i in 1..((num_wave + 1) / 2) {
        // Positive frequency component
        data.push(fft_drawer::DrawData::new_from_complex(
            i as f32,
            fft_result[i],
        ));
        // Negative frequency component (from the end of the FFT result)
        data.push(fft_drawer::DrawData::new_from_complex(
            -(i as i32) as f32,
            fft_result[num_sample - i],
        ));
    }

    // Generate visualization
    let html_visualizer = HTMLVisualizer::new(OUTPUT_FILE.to_string());
    if html_visualizer.render(data) {
        println!("Successfully generated: {}", OUTPUT_FILE);
        println!("Open this file in a web browser to see the animation.");
    } else {
        eprintln!("Error: Failed to write output file.");
    }
}

/// Extracts the first path element from an SVG file.
///
/// # Arguments
///
/// * `file_path` - Path to the SVG file
///
/// # Returns
///
/// `Some(String)` containing the path data if found, `None` otherwise.
fn extract_path_from_svg_file(file_path: &str) -> Option<String> {
    let mut content = String::new();

    let parser = match svg::open(file_path, &mut content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Failed to open SVG file '{}': {}", file_path, e);
            return None;
        }
    };

    for event in parser {
        #[allow(non_upper_case_globals)]
        if let Event::Tag(SvgPath, _, attributes) = event {
            if let Some(d) = attributes.get("d") {
                return Some(d.to_string());
            }
        }
    }

    eprintln!("Error: No path element found in SVG file '{}'", file_path);
    None
}
