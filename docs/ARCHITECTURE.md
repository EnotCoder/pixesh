# Pixesh — Project Architecture

## Overview

Pixesh is a small pixel-art editor with animation, layers and effects. It is
written in Rust and uses the immediate-mode GUI library `egui` through
`eframe`. There is no retained widget tree and no separate state-management
layer: every frame walks the current application state and emits draw commands.

## Stack

| Area | Choice |
|------|--------|
| Language | Rust, edition 2024 |
| GUI | `eframe` / `egui` 0.31 |
| Renderer | `glow` (OpenGL) through the default `desktop` feature |
| Images | `image` crate (PNG and common input formats) |
| Native dialogs | `rfd`, optional (`gtk3` backend), used for opening images and choosing export folders |
| Preferences | `serde` + RON through `eframe` storage |
| Fonts and icons | Embedded in the binary with `include_bytes!` |

### Cargo features

| Feature | Enables |
|---------|---------|
| `persistence` | `eframe/persistence` — stores preferences, egui memory, panel sizes and window geometry |
| `rfd` | Native file/folder dialogs |
| `desktop` | Default feature: `glow`, bundled fonts, accessibility, Wayland/X11, persistence and `rfd` |

A build without the default features still compiles. In that case native file
dialogs are unavailable and preferences fall back to their defaults.

## Source layout

```
pixesh/
├── src/
│   ├── main.rs                  — native window options, application entry point
│   ├── lib.rs                   — run_native_app(), embedded font, global egui style
│   ├── constants.rs             — theme colours, dimensions, Tool/BrushShape/
│   │                              ExportBg/Edge/PanelId, NUM_PALETTE, linked_side()
│   ├── color.rs                 — RGB/HSV conversion and lerp_color()
│   ├── ui.rs                    — shared widgets and texture loading
│   └── app/
│       ├── mod.rs               — thin facade: module declarations and re-exports
│       ├── config.rs            — persisted Prefs, validation, eframe storage helpers
│       ├── effects.rs           — 17 effects ported from Pixelorama
│       ├── snow.rs              — animated falling-snow overlay for the canvas
│       ├── document/
│       │   ├── mod.rs           — Layer, Snapshot, Document, mascot canvas
│       │   ├── anim.rs          — frames, cels, playback helpers
│       │   ├── canvas.rs        — compositing, brush painting, flood fill, mirror, texture cache
│       │   ├── history.rs       — undo/redo snapshots
│       │   ├── io.rs            — layers, PNG export/import, resize, scale, flatten
│       │   ├── text.rs          — 5×7 bitmap font for the Text tool
│       │   └── tools.rs         — brush/select/move/fill/clipboard operations
│       ├── state/
│       │   ├── mod.rs           — PixeshApp struct and constructor
│       │   ├── actions.rs       — dialog state, hints, effect application, playback
│       │   ├── color.rs         — colour synchronisation and history
│       │   ├── layout.rs        — panel edges, visibility and draw order
│       │   ├── prefs.rs         — preference load/save
│       │   ├── tabs.rs          — open/close documents and drag-and-drop
│       │   └── update.rs        — eframe::App update loop and quit dialog
│       └── ui/
│           ├── mod.rs           — UI module declarations
│           ├── canvas_view.rs   — central canvas and tool dispatch
│           ├── drag.rs          — panel docking between window edges
│           ├── input.rs         — keyboard, mouse wheel, arrow pan, temporary eyedropper
│           ├── layers.rs        — layers + colour picker panel
│           ├── status.rs        — status bar
│           ├── timeline.rs      — animation timeline
│           ├── toolbar.rs       — toolbar and document tabs
│           └── dialogs/
│               ├── mod.rs       — dialog dispatch
│               ├── effects.rs   — effects panel
│               ├── export.rs    — PNG export dialog
│               ├── panels.rs    — panel visibility dialog
│               ├── resize.rs    — canvas resize dialog
│               ├── scale.rs     — image scale dialog
│               ├── settings.rs  — settings dialog
│               ├── text.rs      — text tool dialog
│               └── welcome.rs   — welcome/quick-start dialog
├── img/                         — logo and mascot images
├── tex/                         — tool icons and the embedded font
└── docs/                        — architecture and UI reference
```

## Startup and frame flow

