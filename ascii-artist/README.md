# ASCII Artist

Real-time image-to-ASCII art converter built with Rust and egui.

## Features

- Side-by-side original image and ASCII preview
- Real-time updates as you adjust settings
- Customizable character ramp (supports Unicode, e.g., `░▒▓█`)
- Brightness, contrast, and invert controls
- Full RGB and ANSI 16-color modes
- Auto-fit resolution to window size
- Export: copy to clipboard, save as `.txt`, save as `.png`
- Adjustable PNG export scale (1x-4x)
- Dark and light background modes
- Embedded JetBrains Mono font for consistent PNG export

## Build

All build commands must be run from the `ascii-artist/` directory.

### Quick build (no Rust required)

Requires [Node.js](https://nodejs.org). This will automatically install the Rust toolchain if needed:

```
cd ascii-artist
npm run build
```

### Manual build

If you already have [Rust](https://rustup.rs) installed:

```
cd ascii-artist
cargo build --release
```

The binary will be at `target/release/ascii-artist.exe` (~9MB, no runtime dependencies).

## Usage

1. Launch `ascii-artist.exe`
2. Click **Load Image** or press `Ctrl+O` to open an image
3. Adjust settings in the right panel — the preview updates in real-time
4. Export using the toolbar buttons

Supported formats: PNG, JPEG, BMP, GIF (first frame), WebP, TIFF.

## Keybindings

| Key | Action |
|-----|--------|
| Ctrl+O | Open image |
| Ctrl+C | Copy ASCII to clipboard |

## Controls

| Control | Description |
|---------|-------------|
| Character Ramp | Characters from lightest to darkest |
| Brightness | Shift overall lightness (-1.0 to 1.0) |
| Contrast | Amplify light/dark differences (0.1 to 3.0) |
| Invert | Swap light and dark mapping |
| Font Size | Preview character size (4-24pt) |
| Color Mode | Off / Full RGB / ANSI 16 |
| Dark Background | Toggle dark/light preview background |
| Auto-fit | Auto-adjust columns to panel width |
| PNG Export Scale | Resolution multiplier for PNG export (1x-4x) |

## License

MIT
