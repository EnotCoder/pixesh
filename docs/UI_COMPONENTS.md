# Pixesh — UI Components

This document describes the visible UI, the shared widgets and the interaction
model. It complements [ARCHITECTURE.md](ARCHITECTURE.md), which covers the data
model and internals.

## Theme (`src/constants.rs`)

| Constant | Value | Purpose |
|----------|-------|---------|
| `BG` | `rgb(24, 24, 24)` | Canvas background (darkest surface) |
| `PANEL` | `rgb(32, 32, 32)` | Panel and dialog background |
| `PANEL_LIGHT` | `rgb(44, 44, 44)` | Lighter controls, frame thumbnails |
| `BORDER` | `rgb(80, 80, 80)` | All widget borders |
| `TEXT` | `rgb(220, 220, 220)` | Primary text |
| `ACCENT` | `rgb(200, 120, 60)` | Active/selected state |
| `HOVER` | `rgb(60, 60, 60)` | Hover state |
| `DIM` | `rgb(120, 120, 120)` | Secondary text and inactive marks |
| `FONT_SZ` | `20.0` | Base font size |
| `CHAR_W` | `11.0` | Approximate character width for layout estimates |
| `ROW_H` | `22.0` | Base row height; buttons use `ROW_H + 16 = 38 px` |
| `PANEL_PAD` | `12.0` | Panel inner padding |
| `CHECKER_PX` | `8.0` | Checkerboard square size on screen |
| `MAX_SIDE` | `4096` | Maximum canvas side in pixels |

`NUM_PALETTE` defines the nine preset colours bound to keys `1`–`9`:
red, green, blue, yellow, cyan, magenta, white, gray, black.

The global egui style (`src/lib.rs`) uses a dark palette, an embedded pixel
font, `item_spacing = (10, 6)`, `button_padding = (10, 6)` and zero corner
radius everywhere. Custom widgets also paint square corners explicitly.

## Shared widgets (`src/ui.rs`)

### `load_icon_texture(ui, name, bytes) -> TextureHandle`

Decodes an embedded PNG with the `image` crate and uploads it to egui with
`TextureOptions::NEAREST`. If decoding fails, a 1×1 magenta texture is used so
the UI stays usable and the missing asset is obvious.

### `btn_min_w(ui, label, min_w) -> bool`

Text button with a minimum width.

- Width: `max(label.len() * CHAR_W * (font_size / FONT_SZ) + 28, min_w)`
- Height: `font_size + 16`
- Font: inherited `TextStyle::Button`, falling back to `FONT_SZ`
- Border: 4 px `BORDER`, painted inside
- States: normal `PANEL`, hover `HOVER`, pressed `ACCENT`
- Press effect: content moves down 2 px; a 2 px shadow is drawn when idle

Used by dialog buttons and the timeline controls.

### `toggle_btn(ui, label, active) -> bool`

Segmented/selection button, visually identical to `btn_min_w` but with a
minimum width of 80 px and a fixed height of `ROW_H + 16 = 38 px`.

Used by the timeline (Play/Pause/Stop), the export dialog (scale, background,
frame mode) and effect categories.

### `icon_btn_tip(ui, tex_id, active, tip) -> bool`

Square icon button.

- Size: `ROW_H + 16 = 38 × 38 px`
- Border: 4 px `BORDER`
- States: active `ACCENT`, hover `HOVER`, idle `PANEL`
- Press effect: 2 px downward offset, idle shadow
- Tooltip: `tip`

Used throughout the toolbar and the layers panel.

### `separator(ui)`

Vertical divider.

- Allocation: 4 × `ROW_H + 16 = 4 × 38 px`
- Painted line: 4 px, `BORDER`

Note that egui adds `item_spacing.x` before and after this allocation, so a
separator already introduces visual breathing room. Additional `add_space()`
calls around it are usually unnecessary.

### `check_row(ui, label, value, font, set)`

Pixel-art checkbox row used in dialogs and the effects panel.

- Checkbox: 18 × 18 px
- Row height: `19 * font + 12`
- Text: `19 * font`, `TEXT`
- Border: 3 px `BORDER`, outside
- Checked fill: `ACCENT`, inset 4 px
- Checkbox and label are treated as one centred group, capped at 170 px wide
- Clicking either the row or the box toggles the value

## Panel system

Four panels can be shown, hidden, resized and moved between the edges allowed
for their shape:

| Panel | Shape | Allowed edges | Default |
|-------|-------|---------------|---------|
| Toolbar | Horizontal strip | Top / Bottom | Top |
| Status | Horizontal strip | Top / Bottom | Bottom |
| Timeline | Horizontal strip by default; vertical when placed on a side | Top / Bottom / Left / Right | Bottom |
| Layers | Vertical list | Left / Right | Right |

