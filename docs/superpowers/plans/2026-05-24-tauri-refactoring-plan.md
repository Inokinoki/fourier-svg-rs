# Tauri App Architecture Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the 54,000-line monolithic `tauri-app/src/main.rs` into modular architecture with separate frontend files and organized Rust backend, removing non-functional fantasy features.

**Architecture:** Extract embedded HTML/CSS/JS into `ui/` directory served by Tauri's asset protocol. Organize Rust commands into `src/commands/` module. Target: ~6,000 lines total.

**Tech Stack:** Rust, Tauri v2, HTML/CSS/JavaScript (vanilla)

---

## File Structure

```
tauri-app/
├── src/
│   ├── main.rs              # Tauri app setup + command registration (~50 lines)
│   ├── commands/
│   │   ├── mod.rs           # Module exports
│   │   ├── drawing.rs       # process_drawing command
│   │   ├── svg.rs           # parse_svg_file, process_svg_path, get_svg_paths
│   │   ├── export.rs        # export_fourier_data, export_as_gif, export_as_html
│   │   └── files.rs         # save_canvas_as_png, recent_files
│   └── lib.rs               # Public API exports
├── ui/                      # Frontend static files
│   ├── index.html           # Main HTML structure
│   ├── style.css            # All CSS styles
│   ├── app.js               # App initialization, state, Tauri bridge
│   ├── drawing.js           # Canvas drawing, input handling
│   ├── visualization.js     # Fourier animation, epicycle rendering
│   ├── controls.js          # UI controls, sliders, presets
│   └── export.js            # Export functions
├── Cargo.toml
└── tauri.conf.json          # build.frontendDist: "../ui"
```

---

### Task 1: Create ui/ directory structure

**Files:**
- Create: `tauri-app/ui/` directory

- [ ] **Step 1: Create ui directory**

Run: `mkdir -p tauri-app/ui`

- [ ] **Step 2: Verify directory exists**

Run: `ls -la tauri-app/ui/`
Expected: Directory exists (empty)

---

### Task 2: Extract CSS to ui/style.css

**Files:**
- Read: `tauri-app/src/main.rs:133-928` (CSS styles)
- Create: `tauri-app/ui/style.css`

- [ ] **Step 1: Read the CSS from main.rs**

Read lines 133-928 from main.rs (the `<style>` block content).

- [ ] **Step 2: Create style.css with extracted CSS**

Write the CSS content (lines 133-928) to `tauri-app/ui/style.css`. Remove the `<style>` and `</style>` tags.

- [ ] **Step 3: Verify CSS file**

Run: `wc -l tauri-app/ui/style.css`
Expected: ~796 lines

---

### Task 3: Extract HTML to ui/index.html

**Files:**
- Read: `tauri-app/src/main.rs:127-53551` (HTML content)
- Create: `tauri-app/ui/index.html`

- [ ] **Step 1: Read the HTML structure**

Read the HTML content from main.rs. The HTML starts at line 127 (`<html lang="en">`) and ends at line 53551 (`</html>`).

- [ ] **Step 2: Create index.html**

Write a clean `index.html` with:
- DOCTYPE and head (link to style.css)
- Body with sidebar, canvas container, modals
- Script imports for app.js, drawing.js, visualization.js, controls.js, export.js
- Remove all `<style>` blocks (now in style.css)
- Remove all `<script>` blocks (now in .js files)
- Keep only the HTML structure

- [ ] **Step 3: Verify HTML file**

Run: `wc -l tauri-app/ui/index.html`
Expected: ~2,000 lines

---

### Task 4: Extract JavaScript to ui/app.js

**Files:**
- Read: `tauri-app/src/main.rs:17185-17353` (state variables)
- Read: `tauri-app/src/main.rs:46233-46330` (keyboard shortcuts)
- Create: `tauri-app/ui/app.js`

- [ ] **Step 1: Create app.js with state initialization**

Extract from main.rs:
- Lines 17185-17353: State variables (canvas, context, drawing state, animation state, etc.)
- Lines 46233-46330: Keyboard shortcuts handler