```
main()
  └─ pixesh::run_native_app()
       ├─ load embedded font
       ├─ install the global dark, square-corner egui style
       ├─ PixeshApp::new()
       ├─ PixeshApp::load_prefs(storage)
       └─ eframe::App::update(ctx, frame)  — once per frame
            ├─ record last_time
            ├─ handle_input()              — shortcuts, wheel zoom/brush, arrow pan, Alt eyedropper
            ├─ update_playback()           — advance playing animations
            ├─ handle_dropped_files()      — open dropped images as new tabs
            ├─ for panel in draw_order():
            │    ├─ ui_toolbar()
            │    ├─ ui_status()
            │    ├─ ui_timeline()
            │    └─ ui_layers()
            ├─ ui_canvas()                 — central area and tool dispatch
            ├─ ui_dialogs()                — floating dialogs in the foreground
            ├─ draw_drop_overlay()         — “open image” overlay while dragging files
            ├─ ui_panel_drag()             — dock-zone highlights while dragging a panel
            └─ quit confirmation           — only when there are unsaved documents
```

### Why panel order is computed

egui carves panels out of the remaining window rectangle in the order in which
they are created. `PixeshApp::draw_order()` therefore returns horizontal strips
first (top/bottom, grouped by edge) and side panels last (left/right). This
keeps the toolbar and status bar at full window width and prevents the layers
panel from overlapping them. Within one edge, `p_order()` defines a stable
canonical order so panels do not jump around.

## Data model

### `Layer`

```rust
struct Layer {
    name: String,
    cels: Vec<Arc<Vec<Color32>>>, // one pixel buffer per frame
    visible: bool,
}
```

Each animation frame has its own cel (pixel buffer) per layer. `Arc` allows
undo snapshots and layer duplication to share buffers until a buffer is
modified.

### `Document`

One `Document` is one tab. It owns everything that belongs to a single image:

- canvas: `name`, `layers`, `active_layer`, `width`, `height`;
- animation: `frames`, `active_frame`, `playing`, `fps`, `last_play`;
- history: `undo_stack`, `redo_stack`;
- display cache: `canvas_dirty`, `display_cell`, `display_buf`, `tex`;
- selection and clipboard: `sel`, `sel_start`, `sel_end`, `sel_buffer`,
  `sel_tex`, `sel_move_*`, `clipboard`, `clip_w`, `clip_h`, `pasting`;
- whole-layer and selection transforms: `transforming`,
  `transform_orig_rect`, `transform_corner`;
- view: `zoom`, `pan`, `grid`, `needs_zoom_fit`;
- pointer tracking: `last_px_primary`, `last_px_secondary`, `mid_pan_pos`;
- file state: `unsaved`, `export_path`, `export_name`.

`Document::new()` starts with a 16×16 blank canvas and one “Background” layer.
`Document::new_logo()` is the default startup document: a 32×32 canvas filled
with the pixel-art mascot from `img/pixel_logo.png`. `Document::new_sized()`
creates a blank document with an explicit size.

### `Snapshot`

An undo entry captures the layers and cels, the active layer/frame, the frame
count, canvas dimensions, selection/paste state and the clipboard. Layer cels
are `Arc` clones, so a snapshot is cheap until the live buffer is modified.
The undo stack is capped at 50 entries.

### `PixeshApp`

Application-wide state lives on `PixeshApp`:

- documents: `docs: Vec<Document>`, `active_tab`;
- colour: `color`, `hsv_*`, `rgb_*` — all representations are kept in sync;
- tools: `tool`, `brush`, `brush_shape`, temporary tool slots
  (`tool_saved`, `tool_saved_shift`);
- texture caches for icons, the colour picker and the canvas;
- dialog flags (`show_resize`, `show_export`, `show_settings`, `show_text`,
  `show_effects`, ...);
- panel visibility and edge assignment (`show_top_panel`,
  `toolbar_edge`, ...);
- panel-drag state (`dragging_panel`, `drop_edge`, `drag_from_edge`);
- preferences that are not panel geometry (`arrow_speed`, `zoom_speed`,
  `welcome_show_again`);
- colour history, layer rename state, text-tool state, status hint and
  effects-preview cache;
- miscellaneous UI state (`cursor_px`, `close_handled`, `logo_easter_egg`).

### Multi-document behaviour

Opening a file always adds a new tab and activates it. Closing a tab adjusts
`active_tab`; closing the last tab replaces it with a fresh mascot document
instead of leaving the application with no canvas. The toolbar renders one tab
per document with an unsaved marker (`*`) and a close button.

## Editing pipeline

### Canvas and compositing

- `composite()` flattens visible layers for the active frame. Transparent
  pixels are skipped; the topmost non-transparent pixel wins.
- `composite_at(frame)` does the same for an arbitrary frame (used by the
  timeline and sprite-sheet export).
- `composite_display()` additionally bakes the checkerboard pattern into the
  display buffer and blends semi-transparent pixels against it. This buffer is
  uploaded to an egui texture with `TextureOptions::NEAREST`.
- The display buffer is rebuilt only when `canvas_dirty` is set, when the
  checkerboard cell size changes with zoom, or while a selection/canvas move or
  transform preview is active.

