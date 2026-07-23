# Pixesh

A minimal pixel art editor built with Rust and egui.

![screenshot](screenshots/s_1.png)

## Features

- **Drawing tools** — Brush, Eraser, Fill, Eyedropper, Select, Move
- **Copy / Paste** — Copy selection, paste at center, move pasted pixels
- **Layers** — Add/remove layers, toggle visibility
- **HSV color picker** — Saturation/Value field + Hue strip
- **Canvas resize / scale** — Change dimensions or scale the image
- **Crop to selection** — Crop canvas to selection bounds
- **Export PNG** — Save your artwork
- **Grid overlay** — Toggle pixel grid
- **Zoom & pan** — Scroll to zoom, arrows/middle-mouse to pan
- **Zoom to fit** — Auto-zoom on image load
- **Mirror** — Flip active layer horizontally/vertically with toolbar buttons
- **Undo / Redo** — Ctrl+Z / Ctrl+Shift+Z (up to 50 steps)
- **Multi-document tabs** — Open multiple images, Ctrl+Tab to switch

## Controls

### Tools

| Key | Action |
|-----|--------|
| `B` | Brush |
| `E` | Eraser |
| `F` | Fill (flood fill) |
| `R` | Rectangular select |
| `M` | Move (selection or canvas) |
| `Y` | Copy selection to clipboard |
| `P` | Paste clipboard at canvas center |
| `Alt` (hold) | Eyedropper (temporary) |
| `Right-click + drag` | Eraser |

### Color

| Key | Action |
|-----|--------|
| `W` | Darken color |
| `S` | Brighten color |
| `A` | Decrease opacity |
| `D` | Increase opacity |

### Selection

| Key | Action |
|-----|--------|
| `Delete` | Delete selected pixels |
| `Enter` | Crop canvas to selection |
| `Escape` | Deselect / close dialogs |

### Navigation

| Key | Action |
|-----|--------|
| `Scroll` | Zoom in/out |
| `Shift+Scroll` | Brush size (or scroll in brush size dialog) |
| `Arrow keys` | Pan canvas |
| `Shift+Arrow` | Pan canvas (4x speed) |
| `Middle mouse drag` | Pan canvas |

### File & Dialogs

| Key | Action |
|-----|--------|
| `Ctrl+S` | Export PNG dialog |
| `Ctrl+L` | Load image (opens in new tab) |
| `Ctrl+R` | Resize canvas dialog |
| `Ctrl+I` | Scale canvas dialog |
| `Ctrl+B` | Brush size dialog |
| `Ctrl+H` | Settings dialog |
| `Ctrl+W` | Toggle panels visibility |
| `Ctrl+D` | Deselect |
| `Ctrl+Tab` | Switch to next tab |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `G` | Toggle grid |

## Build

```bash
cargo build --release
```

Run with:

```bash
./target/release/pixesh
```

## Project Structure

- `src/main.rs` — Entry point, font & style setup
- `src/app/` — Core application logic
  - `mod.rs` — App state (layers, color, tools, undo, selection, multi-doc tabs)
  - `canvas.rs` — Compositing, pixel painting, Bresenham lines, flood fill, mirror
  - `history.rs` — Undo/redo with snapshots
  - `input.rs` — Keyboard shortcuts, zoom, pan, copy/paste
  - `io.rs` — Save/load PNG, layer management, canvas resize/scale/crop
  - `panel_canvas.rs` — Canvas rendering, tool dispatch
  - `panel_dialogs/` — Resize, Export, Brush Size, Panels, Settings, Scale dialogs
  - `panel_layers.rs` — Layers panel, HSV color picker
  - `panel_toolbar.rs` — Toolbar with tool icons, grid toggle, tab bar
  - `tools.rs` — Tool handlers (brush, fill, eyedropper, selection, move, paste)
- `src/color.rs` — HSV <-> RGB conversion
- `src/constants.rs` — Colors, sizes, Tool enum
- `src/ui.rs` — Custom widgets (button, icon button, checkbox, slider)
- `tex/` — Tool icons
- `screenshots/` — Screenshots
