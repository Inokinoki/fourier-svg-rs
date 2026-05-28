use fourier_svg::process_svg_path as fft_process_svg_path;
use fourier_svg::FourierConfig;
use serde::{Deserialize, Serialize};

use super::drawing::FourierData;

#[derive(Clone, Serialize, Deserialize)]
pub struct SvgPathInfo {
    pub id: String,
    pub d: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SvgPathsResponse {
    pub paths: Vec<SvgPathInfo>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[tauri::command]
pub async fn parse_svg_file(file_path: String) -> Result<SvgPathsResponse, String> {
    let mut content = String::new();
    let mut paths = Vec::new();
    let mut width: Option<f32> = None;
    let mut height: Option<f32> = None;

    match svg::open(&file_path, &mut content) {
        Ok(parser) => {
            let mut path_index = 0;
            for event in parser {
                match event {
                    svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) => {
                        if let Some(d) = attributes.get("d") {
                            paths.push(SvgPathInfo {
                                id: attributes
                                    .get("id")
                                    .map(|v| v.to_string())
                                    .unwrap_or_else(|| format!("path_{}", path_index)),
                                d: d.to_string(),
                            });
                            path_index += 1;
                        }
                    }
                    svg::parser::Event::Tag(svg::node::element::tag::SVG, _, attributes) => {
                        if let Some(w) = attributes.get("width") {
                            width = parse_svg_dimension(w);
                        }
                        if let Some(h) = attributes.get("height") {
                            height = parse_svg_dimension(h);
                        }
                    }
                    _ => {}
                }
            }

            Ok(SvgPathsResponse {
                paths,
                width,
                height,
            })
        }
        Err(e) => Err(format!("Failed to parse SVG: {}", e)),
    }
}

fn parse_svg_dimension(value: &str) -> Option<f32> {
    let value = value
        .trim()
        .trim_end_matches("px")
        .trim_end_matches("pt")
        .trim_end_matches("%");
    value.parse::<f32>().ok()
}

#[tauri::command]
pub fn process_svg_path(path_data: String, num_sample: usize) -> Result<Vec<FourierData>, String> {
    let config = FourierConfig::new(num_sample, num_sample);
    let result = fft_process_svg_path(&path_data, &config);

    if result.is_empty() {
        return Err("No Fourier components computed — check the SVG path".into());
    }

    Ok(FourierData::from_draw_data_vec(&result))
}
