use std::fs::File;
use std::io::BufWriter;

use crate::fft_drawer::DrawData;
use crate::visualizer::Visualizer;
use gif::{Encoder, Frame, Repeat};
use image::{Rgb, RgbImage};

pub struct GIFVisualizer {
    file_name: String,
    width: u16,
    height: u16,
    frames: usize,
    delay: u16,
}

impl GIFVisualizer {
    pub fn new(file_name: String) -> GIFVisualizer {
        GIFVisualizer {
            file_name,
            width: 800,
            height: 600,
            frames: 100,
            delay: 2, // 20ms per frame
        }
    }

    pub fn with_dimensions(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_frames(mut self, frames: usize) -> Self {
        self.frames = frames;
        self
    }

    pub fn with_delay(mut self, delay: u16) -> Self {
        self.delay = delay;
        self
    }

    fn calculate_position(&self, data: &[DrawData], time: f32, idx: usize) -> (f32, f32) {
        let mut x = self.width as f32 / 4.0;
        let mut y = self.height as f32 / 2.0;

        for (i, d) in data.iter().enumerate() {
            let angle = d.angle + 2.0 * std::f32::consts::PI * time * (d.frequency / 20.0);
            let radius = d.radius / 2.0;

            x += radius * angle.cos();
            y += radius * angle.sin();

            // Only draw up to current index in partial trace
            if i >= idx {
                break;
            }
        }

        (x, y)
    }

    fn render_frame(&self, data: &[DrawData], time: f32, wave: &[(f32, f32)]) -> Vec<u8> {
        let mut img = RgbImage::new(self.width as u32, self.height as u32);
        let white = Rgb([255, 255, 255]);
        let black = Rgb([0, 0, 0]);
        let orange = Rgb([202, 126, 86]);

        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = white;
        }

        // Draw wave trace
        for i in 1..wave.len() {
            let x0 = wave[i - 1].0 as i32;
            let y0 = wave[i - 1].1 as i32;
            let x1 = wave[i].0 as i32;
            let y1 = wave[i].1 as i32;

            self.draw_line(&mut img, x0, y0, x1, y1, black);
        }

        // Draw circles
        let mut x = self.width as f32 / 4.0;
        let mut y = self.height as f32 / 2.0;

        for d in data {
            let angle = d.angle + 2.0 * std::f32::consts::PI * time * (d.frequency / 20.0);
            let radius = d.radius / 2.0;

            let new_x = x + radius * angle.cos();
            let new_y = y + radius * angle.sin();

            // Draw line from center to edge
            self.draw_line(
                &mut img,
                x as i32,
                y as i32,
                new_x as i32,
                new_y as i32,
                orange,
            );

            x = new_x;
            y = new_y;
        }

        img.into_raw()
    }

    fn draw_line(&self, img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>) {
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if x0 >= 0 && y0 >= 0 && x0 < self.width as i32 && y0 < self.height as i32 {
                img.put_pixel(x0 as u32, y0 as u32, color);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}

impl Visualizer for GIFVisualizer {
    fn render(&self, data: Vec<DrawData>) -> bool {
        let file = match File::create(&self.file_name) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to create file: {}", e);
                return false;
            }
        };

        let writer = BufWriter::new(file);
        let mut encoder = match Encoder::new(writer, self.width, self.height, &[]) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to create encoder: {}", e);
                return false;
            }
        };

        // Set repeat to infinite
        if let Err(e) = encoder.set_repeat(Repeat::Infinite) {
            eprintln!("Failed to set repeat: {}", e);
            return false;
        }

        let time_step = 0.04;
        let mut wave = Vec::new();

        for frame in 0..self.frames {
            let time = frame as f32 * time_step;

            // Calculate current position and add to wave
            let pos = self.calculate_position(&data, time, data.len());
            wave.push(pos);

            // Limit wave length
            if wave.len() > 400 {
                wave.remove(0);
            }

            // Render frame
            let frame_data = self.render_frame(&data, time, &wave);

            let gif_frame = Frame::from_rgb(self.width, self.height, &frame_data);
            if let Err(e) = encoder.write_frame(&gif_frame) {
                eprintln!("Failed to write frame {}: {}", frame, e);
                return false;
            }
        }

        // Encoder auto-flushes on drop
        println!("GIF saved to {}", self.file_name);
        true
    }
}
