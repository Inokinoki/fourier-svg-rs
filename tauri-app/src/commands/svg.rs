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

    // Read and parse the SVG file
    match svg::open(&file_path, &mut content) {
        Ok(parser) => {
            let mut path_index = 0;
            for event in parser {
                match event {
                    svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) => {
                        if let Some(d) = attributes.get("d") {
                            paths.push(SvgPathInfo {
                                id: if let Some(id) = attributes.get("id") {
                                    id.to_string()
                                } else {
                                    format!("path_{}", path_index)
                                },
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

/// Parse SVG dimension attribute to float
fn parse_svg_dimension(value: &str) -> Option<f32> {
    let value = value
        .trim()
        .trim_end_matches("px")
        .trim_end_matches("pt")
        .trim_end_matches("%");
    value.parse::<f32>().ok()
}

#[tauri::command]
pub fn process_svg_path(path_data: String, num_sample: usize) -> Vec<FourierData> {
    let config = FourierConfig::new(num_sample, num_sample);
    let result = fft_process_svg_path(&path_data, &config);

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

#[tauri::command]
pub fn get_svg_paths(file_path: String) -> Result<Vec<SvgPathInfo>, String> {
    let mut content = String::new();
    let mut paths = Vec::new();

    match svg::open(&file_path, &mut content) {
        Ok(parser) => {
            let mut path_index = 0;
            for event in parser {
                if let svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) = event
                {
                    if let Some(d) = attributes.get("d") {
                        paths.push(SvgPathInfo {
                            id: if let Some(id) = attributes.get("id") {
                                id.to_string()
                            } else {
                                format!("path_{}", path_index)
                            },
                            d: d.to_string(),
                        });
                        path_index += 1;
                    }
                }
            }
            Ok(paths)
        }
        Err(e) => Err(format!("Failed to parse SVG: {}", e)),
    }
}
