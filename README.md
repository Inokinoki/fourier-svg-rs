# Fourier SVG Painter

Use Fourier Transform to draw an SVG Path with animated visualizations.

![Preview for a kiwi](preview.gif)

## Features

- **Multiple Rendering Backends**:
  - **HTML** - Generate HTML/Canvas animations (default)
  - **GIF** - Export as animated GIF images
  - **Tauri** - Desktop application with native window
  - **gpui** - GPU-accelerated rendering (experimental)

- **SVG Path Processing** - Accept SVG files or path strings
- **Configurable** - Adjust sample points, wave count, and output settings
- **Cross-platform** - Works on Linux, macOS, and Windows

## Build

Use cargo to build it:

```bash
cargo build --release
```

### Building with optional backends

```bash
# Build with Tauri backend
cargo build --release --features tauri-backend

# Build with gpui backend
cargo build --release --features gpui-backend
```

## Run

### Basic usage

Accept a svg file and take the first path as the target:

```bash
cargo run --release -- -f ./test.svg
```

Or provide a string with SVG path format:

```bash
cargo run --release -- -p "$(cat ./test.svg.txt)"
```

### Using different backends

Generate GIF animation:

```bash
cargo run --release -- -f ./test.svg --backend gif --frames 100
```

Generate HTML (default):

```bash
cargo run --release -- -f ./test.svg --backend html
```

Use Tauri desktop app:

```bash
cargo run --release --features tauri-backend -- -f ./test.svg --backend tauri
```

Use gpui backend:

```bash
cargo run --release --features gpui-backend -- -f ./test.svg --backend gpui
```

### Options

```
Options:
  -p, --path <SVG_PATH>      Draw an SVG path in string
  -f, --file <SVG_FILE>      Draw the first SVG path in file
  -s, --sample <NUM_SAMPLE>  Use how many sample points to draw the path [default: 10240]
  -w, --wave <NUM_WAVE>      Use how many waves to draw the path [default: 201]
  -b, --backend <BACKEND>    Rendering backend: html, gif, tauri, gpui [default: html]
  -o, --output <OUTPUT>      Output file name (without extension) [default: output]
      --frames <FRAMES>      Number of frames for GIF output [default: 100]
  -h, --help                 Print help
  -V, --version              Print version
```

## Docker

Build and run using Docker:

```bash
# Build the image
docker build -t fourier-svg .

# Run with a file
docker run --rm -v $(pwd)/test.svg:/data/test.svg fourier-svg -f test.svg --backend gif
```

## CI/CD

This project uses GitHub Actions for:
- **Continuous Integration** - Automated testing on multiple platforms
- **Release Automation** - Automatic binary builds and releases
- **Docker Images** - Multi-architecture container images

Releases are automatically created when tags are pushed (e.g., `v1.0.0`).

## How it works

The program uses Fourier Transform to decompose an SVG path into a series of rotating circles that trace out the original shape. The visualization shows:

1. Multiple circles rotating at different frequencies
2. Each circle's radius and speed determined by FFT coefficients
3. The path traced by the final circle reconstructs the original shape

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
