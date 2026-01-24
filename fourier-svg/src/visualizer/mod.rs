pub mod export_visualizer;
pub mod gif_visualizer;
pub mod html_visualizer;

// Re-export commonly used types
pub use export_visualizer::{
    export_to_draw_data, load_fourier_export, ExportMetadata, FourierCoefficient, FourierExport,
};

pub trait Visualizer {
    fn render(&self, data: Vec<crate::fft_drawer::DrawData>) -> bool;
}