Write to `tauri-app/ui/app.js`:
```javascript
// State variables
let canvas = null;
let context = null;
let isDrawing = false;
let drawingPoints = [];
let fourierData = null;
let fullFourierData = null;
let currentMode = 'draw';
// ... (all state from lines 17185-17353)

// Tauri invoke wrapper
async function tauriInvoke(command, args) {
    if (window.__TAURI__ && window.__TAURI__.core) {
        return await window.__TAURI__.core.invoke(command, args);
    }
    throw new Error('Tauri bridge not available');
}

// Status update
function updateStatus(message) {
    const statusEl = document.getElementById('status');
    if (statusEl) statusEl.textContent = message;
}

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    // ... shortcuts from lines 46233-46330
});

// Initialize on load
window.addEventListener('DOMContentLoaded', () => {
    canvas = document.getElementById('canvas');
    context = canvas.getContext('2d');
    // ... initialization
});
```

- [ ] **Step 2: Verify app.js**

Run: `wc -l tauri-app/ui/app.js`
Expected: ~300 lines

---

### Task 5: Extract drawing.js

**Files:**
- Read: `tauri-app/src/main.rs:18313-18600` (canvas events)
- Read: `tauri-app/src/main.rs:18208-18221` (preset shapes)
- Create: `tauri-app/ui/drawing.js`

- [ ] **Step 1: Create drawing.js**

Extract from main.rs:
- Lines 18313-18600: Canvas mouse event handlers (mousedown, mousemove, mouseup)
- Lines 18208-18221: Preset shapes library
- Lines 18271-18312: Undo/redo system

Write to `tauri-app/ui/drawing.js`:
```javascript
// Drawing tools
let drawingTool = 'freehand';
let currentStroke = [];

// Canvas event handlers
function initDrawing(canvas) {
    canvas.addEventListener('mousedown', handleMouseDown);
    canvas.addEventListener('mousemove', handleMouseMove);
    canvas.addEventListener('mouseup', handleMouseUp);
}

function handleMouseDown(e) {
    // ... from lines 18313-18380
}

function handleMouseMove(e) {
    // ... from lines 18382-18520
}

function handleMouseUp(e) {
    // ... from lines 18520-18600
}

// Preset shapes
const presetShapes = {
    circle: (cx, cy, r) => { /* ... */ },
    square: (cx, cy, s) => { /* ... */ },
    triangle: (cx, cy, s) => { /* ... */ },
    star: (cx, cy, r) => { /* ... */ },
    heart: (cx, cy, s) => { /* ... */ },
    spiral: (cx, cy, r) => { /* ... */ }
};

// Undo/redo
let undoStack = [];
let redoStack = [];
function undo() { /* ... */ }
function redo() { /* ... */ }
```

- [ ] **Step 2: Verify drawing.js**

Run: `wc -l tauri-app/ui/drawing.js`
Expected: ~400 lines

---

### Task 6: Extract visualization.js

**Files:**
- Read: `tauri-app/src/main.rs:21305-21641` (animation loop)
- Create: `tauri-app/ui/visualization.js`

- [ ] **Step 1: Create visualization.js**

Extract from main.rs:
- Lines 21305-21331: `initFourierVisualization()`
- Lines 21331-21641: `draw()` animation loop
- Lines 21487-21498: `drawWave()`

Write to `tauri-app/ui/visualization.js`:
```javascript
// Animation state
let time = 0;
let animationId = null;
let circles = [];
let wave = [];

// Initialize visualization
function initFourierVisualization() {
    // ... from lines 21305-21331
}

// Main draw loop
function draw() {
    // ... from lines 21331-21641
}

// Draw wave trace
function drawWave(ctx, wave) {
    // ... from lines 21487-21498
}

// Fourier circle class
class FourierCircle {
    constructor(speed, radius, initialAngle) {
        // ...
    }
    draw(ctx, at) { /* ... */ }
    nextCenter(at) { /* ... */ }
}
```

- [ ] **Step 2: Verify visualization.js**

Run: `wc -l tauri-app/ui/visualization.js`
Expected: ~350 lines

---

### Task 7: Extract controls.js

**Files:**
- Read: `tauri-app/src/main.rs:46353-46860` (event listeners)
- Create: `tauri-app/ui/controls.js`

- [ ] **Step 1: Create controls.js**

Extract from main.rs:
- Lines 46353-46860: Event listener bindings for sidebar controls
- Mode switching logic
- Sample rate, wave count, speed controls
- Color customization

