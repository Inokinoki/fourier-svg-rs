//! GPUI Fourier Visualizer - Native GPU-accelerated GUI Application
//!
//! Features:
//! - Interactive drawing canvas
//! - SVG file loading with file dialog
//! - Multi-layer support with visibility toggle and removal
//! - Adjustable sampling and wave configuration
//! - Export to HTML, GIF, JSON
//! - Undo last drawing

use fourier_svg::{
    combine_layers, process_multiple_paths, process_svg_path, DrawData, FourierCoefficient,
    FourierConfig, FourierExport, GIFVisualizer, HTMLVisualizer, PathLayer, Visualizer,
};
use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, MouseButton,
    MouseDownEvent, MouseMoveEvent, PathPromptOptions, Pixels, Point, Render, SharedString,
    Window, WindowBounds, WindowOptions,
};

struct FourierApp {
    points: Vec<Point<Pixels>>,
    painting: bool,
    layers: Vec<PathLayer>,
    selected_layer: Option<usize>,
    status: SharedString,
    num_sample: usize,
    num_wave: usize,
}

impl FourierApp {
    fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        let _ = cx;
        Self {
            points: vec![],
            painting: false,
            layers: vec![],
            selected_layer: None,
            status: "Draw on canvas or load SVG file".into(),
            num_sample: FourierConfig::default().num_sample,
            num_wave: FourierConfig::default().num_wave,
        }
    }

    fn config(&self) -> FourierConfig {
        FourierConfig::new(self.num_sample, self.num_wave)
    }

    fn clear_canvas(&mut self, cx: &mut Context<Self>) {
        self.points.clear();
        self.painting = false;
        self.layers.clear();
        self.selected_layer = None;
        self.status = "Canvas cleared".into();
        cx.notify();
    }

    fn undo_last_layer(&mut self, cx: &mut Context<Self>) {
        if self.layers.pop().is_some() {
            self.selected_layer = if self.layers.is_empty() {
                None
            } else {
                Some(self.layers.len() - 1)
            };
            self.status = "Removed last layer".into();
        } else {
            self.status = "No layers to remove".into();
        }
        cx.notify();
    }

    fn add_drawing_as_layer(&mut self) {
        if self.points.len() < 2 {
            self.status = "Not enough points to create a path".into();
            return;
        }

        let mut path_string = String::new();
        for (i, p) in self.points.iter().enumerate() {
            let x: f32 = p.x.into();
            let y: f32 = p.y.into();
            if i == 0 {
                path_string.push_str(&format!("M {} {} ", x, y));
            } else {
                path_string.push_str(&format!("L {} {} ", x, y));
            }
        }
        path_string.push_str(" Z");

        let config = self.config();
        let fourier_data = process_svg_path(&path_string, &config);

        self.layers.push(PathLayer {
            id: format!("Drawing {}", self.layers.len() + 1),
            path_data: path_string,
            fourier_data,
            visible: true,
            opacity: 1.0,
        });

        self.selected_layer = Some(self.layers.len() - 1);
        self.status = format!(
            "Added layer with {} coefficients",
            self.layers.last().unwrap().fourier_data.len()
        )
        .into();
    }

    fn load_svg_dialog(&mut self, cx: &mut Context<Self>) {
        let options = PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open SVG File".into()),
        };

        let rx = cx.prompt_for_paths(options);
        let this = cx.weak_entity();
        let app: &App = cx;
        app.spawn(async move |cx| {
            match rx.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.first() {
                        let file_path = path.to_string_lossy().to_string();
                        this.update(cx, |this: &mut FourierApp, cx: &mut Context<FourierApp>| {
                            this.load_svg_as_layers(&file_path, cx);
                        })?;
                    }
                }
                Ok(Ok(None)) => {
                    this.update(cx, |this: &mut FourierApp, _cx: &mut Context<FourierApp>| {
                        this.status = "File selection cancelled".into();
                    })?;
                }
                Ok(Err(e)) => {
                    this.update(cx, |this: &mut FourierApp, _cx: &mut Context<FourierApp>| {
                        this.status = format!("Dialog error: {}", e).into();
                    })?;
                }
                Err(_) => {}
            }
            anyhow::Ok(())
        }).detach();
    }

    fn load_svg_as_layers(&mut self, file_path: &str, cx: &mut Context<Self>) {
        let config = self.config();
        match process_multiple_paths(file_path, &config) {
            Ok(mut new_layers) => {
                for (i, layer) in new_layers.iter_mut().enumerate() {
                    layer.id = format!("{} (Layer {})", layer.id, self.layers.len() + i + 1);
                }
                let count = new_layers.len();
                let total_coeffs: usize = new_layers.iter().map(|l| l.fourier_data.len()).sum();
                self.layers.extend(new_layers);
                if !self.layers.is_empty() {
                    self.selected_layer = Some(self.layers.len() - 1);
                }
                self.status = format!(
                    "Loaded {} path(s) with {} coefficients",
                    count, total_coeffs
                )
                .into();
            }
            Err(e) => {
                self.status = format!("Failed to load SVG: {}", e).into();
            }
        }
        cx.notify();
    }

    fn toggle_layer_visibility(&mut self, layer_idx: usize, cx: &mut Context<Self>) {
        if layer_idx < self.layers.len() {
            self.layers[layer_idx].visible = !self.layers[layer_idx].visible;
            self.status = format!(
                "Layer {} visibility: {}",
                layer_idx + 1,
                if self.layers[layer_idx].visible {
                    "ON"
                } else {
                    "OFF"
                }
            )
            .into();
            cx.notify();
        }
    }

    fn remove_layer(&mut self, layer_idx: usize, cx: &mut Context<Self>) {
        if layer_idx < self.layers.len() {
            self.layers.remove(layer_idx);
            self.selected_layer = if self.layers.is_empty() {
                None
            } else {
                Some(self.layers.len().saturating_sub(1))
            };
            self.status = format!("Removed layer {}", layer_idx + 1).into();
            cx.notify();
        }
    }

    fn get_combined_data(&self) -> Vec<DrawData> {
        combine_layers(&self.layers)
    }

    fn export_html(&self, path: &str) -> bool {
        let data = self.get_combined_data();
        if data.is_empty() {
            return false;
        }
        let visualizer = HTMLVisualizer::new(path.to_string());
        visualizer.render(data)
    }

    fn export_gif(&self, path: &str, frames: usize) -> bool {
        let data = self.get_combined_data();
        if data.is_empty() {
            return false;
        }
        let visualizer = GIFVisualizer::new(path.to_string()).with_frames(frames);
        visualizer.render(data)
    }

    fn export_json(&self, path: &str) -> bool {
        let data = self.get_combined_data();
        if data.is_empty() {
            return false;
        }
        let export = FourierExport {
            version: "1.0".to_string(),
            metadata: fourier_svg::ExportMetadata {
                svg_path: None,
                sample_count: self.num_sample,
                wave_count: data.len(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            data: data
                .iter()
                .map(|d| FourierCoefficient {
                    frequency: d.frequency,
                    radius: d.radius,
                    angle: d.angle,
                })
                .collect(),
        };

        match serde_json::to_string_pretty(&export) {
            Ok(json_str) => std::fs::write(path, json_str).is_ok(),
            Err(_) => false,
        }
    }

    fn update_sample_count(&mut self, sample: usize, cx: &mut Context<Self>) {
        self.num_sample = sample.max(64);
        self.status = format!("Sample count: {}", self.num_sample).into();
        cx.notify();
    }

    fn update_wave_count(&mut self, wave: usize, cx: &mut Context<Self>) {
        self.num_wave = wave.max(4);
        self.status = format!("Wave count: {}", self.num_wave).into();
        cx.notify();
    }
}

impl Render for FourierApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status.clone();
        let layer_count = self.layers.len();
        let total_coeffs: usize = self.layers.iter().map(|l| l.fourier_data.len()).sum();

        div()
            .size_full()
            .bg(rgb(0x1a1a2e))
            .flex()
            .flex_col()
            // Top toolbar
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .bg(rgb(0x16213e))
                    .px_4()
                    .py_2()
                    .child(
                        div().text_color(rgb(0xe94560)).text_xl().child("Fourier SVG (GPUI)"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .id("load-svg")
                                    .child("Load SVG")
                                    .bg(rgb(0x0f3460))
                                    .text_color(rgb(0xffffff))
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.load_svg_dialog(cx);
                                    })),
                            )
                            .child(
                                div()
                                    .id("export-html")
                                    .child("HTML")
                                    .bg(rgb(0x533483))
                                    .text_color(rgb(0xffffff))
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.export_html("output.html") {
                                            this.status = "Exported output.html".into();
                                        } else {
                                            this.status = "No data to export".into();
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .id("export-gif")
                                    .child("GIF")
                                    .bg(rgb(0x533483))
                                    .text_color(rgb(0xffffff))
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.export_gif("output.gif", 100) {
                                            this.status = "Exported output.gif".into();
                                        } else {
                                            this.status = "No data to export".into();
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .id("export-json")
                                    .child("JSON")
                                    .bg(rgb(0x533483))
                                    .text_color(rgb(0xffffff))
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.export_json("output.json") {
                                            this.status = "Exported output.json".into();
                                        } else {
                                            this.status = "No data to export".into();
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .id("undo-btn")
                                    .child("Undo")
                                    .bg(rgb(0x6c757d))
                                    .text_color(rgb(0xffffff))
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.undo_last_layer(cx);
                                    })),
                            )
                            .child(
                                div()
                                    .id("clear-btn")
                                    .child("Clear")
                                    .bg(rgb(0xe94560))
                                    .text_color(rgb(0xffffff))
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear_canvas(cx);
                                    })),
                            ),
                    ),
            )
            // Config bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .bg(rgb(0x0f3460))
                    .px_4()
                    .py_1()
                    .child(
                        div().text_color(rgb(0xa0a0a0)).text_xs().child(format!(
                            "Samples: {}",
                            self.num_sample
                        )),
                    )
                    .child(
                        div()
                            .id("sample-up")
                            .text_color(rgb(0xffffff))
                            .text_xs()
                            .child("[+]")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update_sample_count(this.num_sample * 2, cx);
                            })),
                    )
                    .child(
                        div()
                            .id("sample-down")
                            .text_color(rgb(0xffffff))
                            .text_xs()
                            .child("[-]")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update_sample_count(this.num_sample / 2, cx);
                            })),
                    )
                    .child(
                        div().text_color(rgb(0xa0a0a0)).text_xs().child(format!(
                            "Waves: {}",
                            self.num_wave
                        )),
                    )
                    .child(
                        div()
                            .id("wave-up")
                            .text_color(rgb(0xffffff))
                            .text_xs()
                            .child("[+]")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update_wave_count(this.num_wave + 20, cx);
                            })),
                    )
                    .child(
                        div()
                            .id("wave-down")
                            .text_color(rgb(0xffffff))
                            .text_xs()
                            .child("[-]")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update_wave_count(this.num_wave.saturating_sub(20), cx);
                            })),
                    ),
            )
            // Main content area
            .child(
                div()
                    .id("main-content")
                    .flex()
                    .flex_1()
                    .min_h_0()
                    // Canvas
                    .child(
                        div()
                            .id("canvas-area")
                            .flex_1()
                            .m_4()
                            .bg(rgb(0xffffff))
                            .rounded(px(8.0))
                            .relative()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                                    this.painting = true;
                                    this.points.clear();
                                    this.points.push(ev.position);
                                    cx.notify();
                                }),
                            )
                            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _, cx| {
                                if !this.painting {
                                    return;
                                }
                                this.points.push(ev.position);
                                cx.notify();
                            }))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.painting = false;
                                    this.add_drawing_as_layer();
                                    cx.notify();
                                }),
                            ),
                    )
                    // Layers panel
                    .child(
                        div()
                            .id("layers-panel")
                            .w(px(220.0))
                            .m_4()
                            .ml_0()
                            .bg(rgb(0x16213e))
                            .rounded(px(8.0))
                            .p_4()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_color(rgb(0xe94560))
                                    .text_sm()
                                    .mb_2()
                                    .child(format!(
                                        "Layers ({}) - {} coeffs",
                                        layer_count, total_coeffs
                                    )),
                            )
                            .child(
                                div()
                                    .id("layers-list")
                                    .flex_1()
                                    .overflow_hidden()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .children(self.layers.iter().enumerate().map(|(i, layer)| {
                                        let visible = layer.visible;
                                        let selected = Some(i) == self.selected_layer;
                                        let coeff_count = layer.fourier_data.len();
                                        div()
                                            .id(("layer", i))
                                            .flex()
                                            .justify_between()
                                            .items_center()
                                            .p_1()
                                            .bg(if selected {
                                                rgb(0x0f3460)
                                            } else {
                                                rgb(0x1a1a2e)
                                            })
                                            .rounded(px(4.0))
                                            .child(
                                                div()
                                                    .id(("select-layer", i))
                                                    .flex()
                                                    .flex_col()
                                                    .cursor_pointer()
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            this.selected_layer = Some(i);
                                                            cx.notify();
                                                        },
                                                    ))
                                                    .child(
                                                        div()
                                                            .text_color(rgb(0xffffff))
                                                            .text_xs()
                                                            .child(format!(
                                                                "{} {}",
                                                                if visible { "[V]" } else { "[H]" },
                                                                layer.id
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_color(rgb(0x888888))
                                                            .text_xs()
                                                            .child(format!(
                                                                "{} coefficients",
                                                                coeff_count
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .id(("toggle", i))
                                                            .text_color(rgb(0xa0a0a0))
                                                            .text_xs()
                                                            .child(if visible { "Hide" } else { "Show" })
                                                            .cursor_pointer()
                                                            .on_click(cx.listener(
                                                                move |this, _, _, cx| {
                                                                    this.toggle_layer_visibility(
                                                                        i, cx,
                                                                    );
                                                                },
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .id(("remove", i))
                                                            .text_color(rgb(0xe94560))
                                                            .text_xs()
                                                            .child("X")
                                                            .cursor_pointer()
                                                            .on_click(cx.listener(
                                                                move |this, _, _, cx| {
                                                                    this.remove_layer(i, cx);
                                                                },
                                                            )),
                                                    ),
                                            )
                                    })),
                            ),
                    ),
            )
            // Status bar
            .child(
                div()
                    .bg(rgb(0x0f3460))
                    .px_4()
                    .py_2()
                    .child(
                        div().text_color(rgb(0xa0a0a0)).text_xs().child(format!(
                            "{} | Layers: {} | Coefficients: {} | Config: samples={}, waves={}",
                            status, layer_count, total_coeffs, self.num_sample, self.num_wave
                        )),
                    ),
            )
    }
}

pub fn run_gpui_app() {
    Application::new().run(|cx| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1200.0), px(800.0)),
                })),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| FourierApp::new(window, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}

fn main() {
    run_gpui_app();
}
