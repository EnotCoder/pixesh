# Lua configuration

Pixesh reads a plain Lua 5.4 file with three global tables: `colors`,
`constants` and `keys`. It is created from the bundled template on the first
launch at:

- Linux / macOS: `~/.config/pixesh/config.lua` (or
  `$XDG_CONFIG_HOME/pixesh/config.lua` when set)
- Windows: `%APPDATA%\pixesh\config.lua`

`pixesh --config /path/to/file.lua` overrides the path. `Ctrl+Shift+R`
(`reload_config` in the file) re-reads it at any time. A successful reload
updates colours, sizes and shortcuts immediately; a syntax error or invalid
value is shown in the status bar and the last good configuration stays active.
Every table and field is optional — omitted values keep their defaults. The
full annotated template is [config/config.lua](../config/config.lua).

## Key bindings

`keys` maps an action name to a key string or a list of strings. Modifiers are
`Ctrl`, `Shift`, `Alt` and `Cmd`; keys use egui's logical names (`A`, `Num1`,
`Space`, `Escape`, `Comma`, `OpenBracket`, `ArrowLeft`, …). `+` itself is
written literally: `"Ctrl++"`. Common layouts need both `"Ctrl+="` and
`"Ctrl+Shift+="` for the brush-size shortcut.

```lua
keys = {
    undo = "Ctrl+Z",
    redo = "Ctrl+Shift+Z",
    brush_up = { "Ctrl+=", "Ctrl+Shift+=", "Ctrl++", "Ctrl+Shift++" },
    copy = { "Y" },
}
```

Bindings are checked most-specific-first (`Ctrl+Shift+L` before `Ctrl+L`) and
the first match consumes the key. Set an action to `false` to remove an
inherited binding (`undo = false`). The action's scope is fixed in the app and
cannot be changed from Lua:

| Scope | When the shortcut fires |
|-------|-------------------------|
| `Global` | Always, including dialogs |
| `Clean` | No dialog and no layer rename |
| `DialogFree` | No dialog |
| `Bare` | No Ctrl/Alt/Cmd and no focused text field |
| `BareClean` | Both `Bare` and `Clean` |

### Action names

| Lua name | Default | Scope | Action |
|----------|---------|-------|--------|
| `undo` | `Ctrl+Z` | Global | Undo |
| `redo` | `Ctrl+Shift+Z` | Global | Redo |
| `brush_up` | `Ctrl+=`, `Ctrl+Shift+=`, `Ctrl++`, `Ctrl+Shift++` | Clean | Bigger brush |
| `brush_down` | `Ctrl+-` | Clean | Smaller brush |
| `add_layer` | `Ctrl+L` | Clean | Add layer |
| `remove_layer` | `Ctrl+Shift+L` | Clean | Remove layer |
| `export` | `Ctrl+S` | DialogFree | Export PNG dialog |
| `resize` | `Ctrl+R` | DialogFree | Resize dialog |
| `panels` | `Ctrl+W` | DialogFree | Panels dialog |
| `settings` | `Ctrl+H` | DialogFree | Settings dialog |
| `scale` | `Ctrl+I` | DialogFree | Scale dialog |
| `effects` | `Ctrl+Shift+E` | Global | Effects panel |
| `new_tab` | `Ctrl+N` | Clean | New blank tab |
| `color_input` | `Ctrl+K` | Clean | Live RGB/hex entry |
| `open_file` | `Ctrl+O` | DialogFree | Open image dialog |
| `next_tab` | `Ctrl+Tab` | Global | Next tab |
| `tool_brush` | `B` | Bare | Brush tool |
| `tool_eraser` | `E` | Bare | Eraser tool |
| `tool_fill` | `F` | Bare | Fill tool |
| `tool_select` | `R` | Bare | Select tool |
| `tool_move` | `M` | Bare | Move tool |
| `tool_text` | `T` | Bare | Text tool |
| `toggle_grid` | `G` | Bare | Grid overlay |
| `hsv_value_up` | `W` | Bare | +value |
| `hsv_value_down` | `S` | Bare | −value |
| `hsv_sat_down` | `A` | Bare | −saturation |
| `hsv_sat_up` | `D` | Bare | +saturation |
| `hsv_hue_down` | `I` | Bare | −hue |
| `hsv_hue_up` | `K` | Bare | +hue |
| `alpha_down` | `,` | Bare | −opacity |
| `alpha_up` | `.` | Bare | +opacity |
| `palette_1` … `palette_9` | `1` … `9` | Bare | Preset colours |
| `frame_prev` | `[` | Bare | Previous frame |
| `frame_next` | `]` | Bare | Next frame |
| `toggle_play` | `Space` | Bare | Play / pause |
| `delete` | `Delete` | Global | Delete selection |
| `commit_or_crop` | `Enter` | Clean | Commit paste / crop to selection |
| `escape` | `Escape` | Global | Cancel in priority order |
| `deselect` | `Ctrl+Shift+D` | Clean | Deselect |
| `close_tab` | `Ctrl+D` | Clean | Close active tab |
| `select_all` | `Ctrl+A` | Clean | Select whole canvas |
| `flatten` | `Ctrl+Shift+F` | DialogFree | Flatten layers |
| `copy` | `Y` | Global | Copy selection (also while typing) |
| `copy_clipboard` | `Ctrl+C` | Clean | Copy selection, respects text fields |
| `paste` | `P` | Global | Paste floating selection |
| `paste_clipboard` | `Ctrl+V` | Clean | Paste floating selection |
| `transform` | `Shift+X` | BareClean | Toggle transform |
| `clear_all` | `X` | BareClean | Clear all layers (confirmation) |
| `reload_config` | `Ctrl+Shift+R` | Clean | Reload this file |

