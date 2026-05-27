# Tauri App Architecture Refactoring Design

## Goal

Refactor the 54,000-line monolithic `tauri-app/src/main.rs` into a modular architecture with separate frontend files and organized Rust backend, removing ~38,000 lines of non-functional "fantasy feature" code.

## Current State

- **54,032 lines** in a single `main.rs`
- HTML/CSS/JS embedded in Rust raw string literal (`r##"..."##`)
- 12 Tauri commands mixed with frontend code
- **~70% (38,000 lines) are non-functional fantasy features** (quantum simulation, holographic 3D, etc.)
- Real functionality is ~16,000 lines

## Target Architecture

```
tauri-app/
├── src/
│   ├── main.rs              # Tauri app setup + command registration
│   ├── commands/
│   │   ├── mod.rs           # Command module exports
│   │   ├── drawing.rs       # process_drawing command
│   │   ├── svg.rs           # parse_svg_file, process_svg_path, get_svg_paths
│   │   ├── export.rs        # export_fourier_data, export_as_gif, export_as_html
│   │   └── files.rs         # save_canvas_as_png, recent_files
│   └── lib.rs               # Public API exports
├── ui/                      # Frontend static files (embedded via Tauri)
│   ├── index.html           # Main HTML structure
│   ├── style.css            # All CSS styles
│   ├── app.js               # App initialization, state, event routing
│   ├── drawing.js           # Canvas drawing, input handling
│   ├── visualization.js     # Fourier animation, epicycle rendering
│   ├── controls.js          # UI controls, sliders, presets
│   └── export.js            # Export functions (PNG, JSON, GIF, HTML)
├── Cargo.toml
└── tauri.conf.json          # build.frontend_dist: "../ui"
```

## How It Works

### Tauri Asset Protocol

In `tauri.conf.json`:
```json
{
  "build": {
    "frontendDist": "../ui"
  }
}
```

Tauri v2 reads all files from the `ui/` directory, embeds them in the binary, and serves `index.html` as the entry point. No build step or bundler required.

### Frontend Files

**index.html** - Minimal HTML structure:
- Sidebar with control groups
- Canvas container
- Modal dialogs (help, shortcuts)
- Script imports

**style.css** - All CSS (~800 lines):
- Layout (sidebar, canvas, modals)
- Components (buttons, sliders, controls)
- Themes

**app.js** - Application entry point:
- State initialization
- Tauri invoke wrapper
- Event routing
- Keyboard shortcuts

**drawing.js** - Drawing input:
- Canvas mouse events
- Drawing tools (freehand, line, rectangle, ellipse)
- Undo/redo
- Preset shapes

**visualization.js** - Fourier animation:
- `initFourierVisualization()`
- `draw()` animation loop
- Wave rendering
- Zoom/pan

**controls.js** - UI controls:
- Mode switching (draw/SVG)
- Sample rate, duration sliders
- Wave count, speed controls
- Color customization

**export.js** - Export functions:
- PNG screenshot
- JSON data export
- GIF animation export
- HTML standalone export

### Rust Backend

**main.rs** - App setup:
```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::drawing::process_drawing,
            commands::svg::parse_svg_file,
            commands::svg::process_svg_path,
            commands::svg::get_svg_paths,
            commands::export::export_fourier_data,
            commands::export::export_as_gif,
            commands::export::export_as_html,
            commands::files::save_canvas_as_png,
            commands::files::add_recent_file,
            commands::files::get_recent_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**commands/drawing.rs**:
- `process_drawing(path: String, num_sample: usize) -> Vec<FourierData>`

**commands/svg.rs**:
- `parse_svg_file(file_path: String) -> Result<SvgPathsResponse, String>`
- `process_svg_path(path_data: String, num_sample: usize) -> Vec<FourierData>`
- `get_svg_paths(file_path: String) -> Result<Vec<SvgPathInfo>, String>`

**commands/export.rs**:
- `export_fourier_data(data, file_path, num_samples) -> Result<(), String>`
- `export_as_gif(data, file_path, frames, duration) -> Result<(), String>`
- `export_as_html(data, file_path) -> Result<(), String>`

**commands/files.rs**:
- `save_canvas_as_png(data_url, file_path) -> Result<(), String>`
- `add_recent_file(file_path, file_name) -> Result<Vec<RecentFile>, String>`
- `get_recent_files() -> Result<Vec<RecentFile>, String>`

## Features to Keep

### Core
- [x] Hand-drawn input (freehand, line, rectangle, ellipse)
- [x] SVG file loading with path selection
- [x] Fourier visualization with epicycle animation
- [x] Preset templates (circle, square, triangle, star, heart, spiral)

### Controls
- [x] Wave count slider
- [x] Animation speed control
- [x] Zoom control
- [x] Color customization (epicycle/trace)
- [x] Sample rate / duration
- [x] Undo/redo

### Export
- [x] PNG screenshot
- [x] JSON data export
- [x] GIF animation export
- [x] HTML standalone export

### Other
- [x] Recent files
- [x] Keyboard shortcuts

## Features to Remove

All ~160 "fantasy features" including:
- Quantum simulation, holographic 3D, AR overlay
- Biometric security, neural network, blockchain
- VR/360 viewer, satellite imagery, drone control
- Fusion reactor, space telescope, quantum internet
- Particle accelerator, DNA sequencer, weather simulation
- Time machine, robotics lab, Mars colony
- And 150+ more...

## Migration Strategy

### Phase 1: Create Archive Branch
- ✅ `archive/fantasy-features` branch preserves current state

### Phase 2: Extract Frontend Files
1. Create `ui/` directory
2. Extract CSS from `<style>` block → `ui/style.css`
3. Extract HTML body → `ui/index.html`
4. Extract JavaScript by logical sections → `ui/*.js`
5. Update `tauri.conf.json` to use `frontendDist`

### Phase 3: Extract Rust Commands
1. Create `src/commands/` module
2. Move each Tauri command to appropriate file
3. Update `main.rs` to register commands

### Phase 4: Clean Up
1. Remove all fantasy feature code
2. Remove unused state variables
3. Test all real functionality

## Testing

- Build with `cargo build --features tauri`
- Run with `cargo run --features tauri`
- Verify all real features work:
  - Drawing mode
  - SVG file loading
  - Fourier visualization
  - Export functions
  - Keyboard shortcuts

## File Size Estimates

| File | Current | After |
|------|---------|-------|
| main.rs | 54,032 lines | ~50 lines |
| commands/*.rs | - | ~300 lines |
| ui/index.html | - | ~2,000 lines |
| ui/style.css | - | ~800 lines |
| ui/*.js | - | ~3,000 lines |
| **Total** | **54,032** | **~6,150** |