### Painting

- `paint_pixel()` applies a square or round brush mask to the active layer and
  respects the current selection rectangle.
- `paint_line()` uses Bresenham between the previous and current pointer
  positions so fast strokes have no gaps.
- `flood_fill()` is a stack-based four-neighbour fill, clipped to the selection.
- `mirror_horizontal()` / `mirror_vertical()` flip the active layer in place.
- `screen_to_pixel()` converts a screen position into canvas pixel coordinates.

### Undo/redo

`push_undo()` is called at the start of a stroke or destructive command, not on
every drag frame. Consequently, undo reverts an entire stroke. Redo is cleared
whenever a new undo entry is pushed. `restore_snapshot()` resets selection and
texture state so the next frame rebuilds the canvas.

### Selection and clipboard

- Dragging with the Select tool creates a rectangular marquee.
- Dragging inside an existing selection lifts its pixels into `sel_buffer` and
  moves them; releasing commits the move.
- `Ctrl+C` / `Y` copies the selected pixels. `Ctrl+V` / `P` creates a floating
  paste that is committed on `Enter`, by switching tools, or by starting
  another operation.
- `Delete` clears the selection. `Escape` cancels a paste, a transform, a
  rename or an open dialog before finally deselecting.
- `Enter` crops the canvas to the selection when nothing is being pasted.

### Move and Transform

- Move drags the selection when the press starts inside it, otherwise it moves
  the whole active layer.
- Transform (`X` with a selection) exposes four corner handles. Dragging a
  corner resizes the selection rectangle; the result is committed when the
  drag ends. `Escape` cancels the transform and restores the previous tool.

### Text

The Text tool opens a dialog with a bitmap-font preview. `document/text.rs` contains a
5×7 ASCII font (characters 32–126); text is rasterised directly into the active
layer at the clicked canvas position, with an integer scale of 1–8.

## Animation

Each `Layer` stores one cel per frame. Frame operations live in `document/anim.rs`:

- `add_frame_blank()` inserts a transparent frame after the current one;
- `duplicate_frame()` copies the current frame;
- `delete_frame()` removes the current frame (the last frame cannot be deleted);
- `move_frame()` reorders frames;
- `set_active_frame()` marks the canvas dirty and drops the texture cache.

Playback is driven by `update_playback()`: while `playing` is true, the active
frame advances at `fps` (1–60) and a repaint is requested. The timeline panel
provides Play/Pause, Stop, FPS, frame navigation and add/duplicate/delete/move
controls.

## Effects

