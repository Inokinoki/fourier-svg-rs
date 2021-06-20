use crate::fft_drawer;

pub mod html_visualizer;

pub trait Visualizer {
    fn render(&self, data: Vec<fft_drawer::DrawData>) -> bool;
}