Every panel starts with a 22 × 22 grab handle (six dots in two columns). While
a panel is dragged, valid drop zones near the window edges are highlighted.
Releasing over another valid edge docks the panel there.

Panels are resizable by dragging their outer edge. egui persists their sizes;
the edge is part of the panel id so a former width cannot be reused as a height
after docking.

## Toolbar (`src/app/ui/toolbar.rs`)

The toolbar is a `TopBottomPanel` on its stored edge, filled with `PANEL` and
without egui's separator line. Its body contains a grab handle, the main tool
row, the document tab bar and a 4 px bottom border.

### Main tool row

| # | Element | Type | Notes |
|---|---------|------|-------|
| 1 | Logo | Texture | `img/logo.png`, 42 × 42; clicking starts a short spin animation |
| 2 | Separator | `separator()` | |
| 3 | Brush | `icon_btn_tip` | Tooltip “Brush (B)” |
| 4 | Eraser | `icon_btn_tip` | “Eraser (E)” |
| 5 | Fill | `icon_btn_tip` | “Fill (F)” |
| 6 | Eyedropper | `icon_btn_tip` | “Eyedropper (A)” |
| 7 | Select | `icon_btn_tip` | “Select (R)” |
| 8 | Move | `icon_btn_tip` | “Move (M)” |
| 9 | Text | Custom 38 × 38 | “T”, opens the Text dialog on canvas click |
| 10 | Effects | Custom 38 × 38 | Pixel sparkle icon, opens the effects panel |
| 11 | Separator | `separator()` | |
| 12 | Clear | `icon_btn_tip` | Clears every cel of every layer, with undo |
| 13 | Separator | `separator()` | |
| 14 | Grid | Custom checkbox | 18 px checkbox + “Grid” label; toggles the grid |
| 15 | Mirror H | `icon_btn_tip` | Mirrors the active layer horizontally |
| 16 | Mirror V | `icon_btn_tip` | Mirrors the active layer vertically |
| 17 | Zoom | Measured text | `Zoom: {zoom:.2}`, 20 pt, real text width measured with the font atlas |
| 18 | Brush size | `DragValue` | Only for Brush/Eraser; label “Size:”, 28 pt value font, 1–4096 |
| 19 | Brush shape | `icon_btn_tip` | Only for Brush/Eraser; round and square icons |

The zoom/brush section uses a locally tighter `item_spacing.x = 6.0`. The zoom
text width is measured from the actual galley rather than estimated with
`CHAR_W`; otherwise the allocated rectangle leaves a noticeable empty gap.

### Document tabs

- Tab height: 28 px.
- Label width is estimated as `len * CHAR_W * (18 / FONT_SZ) + 24`.
- Each tab has a 28 × 28 close button; the close icon uses the `clear` texture.
- The active tab fades towards `HOVER`; inactive tabs use `PANEL`.
- An unsaved document is shown as `*name`.
- The trailing 28 × 28 “+” button creates `Untitled N` and activates the new tab.
- Closing the final tab replaces it with a fresh 32 × 32 mascot document.

## Layers panel (`src/app/ui/layers.rs`)

A resizable side panel, default width 280 px (min 120, max 500), filled with
`PANEL`, with a grab handle.

### Layer list

- Header: “Layers”, 30 pt.
- Layers are listed from topmost to bottommost; the active layer is highlighted.
- Row height: `ROW_H * 1.5 + 16 = 49 px`.
- Visibility checkbox: 14 × 14 px, `PANEL_LIGHT` base with an animated `ACCENT`
  fill when visible.
- Layer name: 30 pt. Double-click starts inline renaming; `Enter` commits,
  `Escape` cancels, losing focus commits a non-empty name.
- Clicking a row selects that layer.

### Layer actions

Four 38 × 38 icon buttons with tooltips:

- Add Layer
- Remove Layer (disabled at the final layer by the underlying operation)
- Duplicate Layer
- Flatten Layers

### Colour section

- Header: “Color”, 30 pt.
- Preview: 100 × 100 px, border brightens on hover.
- Readout: `R`, `G`, `B`, `A` values in four rows.
- History: up to four recent colours, 22 × 22 px swatches.
- SV field: 128 × 128 generated texture, displayed at up to 180 px; click or
  drag to set saturation/value.
- Hue strip: 14 px wide vertical gradient; click or drag to set hue.
- Alpha strip: 14 px wide vertical preview against a checkerboard; click or
  drag to set opacity.

All colour controls write back to `color`, `rgb_*` and `hsv_*` together, so the
picker and the numeric readout never disagree.

