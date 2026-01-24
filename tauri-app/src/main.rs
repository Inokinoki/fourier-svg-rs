//! Tauri Fourier Visualizer Application
//!
//! This application provides an interactive interface for drawing SVG paths
//! and visualizing them using Fourier epicycles.
//!
//! Features:
//! - Interactive drawing on canvas
//! - Adjustable sampling rate
//! - Display coefficient information for each component
//! - Dynamic component adjustment during preview
//! - Save/load time and path information
//!
//! Build requirements on Linux:
//!   sudo apt-get install libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

use clap::Parser;
use fourier_svg::{
    build_path_from_svg, export_to_draw_data, load_fourier_export, path_to_fft, DrawData,
};

#[cfg(feature = "tauri")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "tauri")]
use tauri::{WebviewUrl, WebviewWindowBuilder};

#[cfg(feature = "tauri")]
use tauri_plugin_fs::FsExt;

#[cfg(feature = "tauri")]
use tauri_plugin_dialog::DialogExt;

#[cfg(feature = "tauri")]
#[derive(Clone, Serialize, Deserialize)]
struct FourierData {
    s: f32,
    r: f32,
    a: f32,
    idx: usize,
}

#[cfg(feature = "tauri")]
#[derive(Clone, Serialize, Deserialize)]
struct SvgPathInfo {
    id: String,
    d: String,
}

#[cfg(feature = "tauri")]
#[derive(Clone, Serialize, Deserialize)]
struct SvgPathsResponse {
    paths: Vec<SvgPathInfo>,
    width: Option<f32>,
    height: Option<f32>,
}

#[cfg(feature = "tauri")]
#[derive(Clone, Serialize)]
struct AppState {
    num_sample: usize,
    num_wave: usize,
    max_wave: usize,
}

/// Tauri Fourier Visualizer
#[derive(Parser, Debug)]
#[command(author = "Inoki <veyx.shaw@gmail.com>", version = "1.0.0", about)]
#[command(propagate_version = true)]
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

    /// Use how many waves (components) to use
    #[arg(short = 'w', long = "wave", default_value = "201")]
    num_wave: usize,
}