Effects are ported from the open-source
[Pixelorama](https://github.com/Orama-Interactive/Pixelorama) project. The
original GLSL shaders are reimplemented as CPU code over `Vec<Color32>`.

| Category | Effects |
|----------|---------|
| Transform | Offset & Scale, Mirror Image, Rotate Image, Flat to Isometric |
| Color | Invert Colors, Desaturation, Adjust Hue/Sat/Value, Adjust Brightness/Contrast, Color Curves, Palettize, Posterize, Gradient Map |
| Procedural | Outline, Drop Shadow, Gradient |
| Blur | Pixelize, Gaussian Blur |

### Module structure (`src/app/effects.rs`)

- `Cat` — effect category.
- `EffectDef` — name, category and parameter definitions.
- `PT` / `ParamDef` — parameter type (integer slider, float slider, checkbox,
  colour, segmented choice) and its range.
- `EffectParams` — current values, stored **by parameter name** rather than by
  index, so new parameters can be added to `EFFECTS` without touching the
  application logic.
- `EFFECTS` — static table of all 17 effects. The order of `EffectKind`
  variants must match this table (enforced by a test).
- `apply(&mut pixels, w, h, &params, selection)` — the single entry point for
  applying an effect. It honours an optional selection rectangle.

### Panel and application semantics

The effects panel is opened by the sparkle button in the toolbar or with
`Ctrl+Shift+E`. It provides:

- category tabs and an effect list;
- live parameter controls;
- a live preview (the source is downscaled to at most 64 px on its longest
  side, and recalculation is throttled for large images);
- “All layers” and “All frames” scope checkboxes;
- Reset / Apply / Close.

`PixeshApp::apply_effect_to()` first computes every result into a temporary
buffer. If nothing changed, no undo entry is created. Otherwise it calls
`push_undo()` once and commits all changed cels, so a single `Ctrl+Z` reverts
the whole effect application.

Two effects intentionally differ from the originals:

- **Palettize** builds its palette from the image itself using median cut
  instead of a fixed palette.
- **Color Curves** is a per-channel gamma plus black/white points rather than an
  interactive curve editor.

## Panel layout and docking

- `PanelId` identifies the four draggable panels: `Toolbar`, `Status`,
  `Timeline`, `Layers`.
- `Edge` identifies the window edge: `Top`, `Bottom`, `Left`, `Right`.
- Horizontal strips (toolbar, status bar, timeline) may move only between top
  and bottom. The layers panel may move only between left and right.
- Each panel stores its own edge (`toolbar_edge`, `status_edge`,
  `timeline_edge`, `layers_edge`). `config::clamp_edge()` repairs forbidden
  values loaded from disk.
- Dragging starts from a 22×22 grip at the beginning of each panel. While
  dragging, the allowed drop zones (140 px from the relevant window edge) are
  highlighted. Dropping on a different valid edge reassigns the panel.
- Panels are resizable. egui stores their size in `egui::Memory`, keyed by
  `PanelId::ui_id(edge)`. Including the edge in the id prevents a width from
  being reused as a height after moving a panel to another edge.

## Preferences and persistence

Preferences are stored through eframe's storage (RON, `app.ron`). egui itself
persists window geometry, panel sizes and other memory; Pixesh adds a custom
`pixesh_prefs` value containing:

```rust
struct Prefs {
    show_top_panel: bool,
    show_right_panel: bool,
    show_status_bar: bool,
    show_timeline: bool,
    toolbar_edge: Edge,
    layers_edge: Edge,
    status_edge: Edge,
    timeline_edge: Edge,
    welcome_show_again: bool,
    arrow_speed: f32,
    zoom_speed: f32,
    brush: f32,
    brush_shape: BrushShape,
    tool: Tool,
    rgba: [u8; 4],
    color_history: Vec<[u8; 4]>,
}
```

Storage locations:

- Linux: `~/.local/share/pixesh/app.ron`
- Windows: `%APPDATA%\pixesh\data\app.ron`

`eframe::App::auto_save_interval()` is set to 10 seconds, and eframe also saves
on exit. On load, `Prefs::sanitize()` clamps speeds and brush size, truncates
the colour history and repairs illegal panel edges. Missing fields fall back to
`Default` thanks to `#[serde(default)]`; a corrupt file also yields defaults.

Dialogs and transient tools are deliberately **not** restored: all dialog flags
start closed, layer renaming is cleared, and `Tool::Effects` falls back to
Brush. This prevents a dialog or a non-drawing tool from appearing by itself on
the next launch.

## Rendering and performance notes

- **Copy-on-write pixels.** `Layer.cels` are `Arc<Vec<Color32>>`. Undo, layer
  duplication and frame duplication clone only the `Arc`. Painting calls
  `Arc::make_mut()`, which clones the buffer only when it is shared.
- **Cached canvas texture.** The composited display buffer is rebuilt only when
  the document is dirty or the checkerboard scale changes.
- **Checkerboard baked in.** `composite_display()` writes checker and alpha
  blending directly into the display buffer, avoiding per-cell rectangles at
  draw time.
- **Bresenham strokes.** `paint_line()` fills gaps between pointer samples.
- **Pointer reset.** `last_px_*` is cleared when the pointer leaves the canvas
  or a dialog opens, preventing spurious connecting lines when it re-enters.
- **Effect previews.** The preview source is downscaled with nearest-neighbour
  sampling to 64 px. Small layers are recomputed every frame; large layers only
  when parameters change and at most every 0.4 seconds.
- **Palettize cache.** Palette lookup is cached by pixel colour in a
  `HashMap<[u8; 4], Color32>`, so nearest-palette search runs once per unique
  colour rather than once per pixel.
- **Nearest-neighbour textures.** Icons and the canvas use
  `TextureOptions::NEAREST` to preserve crisp pixel-art edges.
- **Embedded assets.** Icons, the mascot and the font are compiled into the
  binary; no asset directory is needed at runtime.

## Tests

Run the test suite with:

```bash
cargo test
```

Unit tests cover, among other things:

- aspect-ratio linking (`constants.rs`);
- preference round-tripping, defaults, sanitising and corrupt files
  (`config.rs`);
- effect table ordering, parameter reachability, selection handling and
  “all effects run” (`effects.rs`);
- PNG loading, failure handling and export helpers (`document/io.rs`);
- document/tab management, panel order, panel edges, preferences and colour
  synchronisation (`state/color.rs`);
- docking edge resolution (`ui/drag.rs`);
- clipboard, paste, select-all and delete semantics (`document/tools.rs`).

## Known limitations

- The undo stack is per document and capped at 50 snapshots.
- Loading an image resets the document to a single animation frame.
- PNG export is the only image export format.
- `composite()` (without the checkerboard) is used only for the eyedropper and
  export; the on-screen canvas always uses the display buffer.
- The wheel zooms the canvas only while no dialog is open; otherwise it scrolls
  the dialog's scroll area.