## Timeline (`src/app/ui/timeline.rs`)

The timeline can live on the top/bottom or left/right edge. On a horizontal
edge it is a resizable bottom/top panel, default height 150 px (min 120, max
70% of the screen). On a side edge it is a resizable side panel, default width
260 px (min 180, max 520).

### Header

- Grab handle.
- “Frames” title, 26 pt.
- Separator.
- `Play` / `Pause` toggle, highlighted while playing.
- `Stop` button, which stops playback and returns to frame 1.
- `FPS` label and a 56 × 30 `DragValue`, range 1–60.
- Separator and the current frame counter, e.g. `3/12`.
- Right-aligned controls: `<`, `>`, `Add`, `Dup`, `Del`.

### Frame thumbnails

- Horizontally scrollable strip.
- Thumbnail: 60 × 60 px with an 18 px caption area.
- Frame number under each thumbnail; the active frame uses `ACCENT`.
- Active border: 4 px `ACCENT`; inactive border: 2 px `BORDER`.
- Clicking a thumbnail activates that frame.

## Status bar (`src/app/ui/status.rs`)

A horizontal `TopBottomPanel` on its stored edge with an 8 px top border.

Left to right:

- Tool name (`Brush`, `Eraser`, `Fill`, `Dropper`, `Select`, `Move`, `Text`,
  `Transform`, `Effects`).
- Canvas size: `Canvas: WxH`.
- Frame: `Frame: current/total`.
- Cursor position: `Pos: (x, y)`, or `Pos: --` outside the canvas.
- Zoom: `Zoom: N%`, calculated as `zoom * 12.5`.
- Active layer name: `Layer: name`.

Pushed to the right edge:

- Brush size: `Brush: N`.
- `Unsaved` in `ACCENT` when the document has unsaved changes.
- A transient status hint (for example “nothing selected” or “clipboard is
  empty”), shown for about 2.5 seconds.

## Canvas (`src/app/ui/canvas_view.rs`)

A `CentralPanel` filled with `BG`.

- The canvas texture is centred in the available area and offset by the pan.
- Zoom is clamped to `0.1..=60.0`.
- After opening a file or changing the canvas size, `needs_zoom_fit` selects a
  zoom that fits the canvas and resets the pan.
- The texture uses nearest-neighbour sampling.
- The checkerboard is baked into the display buffer (see the architecture doc).
- Grid lines are drawn per pixel when the grid is enabled.
- The selection is shown as an animated dashed rectangle.
- Transform mode draws four corner handles.
- The brush cursor preview is a circle for round brushes and a rectangle for
  square brushes.

### Tool dispatch

| Tool | Event | Behaviour |
|------|-------|-----------|
| Brush / Eraser | Primary press and drag | Paint with the current colour; Eraser paints transparent |
| Fill | Primary click | Flood fill the active layer inside the selection |
| Eyedropper | Primary click | Pick the topmost non-transparent pixel of the active frame |
| Select | Primary drag | Create a marquee; dragging inside it moves the selected pixels |
| Move | Primary drag | Move the selection if the press starts inside it, otherwise move the active layer |
| Text | Primary click | Open the Add Text dialog at that pixel |
| Transform | Corner drag | Resize the selected rectangle; release commits |
| Effects | — | The tool only opens the effects panel; it does not edit the canvas |
| Any tool | Right click / right drag | Erase with the current brush |
| Any tool | Middle drag | Pan the canvas |

While a dialog is open, canvas tool dispatch and wheel zoom are suppressed.

## Dialogs

All dialogs are `egui::Area`s anchored to the screen centre, rendered in
`Order::Foreground`, with a `PANEL` fill and a 4 px `BORDER` outline. They are
modal in practice: `dialog_open()` blocks canvas interaction, wheel zoom and
most shortcuts.

### Welcome — 400 × 380

- Logo (56 px), title “Welcome to Pixesh” (28 pt) and subtitle (18 pt).
- 3 × 2 quick-action grid: New File, Open..., 16×16, 32×32, 64×64, 128×128.
- “Show this window on startup” checkbox.
- Close button; `Enter` and `Escape` also close it.
- “New File” creates a 16×16 document; Open uses the native file dialog;
  the numbered actions create a square document of that size.

### Resize Canvas — 250 × 250

- Title “Resize Canvas”.
- `W:` and `H:` `DragValue`s, range 1–4096.
- “Lock ratio” checkbox. Ratio is calculated from the original canvas size
  (`linked_side()`), preventing drift over repeated edits.
- `Enter` applies; Apply/Cancel buttons.
- Resizing keeps the top-left pixels and pads new area with transparency.

### Scale Canvas — 250 × 250

