-- Pixesh configuration (Lua 5.4).
--
-- On the first launch this file is copied to
--   ~/.config/pixesh/config.lua   (or $XDG_CONFIG_HOME/pixesh/config.lua)
-- Use `pixesh --config /path/to/file.lua` to load another one.
-- Press Ctrl+Shift+R in the app to reload it without restarting.
--
-- Every table and every field is optional: absent values keep the built-in
-- defaults. A syntax error leaves the last good configuration in place and
-- shows the error in the status bar.

-- Theme colours: #RGB, #RGBA, #RRGGBB or #RRGGBBAA.
-- Arrays {r, g, b[, a]} with 0..255 components are accepted as well.
colors = {
    bg          = "#181818", -- canvas background
    panel       = "#202020", -- panels and dialogs
    panel_light = "#2c2c2c", -- raised controls
    border      = "#505050", -- widget borders
    text        = "#dcdcdc", -- primary text
    accent      = "#c8783c", -- selection / active state
    hover       = "#3c3c3c", -- hover state
    dim         = "#787878", -- secondary text

    -- Colours selected with the number keys 1..9.
    palette = {
        "#ff0000ff", -- 1 red
        "#00ff00ff", -- 2 green
        "#0000ffff", -- 3 blue
        "#ffff00ff", -- 4 yellow
        "#00ffffff", -- 5 cyan
        "#ff00ffff", -- 6 magenta
        "#ffffffff", -- 7 white
        "#808080ff", -- 8 gray
        "#000000ff", -- 9 black
    },
}

-- Layout sizes, tool steps and navigation limits.
constants = {
    font_size            = 20.0,   -- body font size in pixels
    char_width           = 11.0,   -- approximate character width used for layout
    row_height           = 22.0,   -- base row height
    panel_padding        = 12.0,   -- padding inside panels
    checker_px           = 8.0,    -- transparent-background checker square
    max_side             = 4096,   -- maximum canvas side in pixels
    hsv_step             = 5.0,    -- W/S/A/D/I/K colour step
    alpha_step           = 5.0,    -- ,/. opacity step
    brush_step           = 1.0,    -- Ctrl++/Ctrl+- brush step
    zoom_min             = 0.1,
    zoom_max             = 60.0,
    zoom_step            = 0.1,    -- wheel zoom factor multiplier
    pan_shift_multiplier = 4.0,    -- Shift+arrow speed multiplier
}

-- Action names are fixed; values are key combinations. A list assigns
-- several combinations to one action. Modifiers: Ctrl, Shift, Alt, Cmd.
-- Keys use egui names (A, Num1, Space, Escape, Comma, OpenBracket, ...).
keys = {
    undo            = "Ctrl+Z",
    redo            = "Ctrl+Shift+Z",

    brush_up        = { "Ctrl+=", "Ctrl+Shift+=", "Ctrl++", "Ctrl+Shift++" },
    brush_down      = "Ctrl+-",
    add_layer       = "Ctrl+L",
    remove_layer    = "Ctrl+Shift+L",

    export          = "Ctrl+S",
    resize          = "Ctrl+R",
    panels          = "Ctrl+W",
    settings        = "Ctrl+H",
    scale           = "Ctrl+I",
    effects         = "Ctrl+Shift+E",
    new_tab         = "Ctrl+N",
    color_input     = "Ctrl+K",
    open_file       = "Ctrl+O",
    next_tab        = "Ctrl+Tab",

    tool_brush      = "B",
    tool_eraser     = "E",
    tool_fill       = "F",
    tool_select     = "R",
    tool_move       = "M",
    tool_text       = "T",
    toggle_grid     = "G",

    hsv_value_up    = "W",
    hsv_value_down  = "S",
    hsv_sat_down    = "A",
    hsv_sat_up      = "D",
    hsv_hue_down    = "I",
    hsv_hue_up      = "K",
    alpha_down      = ",",
    alpha_up        = ".",

    palette_1       = "1",
    palette_2       = "2",
    palette_3       = "3",
    palette_4       = "4",
    palette_5       = "5",
    palette_6       = "6",
    palette_7       = "7",
    palette_8       = "8",
    palette_9       = "9",

    frame_prev      = "[",
    frame_next      = "]",
    toggle_play     = "Space",
    delete          = "Delete",
    commit_or_crop  = "Enter",
    escape          = "Escape",

    deselect        = "Ctrl+Shift+D",
    close_tab       = "Ctrl+D",
    select_all      = "Ctrl+A",
    flatten         = "Ctrl+Shift+F",

    copy            = "Y",          -- bare key, works while typing
    copy_clipboard  = "Ctrl+C",     -- respects text fields
    paste           = "P",
    paste_clipboard = "Ctrl+V",
    transform       = "Shift+X",
    clear_all       = "X",
    reload_config   = "Ctrl+Shift+R",
}
