use rustfft::num_complex::Complex;

#[derive(Clone, Debug)]
pub struct DrawData {
    frequency: f32,
    radius: f32,
    angle: f32,
}

impl DrawData {
    pub fn new(f: f32, r: f32, a: f32) -> DrawData {
        DrawData {
            frequency: f,
            radius: r,
            angle: a,
        }
    }

    pub fn new_from_complex(f: f32, c: Complex<f32>) -> DrawData {
        let (r, a) = c.to_polar();
        DrawData {
            frequency: f,
            radius: r,
            angle: a,
        }
    }
}
