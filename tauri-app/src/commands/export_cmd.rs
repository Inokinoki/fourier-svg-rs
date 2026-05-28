use std::fs::File;
use std::io::Write;

use fourier_svg::DrawData;
use fourier_svg::GIFVisualizer;
use fourier_svg::HTMLVisualizer;
use fourier_svg::Visualizer;

use super::drawing::FourierData;

fn to_draw_data_vec(data: &[FourierData]) -> Vec<DrawData> {
    data.iter()
        .map(|d| DrawData {
            frequency: d.frequency,
            radius: d.radius,
            angle: d.angle,
        })
        .collect()
}

#[tauri::command]
pub async fn export_fourier_data(
    data: Vec<FourierData>,
    file_path: String,
    num_samples: usize,
) -> Result<(), String> {
    use fourier_svg::FourierExport;

    let draw_data = to_draw_data_vec(&data);

    let export = FourierExport {
        version: "1.0".to_string(),
        metadata: fourier_svg::ExportMetadata {
            svg_path: None,
            sample_count: num_samples,
            wave_count: draw_data.len(),
            timestamp: chrono::Utc::now().timestamp(),
        },
        data: draw_data
            .iter()
            .map(|d| fourier_svg::FourierCoefficient {
                frequency: d.frequency,
                radius: d.radius,
                angle: d.angle,
            })
            .collect(),
    };

    let json_str = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;

    let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
    file.write_all(json_str.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn export_as_gif(
    data: Vec<FourierData>,
    file_path: String,
    frames: usize,
    duration: f32,
) -> Result<(), String> {
    let draw_data = to_draw_data_vec(&data);

    let delay = ((duration * 1000.0) / frames as f32) as u16 / 10;

    let visualizer = GIFVisualizer::new(file_path.clone())
        .with_dimensions(800, 600)
        .with_frames(frames)
        .with_delay(delay.max(1));

    if visualizer.render(draw_data) {
        Ok(())
    } else {
        Err("Failed to create GIF".to_string())
    }
}

#[tauri::command]
pub async fn export_as_html(data: Vec<FourierData>, file_path: String) -> Result<(), String> {
    let draw_data = to_draw_data_vec(&data);

    let visualizer = HTMLVisualizer::new(file_path.clone());
    if visualizer.render(draw_data) {
        Ok(())
    } else {
        Err("Failed to create HTML".to_string())
    }
}
