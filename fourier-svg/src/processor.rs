//! Fourier Processor - Common processing logic for all applications
//!
//! This module provides unified SVG/Fourier processing functionality
//! that can be shared between CLI, Tauri, and GPUI applications.

use crate::{build_path_from_svg, path_to_fft, DrawData};
use rustfft::num_complex::Complex;

/// Configuration for Fourier processing
#[derive(Debug, Clone)]
pub struct FourierConfig {
    /// Number of sample points (default: 10240)
    pub num_sample: usize,
    /// Number of wave components (default: 201)
    pub num_wave: usize,
}

impl Default for FourierConfig {
    fn default() -> Self {
        Self {
            num_sample: 10240,
            num_wave: 201,
        }
    }
}

impl FourierConfig {
    pub fn new(num_sample: usize, num_wave: usize) -> Self {
        Self {
            num_sample,
            num_wave,
        }
    }

    /// Validate and adjust config to ensure num_sample >= num_wave
    pub fn validate(&mut self) {
        if self.num_sample < self.num_wave {
            self.num_wave = self.num_sample;
        }
    }
}

/// Process SVG path string into Fourier coefficients
pub fn process_svg_path(svg_path: &str, config: &FourierConfig) -> Vec<DrawData> {
    let mut config = config.clone();
    config.validate();

    let path = build_path_from_svg(svg_path);
    let fft_result = path_to_fft(path, config.num_sample);

    build_draw_data_from_fft(&fft_result, config.num_wave)
}

/// Build DrawData from FFT result
pub fn build_draw_data_from_fft(fft_result: &[Complex<f32>], num_wave: usize) -> Vec<DrawData> {
    let mut result = Vec::new();

    // DC component (frequency 0)
    if !fft_result.is_empty() {
        result.push(DrawData::new_from_complex(0.0, fft_result[0]));
    }

    // Positive and negative frequency pairs
    for i in 1..num_wave.div_ceil(2) {
        if i < fft_result.len() {
            result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
        }
        if fft_result.len() > i {
            result.push(DrawData::new_from_complex(
                (0 - i as i32) as f32,
                fft_result[fft_result.len() - i],
            ));
        }
    }

    result
}

/// Extract the first path from an SVG file
pub fn extract_first_path_from_file(file_path: &str) -> Result<String, String> {
    let mut content = String::new();

    match svg::open(file_path, &mut content) {
        Ok(doc) => {
            for event in doc {
                if let svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) = event
                {
                    if let Some(d) = attributes.get("d") {
                        return Ok(d.to_string());
                    }
                }
            }
            Err("No path found in SVG file".to_string())
        }
        Err(e) => Err(format!("Failed to open SVG file: {}", e)),
    }
}

/// Extract all paths from an SVG file
pub fn extract_all_paths_from_file(file_path: &str) -> Result<Vec<(String, String)>, String> {
    let mut content = String::new();
    let mut paths = Vec::new();

    match svg::open(file_path, &mut content) {
        Ok(doc) => {
            let mut path_index = 0;
            for event in doc {
                if let svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) = event
                {
                    if let Some(d) = attributes.get("d") {
                        let id = attributes
                            .get("id")
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| format!("path_{}", path_index));
                        paths.push((id, d.to_string()));
                        path_index += 1;
                    }
                }
            }
            Ok(paths)
        }
        Err(e) => Err(format!("Failed to open SVG file: {}", e)),
    }
}

/// Represents a processed path with its Fourier data
#[derive(Debug, Clone)]
pub struct PathLayer {
    pub id: String,
    pub path_data: String,
    pub fourier_data: Vec<DrawData>,
    pub visible: bool,
    pub opacity: f32,
}

/// Process multiple paths from an SVG file
pub fn process_multiple_paths(
    file_path: &str,
    config: &FourierConfig,
) -> Result<Vec<PathLayer>, String> {
    let paths = extract_all_paths_from_file(file_path)?;
    let mut layers = Vec::new();

    for (id, path_data) in paths {
        let fourier_data = process_svg_path(&path_data, config);
        layers.push(PathLayer {
            id,
            path_data,
            fourier_data,
            visible: true,
            opacity: 1.0,
        });
    }

    Ok(layers)
}