#[cfg(feature = "tauri")]
fn generate_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Fourier Visualizer - Interactive</title>
    <style>
        * { box-sizing: border-box; }
        body {
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            overflow: hidden;
        }
        .sidebar {
            width: 320px;
            background: rgba(255, 255, 255, 0.95);
            padding: 20px;
            overflow-y: auto;
            box-shadow: 2px 0 10px rgba(0,0,0,0.1);
        }
        .main-content {
            flex: 1;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .canvas-container {
            background: white;
            border-radius: 12px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.1);
            padding: 10px;
        }
        canvas {
            display: block;
            border-radius: 8px;
            cursor: crosshair;
        }
        h1 {
            font-size: 20px;
            margin: 0 0 20px 0;
            color: #333;
        }
        h2 {
            font-size: 16px;
            margin: 20px 0 10px 0;
            color: #555;
        }
        .control-group {
            margin-bottom: 15px;
            padding: 10px;
            background: #f8f9fa;
            border-radius: 8px;
        }
        label {
            display: block;
            font-size: 12px;
            color: #666;
            margin-bottom: 5px;
        }
        input[type="range"] {
            width: 100%;
            margin: 5px 0;
        }
        .value-display {
            font-size: 14px;
            font-weight: bold;
            color: #667eea;
        }
        button {
            width: 100%;
            padding: 10px;
            margin: 5px 0;
            border: none;
            border-radius: 6px;
            background: #667eea;
            color: white;
            font-size: 14px;
            cursor: pointer;
            transition: background 0.2s;
        }
        button:hover {
            background: #5568d3;
        }
        button:disabled {
            background: #ccc;
            cursor: not-allowed;
        }
        button.secondary {
            background: #6c757d;
        }
        button.secondary:hover {
            background: #5a6268;
        }
        button.danger {
            background: #dc3545;
        }
        button.danger:hover {
            background: #c82333;
        }
        .coefficients {
            max-height: 300px;
            overflow-y: auto;
            font-size: 11px;
            background: white;
            border-radius: 6px;
            padding: 10px;
        }
        .coef-item {
            display: flex;
            justify-content: space-between;
            padding: 4px 0;
            border-bottom: 1px solid #eee;
        }
        .coef-item:last-child {
            border-bottom: none;
        }
        .coef-index { color: #667eea; font-weight: bold; width: 40px; }
        .coef-freq { width: 60px; }
        .coef-radius { width: 80px; }
        .coef-angle { width: 80px; }
        .status {
            font-size: 12px;
            color: #666;
            padding: 10px;
            background: #fff3cd;
            border-radius: 6px;
            margin-top: 10px;
        }
        .hidden { display: none !important; }
    </style>
</head>
<body>
    <div class="sidebar">
        <h1>Fourier Visualizer</h1>

        <div class="control-group">
            <label>Input Mode:</label>
            <div style="display: flex; gap: 10px; margin-top: 5px;">
                <button id="modeFileBtn" class="secondary">Load SVG</button>
                <button id="modeDrawBtn" class="secondary">Draw</button>
            </div>
        </div>

        <div id="svgControls" class="hidden">
            <h2>SVG File Mode</h2>
            <p style="font-size: 12px; color: #666;">Select an SVG file to load paths</p>

            <div class="control-group">
                <button id="loadSvgBtn">Load SVG File</button>
            </div>

            <div id="pathSelectionGroup" class="control-group hidden">
                <label>Select Path:</label>
                <select id="pathSelect" style="width: 100%; padding: 8px; border-radius: 6px; border: 1px solid #ddd;">
                    <option value="">-- Select a path --</option>
                </select>
            </div>

            <div class="control-group">
                <label>Sample Rate: <span id="sampleValueSvg" class="value-display">10240</span></label>
                <input type="range" id="sampleRateSvg" min="1000" max="20000" value="10240" step="500">
            </div>

            <div class="control-group">
                <label>Duration (seconds): <span id="durationValueSvg" class="value-display">10.0</span></label>
                <input type="range" id="durationSvg" min="1" max="60" value="10" step="0.5">
            </div>

            <button id="visualizeSvgBtn" disabled>Visualize SVG Path</button>
        </div>

        <div id="drawingControls">
            <h2>Drawing Mode</h2>
            <p style="font-size: 12px; color: #666;">Click and drag on the canvas to draw a shape</p>

            <div class="control-group">
                <label>Sample Rate: <span id="sampleValue" class="value-display">10240</span></label>
                <input type="range" id="sampleRate" min="1000" max="20000" value="10240" step="500">
            </div>

            <div class="control-group">
                <label>Duration (seconds): <span id="durationValue" class="value-display">10.0</span></label>
                <input type="range" id="durationDraw" min="1" max="60" value="10" step="0.5">
            </div>

            <button id="visualizeBtn" disabled>Visualize</button>
            <button id="clearBtn" class="secondary">Clear Canvas</button>
        </div>

        <div id="visualizeControls" class="hidden">
            <h2>Visualization Mode</h2>

            <div class="control-group">
                <label>Components: <span id="waveValue" class="value-display">201</span></label>
                <input type="range" id="waveCount" min="1" max="500" value="201" step="1">
            </div>

            <div class="control-group">
                <label>Speed: <span id="speedValue" class="value-display">1.0x</span></label>
                <input type="range" id="speedControl" min="0.1" max="3.0" value="1.0" step="0.1">
            </div>

            <button id="pauseBtn">Pause</button>
            <button id="resetBtn" class="secondary">Reset Animation</button>
            <button id="newDrawBtn" class="danger">New Drawing</button>
        </div>

        <div id="coefficientsPanel" class="hidden">
            <h2>Coefficients</h2>
            <div class="coefficients" id="coefficientsList"></div>
        </div>

        <div class="status" id="status">Ready to draw</div>
    </div>

    <div class="main-content">
        <div class="canvas-container">
            <canvas id="fourier_canvas" width="700" height="600"></canvas>
        </div>
    </div>

    <script>
        const canvas = document.getElementById('fourier_canvas');
        const context = canvas.getContext('2d');

        // State
        let isDrawing = false;
        let drawingPoints = [];
        let drawingPointsWithTime = []; // Store points with timestamp
        let fourierData = null;
        let fullFourierData = null;
        let currentMode = 'draw'; // 'draw' or 'svg'
        let svgPaths = [];
        let selectedPathData = null;
        let drawingStartTime = 0;

        // Animation state
        let time = 0;
        let animation_id = null;
        let is_paused = false;
        let speed_multiplier = 1.0;
        let current_wave_count = 201;
        let circles = [];
        let wave = [];
        let center = { x: 350, y: 300 };

        // Default parameters
        let defaultSampleRate = 10240;
        let defaultDuration = 10.0;

        const Point = class {
            constructor(x, y) {
                this.x = x;
                this.y = y;
            }
        };

        const FourierCircle = class {
            constructor(idx, speed, radius, initial_angle) {
                this.idx = idx;
                this.radius = radius / 2;
                this.speed = speed / 20;
                this.initial_angle = initial_angle;
            }

            draw(ctx, at) {
                ctx.beginPath();
                const x = at.x + this.radius * Math.cos(this.initial_angle + 2 * Math.PI * time * this.speed);
                const y = at.y + this.radius * Math.sin(this.initial_angle + 2 * Math.PI * time * this.speed);
                ctx.moveTo(at.x, at.y);
                ctx.lineTo(x, y);
                ctx.strokeStyle = `hsl(${(this.idx * 5) % 360}, 70%, 60%)`;
                ctx.lineWidth = Math.max(0.5, this.radius / 50);
                ctx.stroke();
            }

            nextCenter(at) {
                const x = at.x + this.radius * Math.cos(this.initial_angle + 2 * Math.PI * time * this.speed);
                const y = at.y + this.radius * Math.sin(this.initial_angle + 2 * Math.PI * time * this.speed);
                return new Point(x, y);
            }
        };

        // Drawing handlers
        canvas.addEventListener('mousedown', (e) => {
            if (fourierData || currentMode === 'svg') return;
            isDrawing = true;
            drawingStartTime = Date.now();
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            drawingPoints = [{ x, y }];
            drawingPointsWithTime = [{ x, y, time: 0 }];
            updateStatus('Drawing...');
            redrawCanvas();
        });

        canvas.addEventListener('mousemove', (e) => {
            if (!isDrawing || fourierData || currentMode === 'svg') return;
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            const elapsed = (Date.now() - drawingStartTime) / 1000; // seconds
            drawingPoints.push({ x, y });
            drawingPointsWithTime.push({ x, y, time: elapsed });
            redrawCanvas();
        });

        canvas.addEventListener('mouseup', () => finishDrawing());
        canvas.addEventListener('mouseleave', () => finishDrawing());

        function finishDrawing() {
            if (isDrawing) {
                isDrawing = false;
                document.getElementById('visualizeBtn').disabled = drawingPoints.length < 3;
                updateStatus(`Drawing complete (${drawingPoints.length} points). Click Visualize.`);
            }
        }

        function redrawCanvas() {
            context.clearRect(0, 0, canvas.width, canvas.height);
            if (drawingPoints.length > 1) {
                context.beginPath();
                context.moveTo(drawingPoints[0].x, drawingPoints[0].y);
                for (let i = 1; i < drawingPoints.length; i++) {
                    context.lineTo(drawingPoints[i].x, drawingPoints[i].y);
                }
                context.strokeStyle = '#333';
                context.lineWidth = 2;
                context.stroke();
            }
        }

        function clearCanvas() {
            context.clearRect(0, 0, canvas.width, canvas.height);
            drawingPoints = [];
            document.getElementById('visualizeBtn').disabled = true;
            updateStatus('Ready to draw');
        }

        function initFourierVisualization() {
            circles = [];
            wave = [];
            current_wave_count = Math.min(current_wave_count, fullFourierData.length);
            fourierData = fullFourierData.slice(0, current_wave_count);

            for (let i = 0; i < fourierData.length; i++) {
                circles[i] = new FourierCircle(
                    fourierData[i].idx,
                    fourierData[i].s,
                    fourierData[i].r,
                    fourierData[i].a
                );
            }

            document.getElementById('drawingControls').classList.add('hidden');
            document.getElementById('visualizeControls').classList.remove('hidden');
            document.getElementById('coefficientsPanel').classList.remove('hidden');
            updateCoefficientsList();

            animation_id = requestAnimationFrame(draw);
        }

        function updateCoefficientsList() {
            const list = document.getElementById('coefficientsList');
            list.innerHTML = '<div class="coef-item"><span class="coef-index">#</span><span class="coef-freq">Freq</span><span class="coef-radius">Radius</span><span class="coef-angle">Angle</span></div>';

            for (let i = 0; i < Math.min(fourierData.length, 50); i++) {
                const d = fourierData[i];
                const div = document.createElement('div');
                div.className = 'coef-item';
                div.innerHTML = `
                    <span class="coef-index">${d.idx}</span>
                    <span class="coef-freq">${d.s.toFixed(1)}</span>
                    <span class="coef-radius">${d.r.toFixed(2)}</span>
                    <span class="coef-angle">${d.a.toFixed(2)}</span>
                `;
                list.appendChild(div);
            }
        }

        function updateWaveCount(newCount) {
            current_wave_count = newCount;
            fourierData = fullFourierData.slice(0, current_wave_count);

            circles = [];
            wave = [];

            for (let i = 0; i < fourierData.length; i++) {
                circles[i] = new FourierCircle(
                    fourierData[i].idx,
                    fourierData[i].s,
                    fourierData[i].r,
                    fourierData[i].a
                );
            }

            updateCoefficientsList();
        }

        function drawWave(ctx) {
            for (let i = 1; i < wave.length; i++) {
                ctx.beginPath();
                ctx.moveTo(wave[i - 1].x, wave[i - 1].y);
                ctx.lineTo(wave[i].x, wave[i].y);
                const alpha = 1 - i * 1.0 / wave.length;
                ctx.strokeStyle = `rgba(0, 0, 0, ${alpha})`;
                ctx.lineWidth = 1;
                ctx.stroke();
            }
        }

        function draw() {
            if (!is_paused) {
                time += 0.04 * speed_multiplier;
            }

            context.clearRect(0, 0, canvas.width, canvas.height);

            if (circles.length > 0) {
                let new_center = circles[0].nextCenter(center);
                for (let i = 1; i < circles.length; i++) {
                    circles[i].draw(context, new_center);
                    new_center = circles[i].nextCenter(new_center);
                }

                if (!is_paused) {
                    wave.unshift(new_center);
                    if (wave.length > 500) wave.pop();
                }

                drawWave(context);
            }

            animation_id = requestAnimationFrame(draw);
        }

        function updateStatus(message) {
            document.getElementById('status').textContent = message;
        }

        // Controls
        document.getElementById('sampleRate').addEventListener('input', (e) => {
            document.getElementById('sampleValue').textContent = e.target.value;
        });

        document.getElementById('clearBtn').addEventListener('click', clearCanvas);

        document.getElementById('visualizeBtn').addEventListener('click', () => {
            let svgPath = 'M ' + drawingPoints[0].x + ' ' + drawingPoints[0].y;
            for (let i = 1; i < drawingPoints.length; i++) {
                svgPath += ' L ' + drawingPoints[i].x + ' ' + drawingPoints[i].y;
            }
            svgPath += ' Z';

            const sampleRate = parseInt(document.getElementById('sampleRate').value);
            updateStatus('Processing... This may take a moment.');

            if (window.__TAURI__ && window.__TAURI__.core) {
                window.__TAURI__.core.invoke('process_drawing', {
                    path: svgPath,
                    numSample: sampleRate
                })
                .then((data) => {
                    fullFourierData = data;
                    document.getElementById('waveCount').max = data.length;
                    initFourierVisualization();
                    updateStatus('Visualizing with ' + data.length + ' components');
                })
                .catch((err) => {
                    console.error('Error:', err);
                    updateStatus('Error: ' + err);
                });
            } else {
                console.log('Drawing path:', svgPath);
                updateStatus('Tauri bridge not available');
            }
        });

        document.getElementById('waveCount').addEventListener('input', (e) => {
            const count = parseInt(e.target.value);
            document.getElementById('waveValue').textContent = count;
            updateWaveCount(count);
            wave = [];
            updateStatus(`Using ${count} components`);
        });

        document.getElementById('speedControl').addEventListener('input', (e) => {
            speed_multiplier = parseFloat(e.target.value);
            document.getElementById('speedValue').textContent = speed_multiplier.toFixed(1) + 'x';
        });

        document.getElementById('pauseBtn').addEventListener('click', function() {
            is_paused = !is_paused;
            this.textContent = is_paused ? 'Play' : 'Pause';
        });

        document.getElementById('resetBtn').addEventListener('click', () => {
            time = 0;
            wave = [];
            updateStatus('Animation reset');
        });

        document.getElementById('newDrawBtn').addEventListener('click', () => {
            if (animation_id) cancelAnimationFrame(animation_id);
            fourierData = null;
            fullFourierData = null;
            time = 0;
            wave = [];
            circles = [];
            context.clearRect(0, 0, canvas.width, canvas.height);
            document.getElementById('drawingControls').classList.remove('hidden');
            document.getElementById('svgControls').classList.add('hidden');
            document.getElementById('visualizeControls').classList.add('hidden');
            document.getElementById('coefficientsPanel').classList.add('hidden');
            updateStatus('Ready to draw');
        });

        // Mode switching
        document.getElementById('modeFileBtn').addEventListener('click', () => {
            currentMode = 'svg';
            document.getElementById('svgControls').classList.remove('hidden');
            document.getElementById('drawingControls').classList.add('hidden');
            document.getElementById('visualizeSvgBtn').disabled = true;
            context.clearRect(0, 0, canvas.width, canvas.height);
            updateStatus('SVG File mode: Load an SVG file to begin');
        });

        document.getElementById('modeDrawBtn').addEventListener('click', () => {
            currentMode = 'draw';
            document.getElementById('svgControls').classList.add('hidden');
            document.getElementById('drawingControls').classList.remove('hidden');
            document.getElementById('visualizeBtn').disabled = drawingPoints.length < 3;
            context.clearRect(0, 0, canvas.width, canvas.height);
            redrawCanvas();
            updateStatus('Drawing mode: Draw on the canvas');
        });

        // SVG file loading
        document.getElementById('loadSvgBtn').addEventListener('click', async () => {
            if (window.__TAURI__ && window.__TAURI__.dialog) {
                try {
                    const selected = await window.__TAURI__.dialog.open({
                        multiple: false,
                        filters: [{
                            name: 'SVG',
                            extensions: ['svg']
                        }]
                    });

                    if (selected) {
                        updateStatus('Parsing SVG file...');
                        const result = await window.__TAURI__.core.invoke('parse_svg_file', {
                            filePath: selected
                        });

                        svgPaths = result.paths;
                        const pathSelect = document.getElementById('pathSelect');
                        pathSelect.innerHTML = '<option value="">-- Select a path --</option>';

                        for (const path of svgPaths) {
                            const option = document.createElement('option');
                            option.value = path.d;
                            option.textContent = path.id || 'Unnamed path';
                            pathSelect.appendChild(option);
                        }

                        document.getElementById('pathSelectionGroup').classList.remove('hidden');
                        updateStatus(`Loaded ${svgPaths.length} paths from SVG. Select a path to visualize.`);
                    }
                } catch (err) {
                    console.error('Error loading SVG:', err);
                    updateStatus('Error loading SVG: ' + err);
                }
            } else {
                updateStatus('File dialog not available');
            }
        });

        // Path selection
        document.getElementById('pathSelect').addEventListener('change', (e) => {
            selectedPathData = e.target.value;
            document.getElementById('visualizeSvgBtn').disabled = !selectedPathData;
            if (selectedPathData) {
                updateStatus('Path selected. Click Visualize SVG Path.');
            }
        });

        // SVG sample rate control
        document.getElementById('sampleRateSvg').addEventListener('input', (e) => {
            document.getElementById('sampleValueSvg').textContent = e.target.value;
        });

        // Drawing sample rate control
        document.getElementById('sampleRate').addEventListener('input', (e) => {
            document.getElementById('sampleValue').textContent = e.target.value;
        });

        // Duration controls
        document.getElementById('durationSvg').addEventListener('input', (e) => {
            document.getElementById('durationValueSvg').textContent = parseFloat(e.target.value).toFixed(1);
        });

        document.getElementById('durationDraw').addEventListener('input', (e) => {
            document.getElementById('durationValue').textContent = parseFloat(e.target.value).toFixed(1);
        });

        // Visualize SVG path
        document.getElementById('visualizeSvgBtn').addEventListener('click', () => {
            if (!selectedPathData) return;

            const sampleRate = parseInt(document.getElementById('sampleRateSvg').value);
            const duration = parseFloat(document.getElementById('durationSvg').value);
            updateStatus('Processing SVG path... This may take a moment.');

            if (window.__TAURI__ && window.__TAURI__.core) {
                window.__TAURI__.core.invoke('process_svg_path', {
                    pathData: selectedPathData,
                    numSample: sampleRate
                })
                .then((data) => {
                    fullFourierData = data;
                    document.getElementById('waveCount').max = data.length;
                    initFourierVisualization();
                    updateStatus(`Visualizing SVG path with ${data.length} components (Duration: ${duration}s)`);
                })
                .catch((err) => {
                    console.error('Error:', err);
                    updateStatus('Error: ' + err);
                });
            } else {
                updateStatus('Tauri bridge not available');
            }
        });
    </script>
</body>
</html>"#
    .to_string()
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn process_drawing(path: String, num_sample: usize) -> Vec<FourierData> {
    let svg_path = build_path_from_svg(&path);
    let fft_result = path_to_fft(svg_path, num_sample);

    // Build DrawData from FFT result
    let mut result = Vec::new();
    result.push(DrawData::new_from_complex(0 as f32, fft_result[0]));
    for i in 1..fft_result.len() / 2 {
        result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
        result.push(DrawData::new_from_complex(
            (0 - i as i32) as f32,
            fft_result[fft_result.len() - i],
        ));
    }

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

/// Command to parse an SVG file and extract all paths
#[cfg(feature = "tauri")]
#[tauri::command]
async fn parse_svg_file(file_path: String) -> Result<SvgPathsResponse, String> {
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
                                    id.clone()
                                } else {
                                    format!("path_{}", path_index)
                                },
                                d: d.clone(),
                            });
                            path_index += 1;
                        }
                    }
                    svg::parser::Event::Tag(svg::node::element::tag::Svg, _, attributes) => {
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

            Ok(SvgPathsResponse { paths, width, height })
        }
        Err(e) => Err(format!("Failed to parse SVG: {}", e)),
    }
}

