//! Tauri Fourier Visualizer Application
//!
//! A standalone Tauri application for visualizing Fourier epicycles.
//! Allows users to draw SVG paths interactively.
//!
//! Note: On Linux, you need to install GTK development libraries:
//!   Ubuntu/Debian: sudo apt-get install libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
//!   Fedora: sudo dnf install webkit2gtk3-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel
//!   Arch: sudo pacman -S webkit2gtk gtk3 libappindicator-gtk3 librsvg
//!
//! Run with: cargo run --bin tauri-app --features tauri-app

use fourier_svg::{DrawData, build_path_from_svg, path_to_fft, load_fourier_export, export_to_draw_data};
use clap::Parser;

#[cfg(feature = "tauri-app")]
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg(feature = "tauri-app")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "tauri-app")]
#[derive(Clone, Serialize, Deserialize)]
struct FourierData {
    s: f32,
    r: f32,
    a: f32,
}

/// Tauri Fourier Visualizer - Draw SVG paths using Fourier Transform
#[derive(Parser, Debug)]
#[command(author = "Inoki <veyx.shaw@gmail.com>", version = "1.0.0", about)]
struct Args {
    /// Draw an SVG path in string
    #[arg(short = 'p', long = "path")]
    svg_path: Option<String>,

    /// Draw the first SVG path in file
    #[arg(short = 'f', long = "file")]
    svg_file: Option<String>,

    /// Load from exported Fourier data JSON file
    #[arg(short = 'i', long = "input")]
    input_file: Option<String>,

    /// Use how many sample points to draw the path
    #[arg(short = 's', long = "sample", default_value = "10240")]
    num_sample: usize,

    /// Use how many waves to draw the path
    #[arg(short = 'w', long = "wave", default_value = "201")]
    num_wave: usize,
}

#[cfg(feature = "tauri-app")]
fn generate_html(initial_data: Option<&str>) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Fourier Visualizer - Tauri</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            background-color: #f0f0f0;
            font-family: Arial, sans-serif;
        }}
        .container {{
            position: relative;
        }}
        canvas {{
            background-color: white;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            border-radius: 8px;
            cursor: crosshair;
        }}
        .controls {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: white;
            padding: 15px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            max-width: 200px;
        }}
        .controls h3 {{
            margin-top: 0;
            font-size: 16px;
            margin-bottom: 10px;
        }}
        .controls button {{
            display: block;
            width: 100%;
            margin: 5px 0;
            padding: 8px 16px;
            cursor: pointer;
            border: none;
            border-radius: 4px;
            background: #007bff;
            color: white;
            font-size: 14px;
        }}
        .controls button:hover {{
            background: #0056b3;
        }}
        .controls button.secondary {{
            background: #6c757d;
        }}
        .controls button.secondary:hover {{
            background: #545b62;
        }}
        .controls button.danger {{
            background: #dc3545;
        }}
        .controls button.danger:hover {{
            background: #c82333;
        }}
        .controls button:disabled {{
            background: #ccc;
            cursor: not-allowed;
        }}
        .status {{
            font-size: 12px;
            color: #666;
            margin-top: 10px;
            padding: 5px;
            background: #f8f9fa;
            border-radius: 4px;
        }}
        .visualizing-mode #drawingControls {{
            display: none;
        }}
        .visualizing-mode #visualizeControls {{
            display: block;
        }}
        .drawing-mode #visualizeControls {{
            display: none;
        }}
        .drawing-mode #drawingControls {{
            display: block;
        }}
    </style>
