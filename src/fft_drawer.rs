//! FFT drawing data structures for Fourier visualization.
//!
//! This module provides the data structures needed to represent Fourier
//! components (epicycles) that can be used for visualization.

use rustfft::num_complex::Complex;

/// Represents a single Fourier component (epicycle) for drawing.
///
/// Each `DrawData` represents a rotating circle with a specific frequency,
/// radius (amplitude), and initial phase angle. When combined, these circles
/// trace out the original path.
#[derive(Clone, Debug)]
pub struct DrawData {
    /// The rotation frequency of this epicycle.
    /// Positive values rotate counter-clockwise, negative values clockwise.
    pub frequency: f32,

    /// The radius (amplitude) of this epicycle.
    /// Larger radii contribute more to the final shape.
    pub radius: f32,

    /// The initial phase angle in radians.
    /// Determines the starting position of the rotation.
    pub angle: f32,
}

impl DrawData {
    /// Creates a new `DrawData` with explicit frequency, radius, and angle.
    ///
    /// # Arguments
    ///
    /// * `f` - The rotation frequency
    /// * `r` - The radius (amplitude)
    /// * `a` - The initial phase angle in radians
    ///
    /// # Example
    ///
    /// ```
    /// use fourier_svg::fft_drawer::DrawData;
    /// let circle = DrawData::new(1.0, 50.0, 0.0);
    /// ```
    #[allow(dead_code)]
    pub fn new(f: f32, r: f32, a: f32) -> DrawData {
        DrawData {
            frequency: f,
            radius: r,
            angle: a,
        }
    }

    /// Creates a new `DrawData` from a frequency and a complex Fourier coefficient.
    ///
    /// The complex number is converted to polar form to extract the radius
    /// (magnitude) and angle (phase).
    ///
    /// # Arguments
    ///
    /// * `f` - The rotation frequency
    /// * `c` - The complex Fourier coefficient
    ///
    /// # Example
    ///
    /// ```
    /// use fourier_svg::fft_drawer::DrawData;
    /// use rustfft::num_complex::Complex;
    ///
    /// let coefficient = Complex::new(3.0, 4.0);
    /// let circle = DrawData::new_from_complex(1.0, coefficient);
    /// assert_eq!(circle.radius, 5.0); // sqrt(3^2 + 4^2)
    /// ```
    pub fn new_from_complex(f: f32, c: Complex<f32>) -> DrawData {
        let (r, a) = c.to_polar();
        DrawData {
            frequency: f,
            radius: r,
            angle: a,
        }
    }
}
