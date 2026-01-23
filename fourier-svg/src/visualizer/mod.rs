pub mod html_visualizer;
pub mod gif_visualizer;
pub mod export_visualizer;

// Re-export commonly used types
pub use export_visualizer::{FourierExport, ExportMetadata, FourierCoefficient, load_fourier_export, export_to_draw_data};

pub trait Visualizer {
    fn render(&self, data: Vec<crate::fft_drawer::DrawData>) -> bool;
}