Dialog-local keys (arrows/Enter/Esc inside dialogs, text field editing,
`Alt` eyedropper, wheel zoom and arrow panning) are not part of `keys`.

## Colours

Values are `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`, or `{r, g, b[, a]}` with
0–255 components. `colors.palette` is the list used by `palette_1`…
`palette_9`; a shorter list keeps the remaining built-in colours.

| Field | Default | Meaning |
|-------|---------|---------|
| `bg` | `#181818` | Canvas background |
| `panel` | `#202020` | Panels and dialogs |
| `panel_light` | `#2c2c2c` | Raised controls |
| `border` | `#505050` | Widget borders |
| `text` | `#dcdcdc` | Primary text |
| `accent` | `#c8783c` | Selection / active state |
| `hover` | `#3c3c3c` | Hover state |
| `dim` | `#787878` | Secondary text |
| `palette` | red, green, blue, yellow, cyan, magenta, white, gray, black | Number-key colours |

Colours apply both to egui's theme and to the custom-painted pixel widgets.

## Constants

| Field | Default | Valid range | Meaning |
|-------|---------|-------------|---------|
| `font_size` | `20` | 1–200 | Body font size in pixels |
| `char_width` | `11` | 0.1–100 | Approximate character width |
| `row_height` | `22` | 1–200 | Base row height |
| `panel_padding` | `12` | 0–200 | Padding inside panels |
| `checker_px` | `8` | 1–128 | Transparency checker square |
| `max_side` | `4096` | 16–16384 | Maximum canvas side |
| `hsv_step` | `5` | 0.1–180 | Colour key step |
| `alpha_step` | `5` | 0.1–255 | Opacity key step |
| `brush_step` | `1` | 0.1–512 | Brush size key/wheel step |
| `zoom_min` | `0.1` | 0.01–10 | Minimum zoom |
| `zoom_max` | `60` | 1–500 | Maximum zoom |
| `zoom_step` | `0.1` | 0.001–10 | Wheel zoom factor multiplier |
| `pan_shift_multiplier` | `4` | 1–20 | Shift+arrow pan speed |

`zoom_min` must be smaller than `zoom_max`. Values outside the stated ranges
are rejected with an error rather than silently clamped, so typos are visible.
Arrow-pan speed and zoom speed themselves remain in the Settings dialog
(`app.ron`) because that dialog edits them at runtime.
