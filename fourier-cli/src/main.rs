use fourier_svg::{
    export_to_draw_data, load_fourier_export, DrawData, ExportVisualizer, FourierConfig,
    FourierSource, GIFVisualizer, HTMLVisualizer, Visualizer,
};

use clap::Parser;

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

    let config = FourierConfig::new(args.num_sample, args.num_wave);

    // Get Fourier data from the appropriate source
    let data: Vec<DrawData> = if let Some(input_path) = &args.input_file {
        match load_fourier_export(input_path) {
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
        let source = if let Some(svg_file) = &args.svg_file {
            FourierSource::SvgFile(svg_file)
        } else if let Some(svg_path) = &args.svg_path {
            FourierSource::SvgPath(svg_path)
        } else {
            println!("No SVG path provided. Use -p <path> or -f <file>.");
            return;
        };

        match fourier_svg::process_source(source, &config) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
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
                false
            }
        };

    if !success {
        eprintln!("Rendering failed!");
    }
}