</head>
<body class="drawing-mode">
    <div class="container">
        <canvas id="fourier_canvas" width="800" height="600"></canvas>
    </div>
    <div class="controls">
        <h3 id="modeTitle">Draw Mode</h3>
        <div id="drawingControls">
            <p style="font-size: 12px; color: #666;">Click and drag to draw a shape</p>
            <button id="visualizeBtn" disabled>Visualize</button>
            <button id="clearBtn" class="secondary">Clear</button>
        </div>
        <div id="visualizeControls">
            <button id="pauseBtn">Pause</button>
            <button id="resetBtn" class="secondary">Reset</button>
            <button id="speedUp">Speed +</button>
            <button id="speedDown">Speed -</button>
            <button id="newDrawBtn" class="danger">New Drawing</button>
        </div>
        <div class="status" id="status">Ready to draw</div>
    </div>
    <script>
        const canvas = document.getElementById('fourier_canvas');
        const context = canvas.getContext('2d');

        // State
        let isDrawing = false;
        let drawingPoints = [];
        let fourierData = null;
        let hasInitialData = false;

        // Animation state
        let time = 0;
        let animation_id = null;
        let is_paused = false;
        let speed_multiplier = 1.0;
        let circles = [];
        let wave = [];
        let center = {{ x: 400, y: 300 }};

        // Initialize with data if available
        const initialData = {};
        if (initialData && initialData.length > 0) {{
            hasInitialData = true;
            fourierData = initialData;
            initFourierVisualization();
        }}

        const Point = class {{
            constructor(x, y) {{
                this.x = x;
                this.y = y;
            }}
        }};

        const FourierCircle = class {{
            constructor(speed, radius, initial_angle) {{
                this.radius = radius / 2;
                this.speed = speed / 20;
                this.initial_angle = initial_angle;
            }}

            draw(ctx, at) {{
                ctx.beginPath();
                const x = at.x + this.radius * Math.cos(this.initial_angle + 2 * Math.PI * time * this.speed);
                const y = at.y + this.radius * Math.sin(this.initial_angle + 2 * Math.PI * time * this.speed);
                ctx.moveTo(at.x, at.y);
                ctx.lineTo(x, y);
                ctx.strokeStyle = 'rgba(202, 126, 86, 0.7)';
                ctx.lineWidth = 1;
                ctx.stroke();
            }}

            nextCenter(at) {{
                const x = at.x + this.radius * Math.cos(this.initial_angle + 2 * Math.PI * time * this.speed);
                const y = at.y + this.radius * Math.sin(this.initial_angle + 2 * Math.PI * time * this.speed);
                return new Point(x, y);
            }}
        }};

        // Drawing mode handlers
        canvas.addEventListener('mousedown', (e) => {{
            if (hasInitialData) return;
            isDrawing = true;
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            drawingPoints = [{{ x, y }}];
            updateStatus('Drawing...');
            redrawCanvas();
        }});

        canvas.addEventListener('mousemove', (e) => {{
            if (!isDrawing || hasInitialData) return;
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            drawingPoints.push({{ x, y }});
            redrawCanvas();
        }});

        canvas.addEventListener('mouseup', () => {{
            if (isDrawing) {{
                isDrawing = false;
                document.getElementById('visualizeBtn').disabled = drawingPoints.length < 3;
                updateStatus('Drawing complete. Click Visualize to see Fourier animation.');
            }}
        }});

        canvas.addEventListener('mouseleave', () => {{
            if (isDrawing) {{
                isDrawing = false;
                document.getElementById('visualizeBtn').disabled = drawingPoints.length < 3;
                updateStatus('Drawing complete. Click Visualize to see Fourier animation.');
            }}
        }});

        function redrawCanvas() {{
            context.clearRect(0, 0, canvas.width, canvas.height);
            if (drawingPoints.length > 1) {{
                context.beginPath();
                context.moveTo(drawingPoints[0].x, drawingPoints[0].y);
                for (let i = 1; i < drawingPoints.length; i++) {{
                    context.lineTo(drawingPoints[i].x, drawingPoints[i].y);
                }}
                context.strokeStyle = '#333';
                context.lineWidth = 2;
                context.stroke();
            }}
        }}

        function clearCanvas() {{
            context.clearRect(0, 0, canvas.width, canvas.height);
            drawingPoints = [];
            document.getElementById('visualizeBtn').disabled = true;
            updateStatus('Ready to draw');
        }}

        // Fourier visualization
        function initFourierVisualization() {{
            circles = [];
            wave = [];
            const n = fourierData.length;
            for (let i = 0; i < n; i++) {{
                circles[i] = new FourierCircle(fourierData[i].s, fourierData[i].r, fourierData[i].a);
            }}
            document.body.classList.remove('drawing-mode');
            document.body.classList.add('visualizing-mode');
            document.getElementById('modeTitle').textContent = 'Visualize Mode';
            animation_id = requestAnimationFrame(draw);
        }}

        function drawWave(ctx) {{
            for (let i = 1; i < wave.length; i++) {{
                ctx.beginPath();
                ctx.moveTo(wave[i - 1].x, wave[i - 1].y);
                ctx.lineTo(wave[i].x, wave[i].y);
                const alpha = 1 - i * 1.0 / wave.length;
                ctx.strokeStyle = 'rgba(0, 0, 0, ' + alpha + ')';
                ctx.lineWidth = 1;
                ctx.stroke();
            }}
        }}

        function draw() {{
            if (!is_paused) {{
                time += 0.04 * speed_multiplier;
            }}

            context.clearRect(0, 0, canvas.width, canvas.height);

            let new_center = circles[0].nextCenter(center);
            for (let i = 1; i < circles.length; i++) {{
                circles[i].draw(context, new_center);
                new_center = circles[i].nextCenter(new_center);
            }}

            if (!is_paused) {{
                wave.unshift(new_center);
                if (wave.length > 400) {{
                    wave.pop();
                }}
            }}

            drawWave(context);
            animation_id = requestAnimationFrame(draw);
        }}

        function updateStatus(message) {{
            document.getElementById('status').textContent = message;
        }}

        // Button handlers
        document.getElementById('clearBtn').addEventListener('click', clearCanvas);

        document.getElementById('visualizeBtn').addEventListener('click', () => {{
            // Convert drawing points to SVG path
            let svgPath = 'M ' + drawingPoints[0].x + ' ' + drawingPoints[0].y;
            for (let i = 1; i < drawingPoints.length; i++) {{
                svgPath += ' L ' + drawingPoints[i].x + ' ' + drawingPoints[i].y;
            }}
            svgPath += ' Z';

            updateStatus('Processing... This may take a moment.');

            // Send to Rust backend for FFT processing
            if (window.__TAURI__ && window.__TAURI__.core) {{
                window.__TAURI__.core.invoke('process_drawing', {{ path: svgPath }})
                    .then((data) => {{
                        fourierData = data;
                        initFourierVisualization();
                        updateStatus('Visualizing...');
                    }})
                    .catch((err) => {{
                        console.error('Error processing drawing:', err);
                        updateStatus('Error: ' + err);
                    }});
            }} else {{
                console.log('Drawing path:', svgPath);
                updateStatus('Tauri bridge not available. See console for path.');
            }}
        }});

        document.getElementById('pauseBtn').addEventListener('click', function() {{
            is_paused = !is_paused;
            this.textContent = is_paused ? 'Play' : 'Pause';
        }});

        document.getElementById('resetBtn').addEventListener('click', function() {{
            time = 0;
            wave = [];
        }});

        document.getElementById('speedUp').addEventListener('click', function() {{
            speed_multiplier = Math.min(speed_multiplier + 0.25, 3.0);
        }});

        document.getElementById('speedDown').addEventListener('click', function() {{
            speed_multiplier = Math.max(speed_multiplier - 0.25, 0.25);
        }});

        document.getElementById('newDrawBtn').addEventListener('click', function() {{
            if (animation_id) {{
                cancelAnimationFrame(animation_id);
                animation_id = null;
            }}
            hasInitialData = false;
            fourierData = null;
            time = 0;
            wave = [];
            circles = [];
            context.clearRect(0, 0, canvas.width, canvas.height);
            document.body.classList.remove('visualizing-mode');
            document.body.classList.add('drawing-mode');
            document.getElementById('modeTitle').textContent = 'Draw Mode';
            document.getElementById('visualizeBtn').disabled = true;
            updateStatus('Ready to draw');
        }});
    </script>