Write to `tauri-app/ui/controls.js`:
```javascript
// Mode switching
function switchToDrawMode() { /* ... */ }
function switchToSvgMode() { /* ... */ }

// Control initialization
function initControls() {
    // Sample rate
    document.getElementById('sampleRate').addEventListener('input', (e) => {
        // ...
    });
    
    // Wave count
    document.getElementById('waveCount').addEventListener('input', (e) => {
        // ...
    });
    
    // Speed control
    document.getElementById('speedControl').addEventListener('input', (e) => {
        // ...
    });
    
    // Colors
    document.getElementById('epicycleColor').addEventListener('input', (e) => {
        // ...
    });
    
    // Drawing tools
    document.getElementById('drawingTool').addEventListener('change', (e) => {
        // ...
    });
}

// Visualize button handler
function handleVisualize() {
    // ... from lines 46430-46489
}

// SVG file loading
function handleLoadSvg() {
    // ... from lines 46956-47003
}
```

- [ ] **Step 2: Verify controls.js**

Run: `wc -l tauri-app/ui/controls.js`
Expected: ~500 lines

---

### Task 8: Extract export.js

**Files:**
- Read: `tauri-app/src/main.rs:21680-21900` (export functions)
- Create: `tauri-app/ui/export.js`

- [ ] **Step 1: Create export.js**

Extract from main.rs:
- Lines 21680-21725: Export PNG
- Lines 21726-21900: Export JSON, GIF, HTML

Write to `tauri-app/ui/export.js`:
```javascript
// Export as PNG
async function exportPng(filePath) {
    const dataUrl = canvas.toDataURL('image/png');
    await tauriInvoke('save_canvas_as_png', { dataUrl, filePath });
}

// Export as JSON
async function exportJson(filePath) {
    await tauriInvoke('export_fourier_data', {
        data: fullFourierData,
        filePath,
        numSamples: fullFourierData.length
    });
}

// Export as GIF
async function exportGif(filePath, frames = 100, duration = 10) {
    await tauriInvoke('export_as_gif', {
        data: fullFourierData,
        filePath,
        frames,
        duration
    });
}

// Export as HTML
async function exportHtml(filePath) {
    await tauriInvoke('export_as_html', {
        data: fullFourierData,
        filePath
    });
}
```

- [ ] **Step 2: Verify export.js**

Run: `wc -l tauri-app/ui/export.js`
Expected: ~100 lines

---

### Task 9: Create Rust commands module

**Files:**
- Create: `tauri-app/src/commands/mod.rs`
- Create: `tauri-app/src/commands/drawing.rs`
- Create: `tauri-app/src/commands/svg.rs`
- Create: `tauri-app/src/commands/export.rs`
- Create: `tauri-app/src/commands/files.rs`

- [ ] **Step 1: Create commands/mod.rs**

```rust
pub mod drawing;
pub mod svg;
pub mod export_cmd;
pub mod files;
```

- [ ] **Step 2: Create commands/drawing.rs**

Extract from main.rs lines 53557-53578:
```rust
use fourier_svg::process_svg_path;
use crate::FourierData;

#[tauri::command]
pub fn process_drawing(path: String, num_sample: usize) -> Vec<FourierData> {
    let config = fourier_svg::FourierConfig::new(num_sample, num_sample);
    let result = process_svg_path(&path, &config);
    
    let mut sorted: Vec<_> = result
        .into_iter()
        .enumerate()
        .map(|(idx, d)| FourierData {
            s: d.frequency,
            r: d.radius,
            a: d.angle,
            idx,
        })
        .collect();
    sorted.sort_by(|a, b| b.r.partial_cmp(&a.r).unwrap());
    sorted
}
```

- [ ] **Step 3: Create commands/svg.rs**

Extract from main.rs lines 53580-53643:
```rust
use crate::{SvgPathInfo, SvgPathsResponse, FourierData};

#[tauri::command]
pub fn parse_svg_file(file_path: String) -> Result<SvgPathsResponse, String> {
    // ... from lines 53580-53629
}

#[tauri::command]
pub fn process_svg_path(path_data: String, num_sample: usize) -> Vec<FourierData> {
    // ... from lines 53641-53663
}

#[tauri::command]
pub fn get_svg_paths(file_path: String) -> Result<Vec<SvgPathInfo>, String> {
    // ... from lines 53857-53869
}
```

- [ ] **Step 4: Create commands/export.rs**

