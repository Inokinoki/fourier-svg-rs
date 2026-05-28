use fourier_svg::process_svg_path;
use fourier_svg::DrawData;
use fourier_svg::FourierConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FourierData {
    pub frequency: f32,
    pub radius: f32,
    pub angle: f32,
    pub idx: usize,
}

impl FourierData {
    pub fn from_draw_data(d: &DrawData, idx: usize) -> Self {
        Self {
            frequency: d.frequency,
            radius: d.radius,
            angle: d.angle,
            idx,
        }
    }

    pub fn from_draw_data_vec(data: &[DrawData]) -> Vec<Self> {
        let mut result: Vec<_> = data
            .iter()
            .enumerate()
            .map(|(idx, d)| Self::from_draw_data(d, idx))
            .collect();
        result.sort_by(|a, b| {
            b.radius
                .partial_cmp(&a.radius)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }
}

#[tauri::command]
pub fn process_drawing(path: String, num_sample: usize) -> Result<Vec<FourierData>, String> {
    let config = FourierConfig::new(num_sample, num_sample);
    let result = process_svg_path(&path, &config);

    if result.is_empty() {
        return Err("No Fourier components computed — check the SVG path".into());
    }

    Ok(FourierData::from_draw_data_vec(&result))
}
