# Fourier SVG Painter

Use the Fourier Transform to draw an SVG path with rotating circles (epicycles).

![Preview for a kiwi](preview.gif)

## How It Works

This tool uses the [Discrete Fourier Transform (DFT)](https://en.wikipedia.org/wiki/Discrete_Fourier_transform) to decompose an SVG path into a series of rotating circles (epicycles). The key steps are:

1. **Path Sampling**: The SVG path is sampled at evenly-spaced points along its length, converting the continuous path into discrete complex numbers where the x-coordinate becomes the real part and the y-coordinate becomes the imaginary part.

2. **Fourier Transform**: The sampled points are transformed using FFT (Fast Fourier Transform) to compute the frequency components. Each frequency component represents a rotating circle with a specific:
   - **Frequency**: How fast the circle rotates
   - **Radius**: The size of the circle (amplitude)
   - **Phase**: The starting angle of rotation

3. **Reconstruction**: By summing up all these rotating circles (from slowest to fastest), the original path can be approximated. Using more circles (waves) produces a more accurate representation.

The result is a mesmerizing animation where spinning circles trace out the original shape!

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.56 or later)

### Build

```bash
cargo build --release
```

## Usage

### From an SVG File

Read the first path from an SVG file:

```bash
cargo run --release -- -f ./test.svg
```

### From an SVG Path String

Provide an SVG path string directly:

```bash
cargo run --release -- -p "M 100 100 L 200 100 L 200 200 Z"
```

Or from a file:

```bash
cargo run --release -- -p "$(cat ./test.svg.txt)"
```

### Options

| Option | Short | Long | Description | Default |
|--------|-------|------|-------------|---------|
| SVG file | `-f` | `--file` | Path to an SVG file | - |
| SVG path | `-p` | `--path` | SVG path string | - |
| Sample points | `-s` | `--sample` | Number of sample points for path discretization | 10240 |
| Waves | `-w` | `--wave` | Number of epicycles (Fourier components) to use | 201 |

### Examples

```bash
# Use default settings
cargo run --release -- -f ./test.svg

# Use more sample points for higher precision
cargo run --release -- -f ./test.svg -s 20480

# Use fewer waves for a simpler animation
cargo run --release -- -f ./test.svg -w 51

# Combine options
cargo run --release -- -f ./test.svg -s 15000 -w 301
```

## Output

The program generates an `output.html` file containing an interactive canvas animation. Open it in any modern web browser to see the Fourier series drawing the path in real-time.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Some ideas for future improvements:

- Add more render backends (e.g., GIF, video export)
- Support for multiple paths in a single SVG
- Interactive controls for the HTML visualization
- Color customization options
