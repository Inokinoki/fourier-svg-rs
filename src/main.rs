use rustfft::{FftPlanner, num_complex::Complex};

use lyon_path::iterator::*;
use lyon_path::math::{point, vector};
use lyon_path::geom::BezierSegment;
use lyon_path::{Path, PathEvent};
use lyon_svg::path_utils::build_path;

mod fft_drawer;
mod visualizer;

// Visualizer
use visualizer::Visualizer;
use visualizer::html_visualizer::HTMLVisualizer;

fn compute_path_length(path: &Path) -> f32 {
    // A simple std::iter::Iterator<PathEvent>,
    let simple_iter = path.iter();

    // Make it an iterator over simpler primitives flattened events,
    // which do not contain any curve. To do so we approximate each curve
    // linear segments according to a tolerance threshold which controls
    // the tradeoff between fidelity of the approximation and amount of
    // generated events. Let's use a tolerance threshold of 0.01.
    // The beauty of this approach is that the flattening happens lazily
    // while iterating without allocating memory for the path.
    let flattened_iter = path.iter().flattened(0.01);

    let mut total_length: f32 = 0.0;
    for evt in flattened_iter {
        match evt {
            PathEvent::Begin { at } => {}
            PathEvent::Line { from, to } => { total_length += (to - from).length(); }
            PathEvent::End { last, first, close } => {
                if close {
                    // Add the closed path
                    total_length += (first - last).length();
                }
            }
            _ => { panic!() }
        }
    }
    total_length
}

fn construct_sample_points(path: &Path, total_length: f32, n_sample: usize) -> Vec<Complex<f32>> {
    let mut samples = Vec::new();

    // A simple std::iter::Iterator<PathEvent>,
    let simple_iter = path.iter();

    // Make it an iterator over simpler primitives flattened events,
    // which do not contain any curve. To do so we approximate each curve
    // linear segments according to a tolerance threshold which controls
    // the tradeoff between fidelity of the approximation and amount of
    // generated events. Let's use a tolerance threshold of 0.01.
    // The beauty of this approach is that the flattening happens lazily
    // while iterating without allocating memory for the path.
    let flattened_iter = path.iter().flattened(0.01);

    let mut itered_length: f32 = 0.0;
    let mut itered_index: u32 = 0;
    let sample_length = total_length / (n_sample as f32);
    for evt in flattened_iter {
        match evt {
            PathEvent::Begin { at } => {
                // Add as the first one
                samples.push(Complex{ re: at.x, im: at.y });
                // println!("Add sample point {:?} at {:?} for begin", itered_index, at);
                itered_index += 1;
            }
            PathEvent::Line { from, to } => {
                let next_sample_length = sample_length * (itered_index as f32);
                let current_line_length = (to - from).length();
                let mut last_added_sample_on_this_segment: f32 = 0.0;
                if (itered_length < next_sample_length) {
                    if itered_length + current_line_length >= next_sample_length {
                        last_added_sample_on_this_segment = sample_length
                            - (itered_length - sample_length * ((itered_index - 1) as f32));
                        // Add a sample point on the segment
                        let sample = from + (to - from) * 
                            ((last_added_sample_on_this_segment) / current_line_length);
                        samples.push(Complex{ re: sample.x, im: sample.y });
                        // println!("Add sample point {:?} at {:?}", itered_index, sample);
                        // Ready to find the next sample point
                        itered_index += 1;
                    }
                }
                // println!("last_added_sample_on_this_segment {:?}", last_added_sample_on_this_segment);

                // Compensation
                let mut compensation_counter = 0;
                while sample_length * (itered_index as f32) <= itered_length + current_line_length {
                    // Add a sample point for compensation
                    let sample = from + (to -from) * (sample_length * compensation_counter as f32) / current_line_length +
                        (to - from) * (last_added_sample_on_this_segment + sample_length) / current_line_length;
                    samples.push(Complex{ re: sample.x, im: sample.y });
                    // println!("Add sample point {:?} at {:?} for compensation", itered_index, sample);
                    // Ready to find the next sample point
                    itered_index += 1;
                    compensation_counter += 1;
                }

                // Accumulate the iterated length
                itered_length += current_line_length;
            }
            PathEvent::End { last, first, close } => {
                if close {
                    // Alias them
                    let from = last;
                    let to = first;

                    let next_sample_length = sample_length * (itered_index as f32);
                    let current_line_length = (to - from).length();
                    let mut last_added_sample_on_this_segment: f32 = 0.0;
                    if (itered_length < next_sample_length) {
                        if itered_length + current_line_length >= next_sample_length {
                            last_added_sample_on_this_segment = sample_length
                                - (itered_length - sample_length * ((itered_index - 1) as f32));
                            // Add a sample point on the segment
                            let sample = from + (to - from) * 
                                ((last_added_sample_on_this_segment) / current_line_length);
                            samples.push(Complex{ re: sample.x, im: sample.y });
                            // println!("Add sample point {:?} at {:?}", itered_index, sample);
                            // Ready to find the next sample point
                            itered_index += 1;
                        }
                    }
                    // println!("last_added_sample_on_this_segment {:?}", last_added_sample_on_this_segment);

                    // Compensation
                    let mut compensation_counter = 0;
                    while sample_length * (itered_index as f32) < itered_length + current_line_length {
                        // Add a sample point for compensation
                        let sample = from + (to -from) * (sample_length * compensation_counter as f32) / current_line_length +
                            (to - from) * (last_added_sample_on_this_segment + sample_length) / current_line_length;
                        samples.push(Complex{ re: sample.x, im: sample.y });
                        // println!("Add sample point {:?} at {:?} for compensation", itered_index, sample);
                        // Ready to find the next sample point
                        itered_index += 1;
                        compensation_counter += 1;
                    }
                }
            }
            _ => { panic!() }
        }
    }
    samples
}