</body>
</html>
"#,
        initial_data.unwrap_or("null")
    )
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn process_drawing(path: String, state: tauri::State<AppState>) -> Vec<FourierData> {
    let svg_path = build_path_from_svg(&path);
    let fft_size = *state.num_sample.lock().unwrap();
    let num_wave = *state.num_wave.lock().unwrap();
    let fft_result = path_to_fft(svg_path, fft_size);

    // Build DrawData from FFT result
    let mut result = Vec::new();
    result.push(DrawData::new_from_complex(0 as f32, fft_result[0]));
    for i in 1..((num_wave + 1) / 2) {
        result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
        result.push(DrawData::new_from_complex((0 - i as i32) as f32, fft_result[fft_size - i]));
    }

    // Convert to FourierData for JSON serialization
    result.iter().map(|d| FourierData {
        s: d.frequency,
        r: d.radius,
        a: d.angle,
    }).collect()
}

#[cfg(feature = "tauri-app")]
struct AppState {
    num_sample: std::sync::Mutex<usize>,
    num_wave: std::sync::Mutex<usize>,
}

#[cfg(feature = "tauri-app")]
fn run_tauri_app(initial_data: Option<Vec<DrawData>>, num_sample: usize, num_wave: usize) {
    // Convert initial data if present
    let initial_json = initial_data.map(|data| {
        let json: String = data.iter()
            .map(|d| format!("{{\"s\": {}, \"r\": {}, \"a\": {}}},", d.frequency, d.radius, d.angle))
            .collect();
        if json.len() > 1 {
            format!("[{}]", &json[0..json.len()-1])
        } else {
            format!("[]")
        }
    });

    // Generate HTML content for the webview
    let html_content = generate_html(initial_json.as_deref());

    // Create temp directory for HTML
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("fourier_visualizer.html");

    if let Err(e) = std::fs::write(&html_path, html_content) {
        eprintln!("Failed to write HTML file: {}", e);
        return;
    }

    println!("HTML written to: {:?}", html_path);

    // Create app state
    let app_state = AppState {
        num_sample: std::sync::Mutex::new(num_sample),
        num_wave: std::sync::Mutex::new(num_wave),
    };

    // Start Tauri app
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![process_drawing])
        .setup(move |app| {
            let window = WebviewWindowBuilder::new(
                app,
                "fourier",
                WebviewUrl::from(html_path.clone())
            )
            .title("Fourier SVG Visualizer - Draw or Load SVG")
            .inner_size(850.0, 650.0)
            .resizable(true)
            .build()?;

            window.eval("console.log('Fourier Visualizer loaded')")?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(feature = "tauri-app"))]
