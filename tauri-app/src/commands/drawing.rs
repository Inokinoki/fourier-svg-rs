use fourier_svg::process_svg_path;
use fourier_svg::FourierConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct FourierData {
    pub s: f32,
    pub r: f32,
    pub a: f32,
    pub idx: usize,
}

#[tauri::command]
pub fn process_drawing(path: String, num_sample: usize) -> Vec<FourierData> {
    let config = FourierConfig::new(num_sample, num_sample);
    let result = process_svg_path(&path, &config);

    // Convert to FourierData for JSON serialization, sorted by radius
    let mut sorted: Vec<_> = result
        .iter()
        .enumerate()
        .map(|(idx, d)| FourierData {
            s: d.frequency,
            r: d.radius,
            a: d.angle,
            idx,
        })
        .collect();
    sorted.sort_by(|a, b| b.r.partial_cmp(&a.r).unwrap_or(std::cmp::Ordering::Equal));

    sorted
}