- Same layout and aspect-ratio behaviour as Resize Canvas.
- `Enter` applies; Apply/Cancel buttons.
- Scaling resamples the image with nearest-neighbour index mapping.

### Export PNG — 380 × 484

- Title “Export PNG”.
- Folder path, truncated to 14 characters, plus a `...` folder picker when
  `rfd` is enabled.
- File name `TextEdit`.
- Scale: x1, x2, x4, x8.
- Background: None, White, Black, Check (checkerboard).
- Frame mode: `Current` or `Sheet`, shown when the document has more than one
  frame.
- “Each layer” checkbox.
- `Out: WxH px` preview of the final size.
- Save/Cancel; `Enter` also saves.

Export behaviour:

- Single frame: exports one PNG; “Each layer” additionally writes one PNG per
  layer.
- Multiple frames, `Current`: writes numbered PNGs (`name_001.png`, ...).
- Multiple frames, `Sheet`: writes one horizontal sprite sheet.
- Scale uses nearest-neighbour upscaling.

### Panels — 250 × 320

- Title “Panels”.
- Toggle rows for Toolbar, Layers, Status Bar and Timeline.
- OK button; `Enter` and `Escape` close it.

### Settings — 300 × 360

- Title “Settings”.
- Arrow Speed slider, 1–100, step 5.
- Zoom Speed slider, 0.005–5.
- “Show Welcome at startup” checkbox.
- OK button; `Enter` and `Escape` close it.

### Add Text — 340 × 240

- Title “Add Text”.
- Single-line text input with the hint “Enter text...”.
- Scale `DragValue`, 1–8.
- Cursor preview `@(x,y)` when a canvas position was chosen.
- Draw/Cancel. Draw rasterises the text with the 5×7 bitmap font into the
  active layer.

### Effects — 520 × 620

- Title “Effects” with the note “ported from Pixelorama”.
- Category buttons: Transform, Color, Procedural, Blur.
- Scrollable effect list for the active category.
- Parameter controls: integer/float sliders, checkboxes, colour pickers and
  segmented choices.
- Live preview over a checkerboard, with canvas dimensions, active layer and
  frame index; shows “selection only” when a selection exists.
- “All layers” and “All frames” checkboxes.
- Reset / Apply / Close.

### Quit confirmation — 260 × 260

Defined inline in `src/app/state/update.rs`. Appears when the window is closed while at
least one document is unsaved: “Unsaved changes!” / “Quit anyway?” with
Quit and Cancel buttons.

## Keyboard and mouse shortcuts (`src/app/ui/input.rs`)

| Input | Action |
|-------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+S` | Open the export dialog |
| `Ctrl+L` | Open an image in a new tab |
| `Ctrl+R` | Resize canvas |
| `Ctrl+I` | Scale canvas |
| `Ctrl+W` | Panels dialog |
| `Ctrl+H` | Settings dialog |
| `Ctrl+Shift+E` | Toggle the effects panel |
| `Ctrl+Shift+F` | Flatten layers |
| `Ctrl+Tab` | Next tab |
| `Ctrl+D` | Deselect |
| `Ctrl+A` | Select the whole canvas |
| `Ctrl+C` / `Y` | Copy the selection |
| `Ctrl+V` / `P` | Paste as a floating selection |
| `B` `E` `F` | Brush, Eraser, Fill |
| `R` `M` `T` | Select, Move, Text |
| `X` | Toggle Transform (requires a selection) |
| `G` | Toggle the grid |
| `W` / `S` | Increase / decrease value (brightness) by 5 |
| `A` / `D` | Decrease / increase saturation by 5 |
| `I` / `K` | Rotate hue backward / forward by 5° |
| `,` / `.` | Decrease / increase opacity by 5 |
| `1`–`9` | Preset colours (red, green, blue, yellow, cyan, magenta, white, gray, black) |
| `[` / `]` | Previous / next frame |
| `Space` | Play / pause animation |
| `Scroll` | Zoom the canvas (no dialog open) |
| `Shift+Scroll` | Change brush size |
| Arrow keys | Pan the canvas; `Shift` makes it 4× faster |
| `Middle drag` | Pan the canvas |
| `Alt` (hold) | Temporarily switch to the eyedropper |
| `Right click` / `Right drag` | Erase |
| `Enter` | Commit a floating paste; otherwise crop to the selection |
| `Delete` | Delete the selected pixels |
| `Escape` | Cancel paste/transform/rename/text/dialog, otherwise deselect |

While a text field has focus, single-key tool shortcuts and clipboard
shortcuts are left to the field (`Ctrl+C`, `Ctrl+V`, `Ctrl+A`, etc.).
