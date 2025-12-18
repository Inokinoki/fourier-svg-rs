//! SVG path utilities for sampling and FFT conversion.
//!
//! This module provides functions to:
//! - Parse SVG path strings into lyon paths
//! - Compute path lengths
//! - Sample paths at evenly-spaced intervals
//! - Convert sampled paths to Fourier coefficients using FFT

use lyon_path::iterator::*;
use lyon_path::{Path, PathEvent};
use lyon_svg::path_utils::build_path;

use rustfft::{num_complex::Complex, FftPlanner};

/// The tolerance threshold for path flattening.
/// Lower values produce more accurate curves but more segments.
const FLATTEN_TOLERANCE: f32 = 0.01;

/// Builds a lyon `Path` from an SVG path command string.
///
/// # Arguments
///
/// * `svg_commands` - A string containing SVG path commands (e.g., "M 0 0 L 100 100")
///
/// # Returns
///
/// A lyon `Path` object representing the parsed SVG path.
///
/// # Panics
///
/// Panics if the SVG path string is malformed or cannot be parsed.
///
/// # Example
///
/// ```
/// let path = build_path_from_svg("M 0 0 L 100 0 L 100 100 Z");
/// ```
pub fn build_path_from_svg(svg_commands: &str) -> Path {
    let svg_builder = Path::builder().with_svg();
    match build_path(svg_builder, svg_commands) {
        Ok(path) => path,
        Err(_) => panic!("Failed to parse SVG path commands"),
    }
}

/// Computes the total length of a path by summing all segment lengths.
///
/// The path is flattened (curves are approximated as line segments) before
/// computing the length. This provides a good approximation of the true
/// arc length.
///
/// # Arguments
///
/// * `path` - A reference to the lyon `Path` to measure
///
/// # Returns
///
/// The total length of the path as a `f32`.
pub fn compute_path_length(path: &Path) -> f32 {
    // Flatten curves into line segments for easier length computation.
    // The flattening happens lazily without allocating memory for the path.
    let flattened_iter = path.iter().flattened(FLATTEN_TOLERANCE);

    let mut total_length: f32 = 0.0;
    for evt in flattened_iter {
        match evt {
            PathEvent::Begin { at: _ } => {}
            PathEvent::Line { from, to } => {
                total_length += (to - from).length();
            }
            PathEvent::End { last, first, close } => {
                if close {
                    // Add the closing segment length
                    total_length += (first - last).length();
                }
            }
            _ => panic!("Unexpected path event after flattening"),
        }
    }
    total_length
}

/// Constructs evenly-spaced sample points along a path.
///
/// This function walks along the flattened path and samples points at
/// regular intervals based on the total path length divided by the
/// number of samples.
///
/// # Arguments
///
/// * `path` - A reference to the lyon `Path` to sample
/// * `total_length` - The pre-computed total length of the path
/// * `n_sample` - The desired number of sample points
///
/// # Returns
///
/// A vector of `Complex<f32>` where each complex number represents a 2D point
/// with `re` as x-coordinate and `im` as y-coordinate.
pub fn construct_sample_points(
    path: &Path,
    total_length: f32,
    n_sample: usize,
) -> Vec<Complex<f32>> {
    let mut samples = Vec::with_capacity(n_sample);

    // Flatten curves into line segments for uniform sampling
    let flattened_iter = path.iter().flattened(FLATTEN_TOLERANCE);

    let mut iterated_length: f32 = 0.0;
    let mut sample_index: u32 = 0;
    let sample_length = total_length / (n_sample as f32);

    for evt in flattened_iter {
        match evt {
            PathEvent::Begin { at } => {
                // Add the starting point as the first sample
                samples.push(Complex { re: at.x, im: at.y });
                sample_index += 1;
            }
            PathEvent::Line { from, to } => {
                sample_line_segment(
                    from,
                    to,
                    &mut samples,
                    &mut iterated_length,
                    &mut sample_index,
                    sample_length,
                );
            }
            PathEvent::End { last, first, close } => {
                if close {
                    // Handle the closing segment from last point back to first
                    sample_line_segment(
                        last,
                        first,
                        &mut samples,
                        &mut iterated_length,
                        &mut sample_index,
                        sample_length,
                    );
                }
            }
            _ => panic!("Unexpected path event after flattening"),
        }
    }
    samples
}

/// Samples points along a single line segment.
///
/// This helper function adds sample points along a line segment at regular
/// intervals, handling both the first sample on the segment and any
/// additional samples needed for longer segments.
fn sample_line_segment(
    from: lyon_path::math::Point,
    to: lyon_path::math::Point,
    samples: &mut Vec<Complex<f32>>,
    iterated_length: &mut f32,
    sample_index: &mut u32,
    sample_length: f32,
) {
    let next_sample_length = sample_length * (*sample_index as f32);
    let current_line_length = (to - from).length();
    let mut last_added_sample_on_segment: f32 = 0.0;

    // Check if we need to add a sample point on this segment
    if *iterated_length < next_sample_length
        && *iterated_length + current_line_length >= next_sample_length
    {
        last_added_sample_on_segment =
            sample_length - (*iterated_length - sample_length * ((*sample_index - 1) as f32));

        // Interpolate to find the sample point on the segment
        let t = last_added_sample_on_segment / current_line_length;
        let sample = from + (to - from) * t;
        samples.push(Complex {
            re: sample.x,
            im: sample.y,
        });
        *sample_index += 1;
    }

    // Add additional samples if the segment is longer than the sample interval
    let mut compensation_counter = 0;
    while sample_length * (*sample_index as f32) <= *iterated_length + current_line_length {
        let offset = (sample_length * compensation_counter as f32
            + last_added_sample_on_segment
            + sample_length)
            / current_line_length;
        let sample = from + (to - from) * offset;
        samples.push(Complex {
            re: sample.x,
            im: sample.y,
        });
        *sample_index += 1;
        compensation_counter += 1;
    }

    // Update the total iterated length
    *iterated_length += current_line_length;
}

/// Converts a path to Fourier coefficients using FFT.
///
/// This function:
/// 1. Computes the path length
/// 2. Samples the path at `n_sample` evenly-spaced points
/// 3. Applies FFT to compute the frequency components
/// 4. Normalizes the coefficients
///
/// # Arguments
///
/// * `path` - The lyon `Path` to transform
/// * `n_sample` - The number of sample points (also the FFT size)
///
/// # Returns
///
/// A vector of `Complex<f32>` Fourier coefficients, normalized by the sample count.
/// The coefficient at index `k` corresponds to frequency `k` (or `k - n_sample` for
/// negative frequencies when `k > n_sample/2`).
pub fn path_to_fft(path: Path, n_sample: usize) -> Vec<Complex<f32>> {
    let path_length = compute_path_length(&path);
    let mut samples = construct_sample_points(&path, path_length, n_sample);

    // Ensure we have exactly n_sample points (truncate if necessary)
    samples.truncate(n_sample);

    // Perform FFT
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n_sample);
    fft.process(&mut samples);

    // Normalize coefficients by sample count
    let scale = 1.0 / samples.len() as f32;
    for sample in &mut samples {
        *sample *= scale;
    }

    samples
}
