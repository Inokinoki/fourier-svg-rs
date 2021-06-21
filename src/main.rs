mod fft_drawer;
mod visualizer;
mod path_util;

// Visualizer
use visualizer::Visualizer;
use visualizer::html_visualizer::HTMLVisualizer;

// Path util
use path_util::{
    build_path_from_svg,
    path_to_fft
};

use svg::node::element::tag::Path;
use svg::parser::Event;

use clap::{Arg, App, AppSettings};

fn main() {
    // Add param
    let app = App::new("Fourier SVG Drawer")
        .version("1.0.0")
        .author("Inoki <veyx.shaw@gmail.com>")
        .about("Draw a path in SVG format using Fourier Transform")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("SVG Path")
            .short("p")
            .long("path")
            .help("Draw an SVG path in string")
            .takes_value(true))
        .arg(Arg::with_name("SVG file")
            .short("f")
            .long("file")
            .help("Draw the first SVG path in file")
            .takes_value(true))
        .arg(Arg::with_name("Number of sample points")
            .short("s")
            .long("sample")
            .help("Use how many sample points to draw the path")
            .takes_value(true))
        .arg(Arg::with_name("Number of waves")
            .short("w")
            .long("wave")
            .help("Use how many waves to draw the path")
            .takes_value(true));
    let matches = app.get_matches();

    // SVG source args
    let arg_path = matches.value_of("SVG Path").unwrap_or("");
    let arg_svg_file = matches.value_of("SVG file").unwrap_or("");

    // FFT config args
    let arg_sample = matches.value_of("Number of sample points").unwrap_or("10240");
    let arg_wave = matches.value_of("Number of waves").unwrap_or("201");

    // Retrieve svg from web or local file
    let mut svg_string: String = "".to_string();
    if arg_svg_file.len() > 0 {
        // Read path from svg file
        let mut content = String::new();
        for event in svg::open(arg_svg_file, &mut content).unwrap() {
            match event {
                Event::Tag(Path, _, attributes) => {
                    svg_string = attributes.get("d").unwrap().to_string();
                    // svg_string = data;
                    break;
                }
                _ => {}
            }
        }
    } else if arg_path.len() > 0 {
        // Read path from svg path string
        svg_string = arg_path.to_string();
    } else {
        println!("No SVG path provided.");
        return;
    }

    let num_sample = arg_sample.parse::<usize>().unwrap_or(10240);
    let mut num_wave = arg_wave.parse::<usize>().unwrap_or(201);

    // Make sure num_sample >= num_wave
    if num_sample < num_wave {
        num_wave = num_sample;
    }

    let path = build_path_from_svg(&svg_string);

    let fft_size = num_sample;
    let fft_result = path_to_fft(path, fft_size);

    // Temporally output to json
    let mut data = Vec::new();
    data.push(fft_drawer::DrawData::new_from_complex(0 as f32, fft_result[0]));
    // Can change from param
    for i in 1..((num_wave + 1) / 2) {
        data.push(fft_drawer::DrawData::new_from_complex(i as f32, fft_result[i]));
        data.push(fft_drawer::DrawData::new_from_complex((0 - i as i32) as f32, fft_result[fft_size - i]));
    }

    // TODO: Add an option to choose a different visualizer
    let html_visualizer = HTMLVisualizer::new("output.html".to_string());
    html_visualizer.render(data);
}
