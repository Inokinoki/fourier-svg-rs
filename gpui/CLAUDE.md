# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fourier SVG is a Rust workspace that implements Fourier Transform visualization for SVG paths. It decomposes SVG shapes into rotating epicycles (circles) that trace the original path.

## Common Commands

### Building
```bash
# Build entire workspace
cargo build --workspace

# Build specific package
cargo build -p fourier-svg

# Build with features (e.g., Tauri app)
cargo build -p tauri-app --features tauri

# Release build
cargo build --release --workspace
```

### Testing
```bash
# Run all tests
cargo test --workspace

# Run tests for specific package
cargo test -p fourier-svg

# Run single test
cargo test test_name -- --exact
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting without changes (CI check)
cargo fmt -- --check

# Run linter
cargo clippy --workspace --all-targets

# Fix clippy warnings automatically
cargo clippy --workspace --all-targets --fix
```

### Running Applications
```bash
# CLI tool - generate HTML from SVG
cargo run -p fourier-cli -- -f ./test.svg

# CLI tool - generate GIF
cargo run -p fourier-cli -- -f ./test.svg --backend gif --frames 100

# CLI tool - export Fourier data to JSON
cargo run -p fourier-cli -- -f ./test.svg --backend export

# Tauri desktop app (requires system dependencies)
cargo run -p tauri-app --features tauri
```

### Tauri App System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

**macOS:**
```bash
brew install gtk+3
```

**Fedora:**
```bash
sudo dnf install webkit2gtk3-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel
```

## Workspace Architecture

The project is a Cargo workspace with 4 packages sharing dependencies defined in the root `Cargo.toml`:

```
fourier-svg-rs/
├── fourier-svg/      # Core library - all FFT and SVG processing logic
├── fourier-cli/      # CLI tool for HTML/GIF/JSON output
├── gpui-app/         # CLI with coefficient display
└── tauri-app/        # Interactive desktop application
```

### Core Library (`fourier-svg/`)

The central library that all other packages depend on. Contains:

**`path_util.rs`** - SVG path processing and FFT computation
- `build_path_from_svg()` - Parse SVG path string into lyon Path
- `compute_path_length()` - Calculate total path length for sampling
- `construct_sample_points()` - Generate evenly-spaced sample points along path
- `path_to_fft()` - Main entry: path → complex samples → FFT → coefficients

**`fft_drawer.rs`** - Core data structures
- `DrawData` struct: `{ frequency, radius, angle }` - Represents one epicycle
- `new_from_complex()` - Convert FFT complex coefficient to DrawData (polar coordinates)

**`visualizer/`** - Output format implementations
- `html_visualizer.rs` - Generate HTML/Canvas animations
- `gif_visualizer.rs` - Generate animated GIFs
- `export_visualizer.rs` - Save/load Fourier data as JSON

### Key Data Flow

The Fourier transform pipeline:

1. **Input**: SVG path string (`M 10 10 L 20 20 ...`) or file
2. **Parse**: `build_path_from_svg()` → lyon `Path`
3. **Sample**: `construct_sample_points()` → `Vec<Complex<f32>>` (x + yi)
4. **FFT**: `rustfft` processes samples → frequency domain coefficients
5. **Convert**: Complex → `DrawData` (polar: radius + angle)
6. **Sort**: By radius (largest circles first for visualization)
7. **Output**: HTML/GIF/JSON or real-time rendering

### The Epicycle Visualization

Each `DrawData` represents one rotating circle:
- **frequency**: How fast the circle rotates (positive = clockwise, negative = counter-clockwise)
- **radius**: Size of the circle (amplitude of that frequency component)
- **angle**: Initial phase offset

Circles are chained: each circle's center is the previous circle's edge point. The final point traces the reconstructed path.

### Tauri App Architecture

**Mode**: Desktop GUI with two input modes
- **SVG File Mode**: Load SVG via file dialog, select path from dropdown
- **Drawing Mode**: Freehand drawing on canvas with time-stamp capture

**Tauri Commands** (Rust backend callable from JS):
- `process_drawing(path, num_sample)` - Process hand-drawn path
- `parse_svg_file(file_path)` - Extract all paths from SVG file
- `process_svg_path(path_data, num_sample)` - Process selected SVG path

**UI**: Embedded HTML/JavaScript (generated via `generate_html()` in main.rs)

**Plugins Used**:
- `tauri-plugin-fs` - File system access
- `tauri-plugin-dialog` - File open dialogs

**Permissions**: Configured in `tauri.conf.json` under `plugins.fs` and `plugins.dialog`

## Code Style Rules

Enforced by CI via `cargo fmt -- --check` and `cargo clippy`:

1. **Function signatures**: Keep on one line unless exceeding line width
2. **Struct literals**: Use multi-line format for multiple fields
3. **Method chaining**: Use multi-line format when exceeding line width
4. **Unused variables**: Prefix with `_`
5. **Unused imports**: Remove them

### Common Pitfalls

**SVG crate type conversions**: `Attributes` returns `Value`, not `String`
```rust
// Wrong
id.clone()

// Correct
id.to_string()
```

**SVG tag constants**: Uppercase
```rust
svg::node::element::tag::SVG  // Correct
svg::node::element::tag::Svg  // Wrong
```

**Tauri plugin imports**: Don't import trait types if unused
```rust
use tauri_plugin_fs::FsExt;      // Remove if unused
use tauri_plugin_dialog::DialogExt;  // Remove if unused
```

## CI/CD Pipeline

### CI Workflow (`.github/workflows/ci.yml`)
Runs on every push and pull request to `master`, `main`, `develop`:
1. Tests on Ubuntu, Windows, macOS
2. Build with `--workspace` and `--all-features`
3. Format check: `cargo fmt -- --check`
4. Lint: `cargo clippy --workspace --all-targets -- -D warnings`

### Release Workflow (`.github/workflows/release.yml`)
Triggered on version tags (`v*.*.*`) or manual workflow dispatch:

**CLI Tools Build (`build-cli`):**
- Builds `fourier-cli` and `gpui-app` for:
  - Linux x86_64 and aarch64
  - Windows x86_64
  - macOS x86_64 and aarch64 (universal)
- Strips binaries and packages as `.tar.gz` (Linux/macOS) or `.zip` (Windows)

**Tauri App Build (`build-tauri`):**
- Uses `tauri-apps/tauri-action` for bundling
- Builds for:
  - Linux x86_64 and aarch64 (AppImage/deb)
  - Windows x86_64 (NSIS installer)
  - macOS x86_64 and aarch64 (DMG/app bundle)
- Installs platform-specific dependencies (WebKitGTK, GTK3)

**Artifacts Uploaded:**
All binaries and installers are uploaded to GitHub Releases when a version tag is pushed.

**Docker Build (`build-docker`):**
- Multi-platform images: linux/amd64, linux/arm64
- Pushes to Docker Hub on version tags
- Requires `DOCKER_USERNAME` and `DOCKER_PASSWORD` secrets

## Docker Build

For consistent builds without system dependencies:

```bash
docker build -t fourier-tauri-app .
docker run --rm -v $(pwd):/app -w /app fourier-tauri-app cargo build --release --features tauri -p tauri-app
```

Note: Tauri GUI requires X11/VNC for display when running in containers.