Extract from main.rs lines 53664-53710, 53797-53833, 53833-53857:
```rust
use crate::FourierData;

#[tauri::command]
pub fn export_fourier_data(
    data: Vec<FourierData>,
    file_path: String,
    num_samples: usize
) -> Result<(), String> {
    // ... from lines 53664-53710
}

#[tauri::command]
pub async fn export_as_gif(
    data: Vec<FourierData>,
    file_path: String,
    frames: usize,
    duration: f32
) -> Result<(), String> {
    // ... from lines 53797-53833
}

#[tauri::command]
pub async fn export_as_html(
    data: Vec<FourierData>,
    file_path: String
) -> Result<(), String> {
    // ... from lines 53833-53857
}
```

- [ ] **Step 5: Create commands/files.rs**

Extract from main.rs lines 53711-53797:
```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub timestamp: i64,
}

#[tauri::command]
pub async fn save_canvas_as_png(
    data_url: String,
    file_path: String
) -> Result<(), String> {
    // ... from lines 53711-53735
}

#[tauri::command]
pub async fn add_recent_file(
    file_path: String,
    file_name: String
) -> Result<Vec<RecentFile>, String> {
    // ... from lines 53736-53767
}

#[tauri::command]
pub async fn get_recent_files() -> Result<Vec<RecentFile>, String> {
    // ... from lines 53768-53797
}
```

- [ ] **Step 6: Verify commands module**

Run: `wc -l tauri-app/src/commands/*.rs`
Expected: ~300 lines total

---

### Task 10: Update main.rs

**Files:**
- Modify: `tauri-app/src/main.rs`

- [ ] **Step 1: Rewrite main.rs**

Replace the entire 54,000-line file with:
```rust
//! Tauri Fourier Visualizer Application

mod commands;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct FourierData {
    pub s: f32,
    pub r: f32,
    pub a: f32,
    pub idx: usize,
}

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

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long = "path")]
    svg_path: Option<String>,
    
    #[arg(short = 'f', long = "file")]
    svg_file: Option<String>,
    
    #[arg(short = 'i', long = "input")]
    input_file: Option<String>,
    
    #[arg(short = 's', long = "sample", default_value = "10240")]
    num_sample: usize,
    
    #[arg(short = 'w', long = "wave", default_value = "201")]
    num_wave: usize,
}

fn main() {
    let args = Args::parse();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::drawing::process_drawing,
            commands::svg::parse_svg_file,
            commands::svg::process_svg_path,
            commands::svg::get_svg_paths,
            commands::export_cmd::export_fourier_data,
            commands::export_cmd::export_as_gif,
            commands::export_cmd::export_as_html,
            commands::files::save_canvas_as_png,
            commands::files::add_recent_file,
            commands::files::get_recent_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify main.rs**

Run: `wc -l tauri-app/src/main.rs`
Expected: ~80 lines

---

### Task 11: Update tauri.conf.json

**Files:**
- Modify: `tauri-app/tauri.conf.json`

- [ ] **Step 1: Update frontendDist**

Add to tauri.conf.json:
```json
{
  "build": {
    "frontendDist": "../ui",
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  }
}
```

- [ ] **Step 2: Verify config**

Run: `cat tauri-app/tauri.conf.json | grep frontendDist`
Expected: `"frontendDist": "../ui"`

---

### Task 12: Test the refactored app

- [ ] **Step 1: Build the app**

Run: `cargo build -p tauri-app --features tauri 2>&1`
Expected: Successful compilation

- [ ] **Step 2: Run the app (if possible)**

Run: `cargo run -p tauri-app --features tauri`
Expected: App launches with UI

- [ ] **Step 3: Verify file sizes**

Run:
```bash
wc -l tauri-app/src/main.rs
wc -l tauri-app/src/commands/*.rs
wc -l tauri-app/ui/*.html tauri-app/ui/*.css tauri-app/ui/*.js
```
Expected: main.rs ~80 lines, commands ~300 lines, ui ~3000 lines

- [ ] **Step 4: Commit**

```bash
git add tauri-app/
git commit -m "refactor: split tauri-app into modular architecture

- Extract HTML/CSS/JS to ui/ directory
- Organize Rust commands into src/commands/ module
- Remove 38,000 lines of non-functional fantasy features
- Total: 54,032 lines → ~3,400 lines (94% reduction)"
```
