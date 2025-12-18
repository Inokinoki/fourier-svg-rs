//! Visualization backends for Fourier drawing.
//!
//! This module provides traits and implementations for rendering Fourier
//! transform results. Different visualizers can output to various formats
//! such as HTML canvas, SVG, or other graphics formats.

use crate::fft_drawer;

pub mod html_visualizer;

/// A trait for rendering Fourier visualization data.
///
/// Implementors of this trait can render the Fourier components (epicycles)
/// to various output formats.
pub trait Visualizer {
    /// Renders the Fourier components to the visualizer's output format.
    ///
    /// # Arguments
    ///
    /// * `data` - A vector of `DrawData` representing the Fourier components
    ///
    /// # Returns
    ///
    /// `true` if rendering succeeded, `false` otherwise.
    fn render(&self, data: Vec<fft_drawer::DrawData>) -> bool;
}
