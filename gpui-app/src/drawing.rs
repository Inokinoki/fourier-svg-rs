//! gpui Fourier Visualizer - Interactive Drawing Application

use gpui::{
    canvas, div, prelude::*, px, rgb, size, App, Application, Bounds, Context, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Path, PathBuilder, Pixels, Point, Render, StrokeOptions, Window,
    WindowBounds, WindowOptions,
};

struct DrawingViewer {
    points: Vec<Point<Pixels>>,
    painting: bool,
}

impl DrawingViewer {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            points: vec![],
            painting: false,
        }
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.points.clear();
        self.painting = false;
        cx.notify();
    }
}

impl Render for DrawingViewer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let points = self.points.clone();

        div()
            .font_size(px(16.0))
            .bg(rgb(0x1a1a2e))
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_color(rgb(0xe94560))
                            .text_xl()
                            .child("Fourier Drawing Canvas"),
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
                            .active(|this| this.opacity(0.8))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .mt_4()
                    .text_color(rgb(0xa0a0a0))
                    .child(format!("Points: {}", points.len())),
            )
            .child(
                div()
                    .size_full()
                    .mt_4()
                    .bg(rgb(0xffffff))
                    .border_1()
                    .border_color(rgb(0x404040))
                    .child(
                        canvas(
                            move |_, _, _| {},
                            move |_, _, window, _| {
                                if points.len() < 2 {
                                    return;
                                }

                                let mut builder = PathBuilder::stroke(px(2.0));
                                for (i, p) in points.iter().enumerate() {
                                    if i == 0 {
                                        builder.move_to(*p);
                                    } else {
                                        builder.line_to(*p);
                                    }
                                }

                                if let Ok(path) = builder.build() {
                                    window.paint_path(path, rgb(0xe94560).into());
                                }
                            },
                        )
                        .size_full(),
                    )
                    .on_mouse_down(
                        gpui::MouseButton::Left,
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
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.painting = false;
                            cx.notify();
                        }),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.0), px(700.0)),
                })),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| DrawingViewer::new(window, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
