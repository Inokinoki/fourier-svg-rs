use rustfft::{FftPlanner, num_complex::Complex};

use lyon_path::iterator::*;
use lyon_path::math::{point, vector};
use lyon_path::geom::BezierSegment;
use lyon_path::{Path, PathEvent};

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

fn construct_sample_points(path: &Path, total_length: f32, n_sample: u32) -> Vec<Complex<f32>> {
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
                // println!("Add sample point {:?} at 0", itered_index);
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

fn main() {
    // Start with a path.
    let mut builder = Path::builder();
    builder.begin(point(0.0, 0.0));
    builder.line_to(point(10.0, 0.0));
    builder.cubic_bezier_to(point(10.0, 10.0), point(0.0, 10.0), point(0.0, 5.0));
    builder.end(true);
    let path = builder.build();

    let path_length = compute_path_length(&path);
    println!("Length: {:?}", path_length);
    let mut samples = construct_sample_points(&path, path_length, 512);
    println!("Samples Length: {:?}", samples.len());

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(512);

    fft.process(&mut samples);

    let fft_size = 512;
    for i in 0..10 {
        println!("Freq +{:?}: {:?}, \t -{:?}: {:?}", i, samples[i], i, samples[fft_size - 1 - i]);
    }
}