fn path_to_fft(path: Path, n_sample: usize) -> Vec<Complex<f32>> {
    let path_length = compute_path_length(&path);
    let mut samples = construct_sample_points(&path, path_length, n_sample);

    while samples.len() > n_sample {
        samples.remove(n_sample);
    }
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n_sample);

    fft.process(&mut samples);

    for i in 0..samples.len() {
        samples[i] = samples[i] / samples.len() as f32;
    }
    samples
}

fn build_path_from_svg(svg_commands: &str) -> Path {
    let svg_builder = Path::builder().with_svg();
    match build_path(svg_builder, svg_commands) {
        Ok (path) => {
            return path;
        }
        _ => {
            panic!();
        }
    }
}

use clap::{Arg, App, SubCommand};

fn main() {
    // Add param
    let app = App::new("Fourier SVG Drawer")
        .version("1.0.0")
        .author("Inoki <veyx.shaw@gmail.com>")
        .about("Draw a path in SVG format using Fourier Transform")
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

    if arg_svg_file.len() > 0 {
        // TODO: Read path from svg file
        return;
    } else if (arg_path.len() > 0) {
        // TODO: Read path from svg path string
        return;
    }

    let num_sample = arg_wave.parse::<usize>().unwrap_or(10240);
    let mut num_wave = arg_wave.parse::<usize>().unwrap_or(201);

    // Make sure num_sample >= num_wave
    if num_sample < num_wave {
        num_wave = num_sample;
    }

    // TODO: Retrieve svg from web or local file

    // Start with a path.
    // let mut builder = Path::builder();
    // builder.begin(point(0.0, 0.0));
    // builder.line_to(point(10.0, 0.0));
    // builder.cubic_bezier_to(point(10.0, 10.0), point(0.0, 10.0), point(0.0, 5.0));
    // builder.end(true);
    // let path = builder.build();
    let svg_string = &"M210.333,65.331C104.367,66.105-12.349,150.637,1.056,276.449c4.303,40.393,18.533,63.704,52.171,79.03
c36.307,16.544,57.022,54.556,50.406,112.954c-9.935,4.88-17.405,11.031-19.132,20.015c7.531-0.17,14.943-0.312,22.59,4.341
c20.333,12.375,31.296,27.363,42.979,51.72c1.714,3.572,8.192,2.849,8.312-3.078c0.17-8.467-1.856-17.454-5.226-26.933
c-2.955-8.313,3.059-7.985,6.917-6.106c6.399,3.115,16.334,9.43,30.39,13.098c5.392,1.407,5.995-3.877,5.224-6.991
c-1.864-7.522-11.009-10.862-24.519-19.229c-4.82-2.984-0.927-9.736,5.168-8.351l20.234,2.415c3.359,0.763,4.555-6.114,0.882-7.875
c-14.198-6.804-28.897-10.098-53.864-7.799c-11.617-29.265-29.811-61.617-15.674-81.681c12.639-17.938,31.216-20.74,39.147,43.489
c-5.002,3.107-11.215,5.031-11.332,13.024c7.201-2.845,11.207-1.399,14.791,0c17.912,6.998,35.462,21.826,52.982,37.309
c3.739,3.303,8.413-1.718,6.991-6.034c-2.138-6.494-8.053-10.659-14.791-20.016c-3.239-4.495,5.03-7.045,10.886-6.876
c13.849,0.396,22.886,8.268,35.177,11.218c4.483,1.076,9.741-1.964,6.917-6.917c-3.472-6.085-13.015-9.124-19.18-13.413
c-4.357-3.029-3.025-7.132,2.697-6.602c3.905,0.361,8.478,2.271,13.908,1.767c9.946-0.925,7.717-7.169-0.883-9.566
c-19.036-5.304-39.891-6.311-61.665-5.225c-43.837-8.358-31.554-84.887,0-90.363c29.571-5.132,62.966-13.339,99.928-32.156
c32.668-5.429,64.835-12.446,92.939-33.85c48.106-14.469,111.903,16.113,204.241,149.695c3.926,5.681,15.819,9.94,9.524-6.351
c-15.893-41.125-68.176-93.328-92.13-132.085c-24.581-39.774-14.34-61.243-39.957-91.247
c-21.326-24.978-47.502-25.803-77.339-17.365c-23.461,6.634-39.234-7.117-52.98-31.273C318.42,87.525,265.838,64.927,210.333,65.331
z";

    let path = build_path_from_svg(svg_string);

    let fft_size = num_sample;
    let mut fft_result = path_to_fft(path, fft_size);

    // Temporally output to json
    let mut data = Vec::new();
    data.push(fft_drawer::DrawData::new_from_complex(0 as f32, fft_result[0]));
    // Can change from param
    for i in 1..(num_wave / 2) {
        data.push(fft_drawer::DrawData::new_from_complex(i as f32, fft_result[i]));
        data.push(fft_drawer::DrawData::new_from_complex((0 - i as i32) as f32, fft_result[fft_size - i]));
    }

    // TODO: Add an option to choose a different visualizer
    let html_visualizer = HTMLVisualizer::new("output.html".to_string());
    html_visualizer.render(data);
}
