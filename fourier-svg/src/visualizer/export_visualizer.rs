use std::fs::File;
use std::io::BufWriter;

use crate::visualizer::Visualizer;
use crate::fft_drawer::DrawData;
use serde::{Deserialize, Serialize};

/// Fourier data export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FourierExport {
    /// Format version
    pub version: String,
    /// Metadata about the export
    pub metadata: ExportMetadata,
    /// The Fourier coefficient data
    pub data: Vec<FourierCoefficient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Original SVG path string (if available)
    pub svg_path: Option<String>,
    /// Number of sample points used for FFT
    pub sample_count: usize,
    /// Number of waves/coefficients
    pub wave_count: usize,
    /// Timestamp of export
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FourierCoefficient {
    /// Frequency index
    pub frequency: f32,
    /// Radius (amplitude)
    pub radius: f32,
    /// Initial phase angle (in radians)
    pub angle: f32,
}

pub struct ExportVisualizer {
    file_name: String,
    svg_path: Option<String>,
    sample_count: usize,
    wave_count: usize,
}

impl ExportVisualizer {
    pub fn new(file_name: String) -> ExportVisualizer {
        ExportVisualizer {
            file_name,
            svg_path: None,
            sample_count: 0,
            wave_count: 0,
        }
    }

    pub fn with_metadata(mut self, svg_path: Option<String>, sample_count: usize, wave_count: usize) -> Self {
        self.svg_path = svg_path;
        self.sample_count = sample_count;
        self.wave_count = wave_count;
        self
    }
}

impl Visualizer for ExportVisualizer {
    fn render(&self, data: Vec<DrawData>) -> bool {
        // Convert DrawData to FourierCoefficient
        let coefficients: Vec<FourierCoefficient> = data
            .iter()
            .map(|d| FourierCoefficient {
                frequency: d.frequency,
                radius: d.radius,
                angle: d.angle,
            })
            .collect();

        let export = FourierExport {
            version: "1.0".to_string(),
            metadata: ExportMetadata {
                svg_path: self.svg_path.clone(),
                sample_count: self.sample_count,
                wave_count: self.wave_count,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            },
            data: coefficients,
        };

        // Write to JSON file
        match File::create(&self.file_name) {
            Ok(file) => {
                let writer = BufWriter::new(file);
                match serde_json::to_writer_pretty(writer, &export) {
                    Ok(_) => {
                        println!("Fourier data exported to {}", self.file_name);
                        true
                    }
                    Err(e) => {
                        eprintln!("Failed to write JSON: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create file: {}", e);
                false
            }
        }
    }
}

/// Load Fourier data from a JSON export file
pub fn load_fourier_export(path: &str) -> Result<FourierExport, String> {
    match File::open(path) {
        Ok(file) => {
            match serde_json::from_reader(file) {
                Ok(export) => Ok(export),
                Err(e) => Err(format!("Failed to parse JSON: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to open file: {}", e)),
    }
}

/// Convert FourierExport back to DrawData for rendering
pub fn export_to_draw_data(export: &FourierExport) -> Vec<DrawData> {
    export.data
        .iter()
        .map(|c| DrawData {
            frequency: c.frequency,
            radius: c.radius,
            angle: c.angle,
        })
        .collect()
}