fn run_tauri_app(_initial_data: Option<Vec<DrawData>>, _num_sample: usize, _num_wave: usize) {
    eprintln!("Tauri visualizer requires the 'tauri-app' feature to be enabled.");
    eprintln!("Run with: cargo run --bin tauri-app --features tauri-app");
    eprintln!("");
    eprintln!("Note: On Linux, you need to install GTK development libraries:");
    eprintln!("  Ubuntu/Debian: sudo apt-get install libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev");
    eprintln!("  Fedora: sudo dnf install webkit2gtk3-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel");
    eprintln!("  Arch: sudo pacman -S webkit2gtk gtk3 libappindicator-gtk3 librsvg");
}

fn main() {
    let args = Args::parse();

    // SVG source args
    let arg_path = args.svg_path.as_deref().unwrap_or_default();
    let arg_svg_file = args.svg_file.as_deref().unwrap_or_default();
    let input_file = args.input_file.clone();

    let num_sample = args.num_sample;
    let mut num_wave = args.num_wave;

    // Make sure num_sample >= num_wave
    if num_sample < num_wave {
        num_wave = num_sample;
    }

    // Get Fourier data - either from exported file or compute from SVG
    let initial_data: Option<Vec<DrawData>> = if let Some(input_path) = input_file {
        // Load from exported Fourier data
        match load_fourier_export(&input_path) {
            Ok(export) => {
                println!("Loaded Fourier data from {} ({} coefficients, {} samples)",
                    input_path, export.metadata.wave_count, export.metadata.sample_count);
                Some(export_to_draw_data(&export))
            }
            Err(e) => {
                eprintln!("Failed to load Fourier data: {}", e);
                None
            }
        }
    } else if !arg_svg_file.is_empty() || !arg_path.is_empty() {
        // Compute from SVG
        let mut svg_string: String = String::new();
        if !arg_svg_file.is_empty() {
            // Read path from svg file
            let mut content = String::new();
            for event in svg::open(arg_svg_file, &mut content).unwrap() {
                match event {
                    svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) => {
                        svg_string = attributes.get("d").unwrap().to_string();
                        break;
                    }
                    _ => {}
                }
            }
        } else if !arg_path.is_empty() {
            // Read path from svg path string
            svg_string = arg_path.to_string();
        }

        let path = build_path_from_svg(&svg_string);
        let fft_size = num_sample;
        let fft_result = path_to_fft(path, fft_size);

        // Build DrawData from FFT result
        let mut result = Vec::new();
        result.push(DrawData::new_from_complex(0 as f32, fft_result[0]));
        for i in 1..((num_wave + 1) / 2) {
            result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
            result.push(DrawData::new_from_complex((0 - i as i32) as f32, fft_result[fft_size - i]));
        }

        Some(result)
    } else {
        // No SVG provided - user will draw in the app
        println!("No SVG path provided. You can draw one in the application.");
        None
    };

    run_tauri_app(initial_data, num_sample, num_wave);
}