/// Parse SVG dimension attribute to float
#[cfg(feature = "tauri")]
fn parse_svg_dimension(value: &str) -> Option<f32> {
    let value = value.trim().trim_end_matches("px").trim_end_matches("pt").trim_end_matches("%");
    value.parse::<f32>().ok()
}

/// Command to process a selected SVG path
#[cfg(feature = "tauri")]
#[tauri::command]
fn process_svg_path(path_data: String, num_sample: usize) -> Vec<FourierData> {
    let svg_path = build_path_from_svg(&path_data);
    let fft_result = path_to_fft(svg_path, num_sample);

    // Build DrawData from FFT result
    let mut result = Vec::new();
    result.push(DrawData::new_from_complex(0 as f32, fft_result[0]));
    for i in 1..fft_result.len() / 2 {
        result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
        result.push(DrawData::new_from_complex(
            (0 - i as i32) as f32,
            fft_result[fft_result.len() - i],
        ));
    }

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

#[cfg(feature = "tauri")]
fn run_tauri_app(initial_data: Option<Vec<DrawData>>, _num_sample: usize, _num_wave: usize) {
    let html_content = generate_html();

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("fourier_visualizer_interactive.html");

    if let Err(e) = std::fs::write(&html_path, html_content) {
        eprintln!("Failed to write HTML file: {}", e);
        return;
    }

    println!("HTML written to: {:?}", html_path);

    // Build and run Tauri app programmatically
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            process_drawing,
            parse_svg_file,
            process_svg_path
        ])
        .setup(move |app| {
            let window =
                WebviewWindowBuilder::new(app, "fourier", WebviewUrl::App(html_path.clone()))
                    .title("Fourier SVG Visualizer - Interactive")
                    .inner_size(1050.0, 650.0)
                    .resizable(true)
                    .build()?;

            window.eval("console.log('Fourier Visualizer loaded')")?;
            Ok(())
        })
        .run(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");
}