/// Combine multiple path layers into a single Fourier visualization
///
/// Each path was processed with its own independent FFT.
/// This function merges the coefficients for combined visualization.
/// Opacity is applied to each layer's coefficient radii.
pub fn combine_layers(layers: &[PathLayer]) -> Vec<DrawData> {
    let mut combined = Vec::new();

    for layer in layers {
        if layer.visible {
            for draw_data in &layer.fourier_data {
                combined.push(DrawData {
                    frequency: draw_data.frequency,
                    radius: draw_data.radius * layer.opacity,
                    angle: draw_data.angle,
                });
            }
        }
    }

    // Sort by radius (largest first) for better visualization
    combined.sort_by(|a, b| {
        b.radius
            .partial_cmp(&a.radius)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    combined
}

/// Get Fourier data from various sources
pub enum FourierSource<'a> {
    /// SVG path string
    SvgPath(&'a str),
    /// SVG file path
    SvgFile(&'a str),
    /// Exported Fourier JSON file
    FourierJson(&'a str),
}

/// Main entry point: process any source into Fourier data
pub fn process_source(
    source: FourierSource,
    config: &FourierConfig,
) -> Result<Vec<DrawData>, String> {
    match source {
        FourierSource::SvgPath(path_str) => Ok(process_svg_path(path_str, config)),
        FourierSource::SvgFile(file_path) => {
            let svg_path = extract_first_path_from_file(file_path)?;
            Ok(process_svg_path(&svg_path, config))
        }
        FourierSource::FourierJson(json_path) => match crate::load_fourier_export(json_path) {
            Ok(export) => Ok(crate::export_to_draw_data(&export)),
            Err(e) => Err(format!("Failed to load Fourier data: {}", e)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = FourierConfig {
            num_sample: 100,
            num_wave: 200,
        };
        config.validate();
        assert_eq!(config.num_wave, 100);
    }

    #[test]
    fn test_process_simple_path() {
        let config = FourierConfig::new(1024, 51);
        let result = process_svg_path("M 0 0 L 10 10 L 20 0 Z", &config);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_each_path_gets_independent_fft() {
        // Create two different paths
        let path1 = "M 0 0 L 10 10 L 20 0 Z";
        let path2 = "M 50 50 L 60 60 L 70 50 Z";

        let config = FourierConfig::new(1024, 51);

        // Process each path independently
        let fft1 = process_svg_path(path1, &config);
        let fft2 = process_svg_path(path2, &config);

        // They should have different Fourier coefficients
        // (different positions = different DC component at least)
        assert_eq!(fft1.len(), fft2.len());

        // DC component (index 0) should be different due to different positions
        assert!(fft1[0].radius != fft2[0].radius || fft1[0].angle != fft2[0].angle);
    }

    #[test]
    fn test_combine_layers_preserves_independence() {
        let config = FourierConfig::new(1024, 51);
        
        // Create two layers with different paths
        let layer1 = PathLayer {
            id: "layer1".to_string(),
            path_data: "M 0 0 L 10 10 L 20 0 Z".to_string(),
            fourier_data: process_svg_path("M 0 0 L 10 10 L 20 0 Z", &config),
            visible: true,
            opacity: 1.0,
        };
        
        let layer2 = PathLayer {
            id: "layer2".to_string(),
            path_data: "M 50 50 L 60 60 L 70 50 Z".to_string(),
            fourier_data: process_svg_path("M 50 50 L 60 60 L 70 50 Z", &config),
            visible: true,
            opacity: 0.5,
        };
        
        let layers = vec![layer1.clone(), layer2.clone()];
        let combined = combine_layers(&layers);
        
        // Combined should have coefficients from both layers
        assert_eq!(combined.len(), layers[0].fourier_data.len() + layers[1].fourier_data.len());
        
        // Verify total coefficient count is correct
        let expected_count = layer1.fourier_data.len() + layer2.fourier_data.len();
        assert_eq!(combined.len(), expected_count);
        
        // Verify that opacity was applied (some coefficients should have reduced radius)
        // Count coefficients that match layer2's reduced radii
        let layer2_reduced_count = combined.iter().filter(|c| {
            layer2.fourier_data.iter().any(|l2| {
                let expected = l2.radius * 0.5;
                (c.radius - expected).abs() < 0.001
            })
        }).count();
        
        // At least some coefficients from layer2 should have reduced radius
        assert!(layer2_reduced_count > 0);
    }
}
