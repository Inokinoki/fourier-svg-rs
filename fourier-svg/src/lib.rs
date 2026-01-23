//! Fourier SVG - Draw SVG paths using Fourier Transform
//!
//! This library provides core functionality for:
//! - Parsing SVG paths
//! - Computing Fourier transforms
//! - Rendering visualizations (HTML, GIF, export)
//! - Loading/saving Fourier data

pub mod fft_drawer;
pub mod path_util;
pub mod visualizer;

// Re-export commonly used types
pub use fft_drawer::DrawData;
pub use path_util::{build_path_from_svg, path_to_fft};
pub use visualizer::{
    export_visualizer::ExportVisualizer,
    export_visualizer::{
        export_to_draw_data, load_fourier_export, ExportMetadata, FourierCoefficient, FourierExport,
    },
    gif_visualizer::GIFVisualizer,
    html_visualizer::HTMLVisualizer,
    Visualizer,
};