#[cfg(not(feature = "tauri"))]
fn run_tauri_app(_initial_data: Option<Vec<DrawData>>, _num_sample: usize, _num_wave: usize) {
    eprintln!("Tauri visualizer requires the 'tauri' feature to be enabled.");
    eprintln!("Run with: cargo run --features tauri");
    eprintln!();
    eprintln!("Build requirements on Linux:");
    eprintln!("  sudo apt-get install libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev");
}

fn main() {
    let args = Args::parse();

    let arg_path = args.svg_path.as_deref().unwrap_or_default();
    let arg_svg_file = args.svg_file.as_deref().unwrap_or_default();
    let input_file = args.input_file.clone();

    let num_sample = args.num_sample;
    let num_wave = args.num_wave;

    let initial_data: Option<Vec<DrawData>> = if let Some(input_path) = input_file {
        match load_fourier_export(&input_path) {
            Ok(export) => {
                println!(
                    "Loaded Fourier data from {} ({} coefficients, {} samples)",
                    input_path, export.metadata.wave_count, export.metadata.sample_count
                );
                Some(export_to_draw_data(&export))
            }
            Err(e) => {
                eprintln!("Failed to load Fourier data: {}", e);
                None
            }
        }
    } else if !arg_svg_file.is_empty() || !arg_path.is_empty() {
        let mut svg_string: String = String::new();
        if !arg_svg_file.is_empty() {
            let mut content = String::new();
            for event in svg::open(arg_svg_file, &mut content).unwrap() {
                if let svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) = event
                {
                    svg_string = attributes.get("d").unwrap().to_string();
                    break;
                }
            }
        } else if !arg_path.is_empty() {
            svg_string = arg_path.to_string();
        }

        let path = build_path_from_svg(&svg_string);
        let fft_result = path_to_fft(path, num_sample);

        let mut result = Vec::new();
        result.push(DrawData::new_from_complex(0 as f32, fft_result[0]));
        for i in 1..num_wave.div_ceil(2) {
            result.push(DrawData::new_from_complex(i as f32, fft_result[i]));
            result.push(DrawData::new_from_complex(
                (0 - i as i32) as f32,
                fft_result[fft_result.len() - i],
            ));
        }

        Some(result)
    } else {
        println!("No SVG path provided - launching in interactive drawing mode");
        None
    };

    run_tauri_app(initial_data, num_sample, num_wave);
}
