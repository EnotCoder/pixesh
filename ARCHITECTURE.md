# Pixesh — архитектура проекта

## Стек

- **Язык:** Rust
- **GUI:** `eframe`/`egui` (0.31)
- **Изображения:** `image` crate
- **Диалоги:** `rfd` (выбор папки для экспорта)
- **Рендер:** wgpu (через egui)

## Структура файлов

```
pixesh/
├── src/
│   ├── main.rs              — точка входа, настройка шрифта/темы, запуск окна
│   ├── constants.rs         — цвета PANEL/BG/TEXT/ACCENT/HOVER/BORDER,
│   │                         размеры FONT_SZ/CHAR_W/ROW_H,
│   │                         enum Tool { Brush, Eraser, Fill, Eyedropper }
│   ├── color.rs             — hsv_to_rgb() / rgb_to_hsv()
│   ├── ui.rs                — виджеты btn/icon_btn/checkbox/slider/separator
│   └── app/
│       ├── mod.rs           — Layer, Snapshot, PixeshApp (все поля + new()),
│       │                     impl eframe::App (update() диспатчит)
│       ├── canvas.rs        — brush_i, pixels_mut, composite, composite_display,
│       │                     paint_pixel, paint_line, flood_fill, screen_to_pixel
│       ├── history.rs       — push_undo, undo, redo
│       ├── io.rs            — add_layer, remove_layer, save_png, load_png,
│       │                     resize_canvas
│       ├── input.rs         — handle_input (горячие клавиши, колёсико, стрелки)
│       ├── panel_toolbar.rs — верхняя панель с инструментами
│       ├── panel_layers.rs  — правая панель: слои + HSV-пикер
│       ├── panel_canvas.rs  — центральный холст: отрисовка, рисование
│       └── panel_dialogs.rs — диалоги Resize и Export PNG
└── tex/                     — иконки инструментов (PNG)
```

## Поток выполнения

```
main()
  └→ PixeshApp::new()
       └→ eframe::App::update(ctx, frame)  ← каждый кадр
            ├→ self.handle_input(ctx)         — горячие клавиши
            ├→ self.ui_toolbar(ctx)           — TopBottomPanel
            ├→ self.ui_layers(ctx)            — SidePanel (справа)
            ├→ self.ui_canvas(ctx)            — CentralPanel
            └→ self.ui_dialogs(ctx)           — всплывающие окна
```

## Ключевые структуры

### `Layer`
- `name: String`
- `pixels: Arc<Vec<Color32>>` — пиксели слоя (RGBA)
- `visible: bool`

### `Snapshot` (undo/redo)
- `layers: Vec<Arc<Vec<Color32>>>` — копии Arc (refcount), без клонирования Vec
- `active: usize`

### `PixeshApp` (основные поля)
- `layers`, `active_layer`, `width`, `height` — холст
- `color`, `hsv_h/s/v`, `rgb_r/g/b` — текущий цвет (дублирование для HSV UI)
- `brush`, `tool`, `tool_saved` — кисть и инструмент
- `last_px_primary/secondary` — трекинг драга (для paint_line)
- `grid`, `zoom`, `pan` — отображение
- `tex`, `*_tex`, `sv_tex` — кэш текстур egui
- `canvas_dirty: bool` — флаг перекомпозита
- `undo_stack`, `redo_stack` — история
- `show_resize`, `resize_w/h`, `show_export`, `export_name/path` — диалоги

## Оптимизации

- **Arc для пикселей:** `Layer.pixels = Arc<Vec<Color32>>`. push_undo копирует только Arc
  (инкремент счетчика). `paint_pixel`/`flood_fill` используют `Arc::make_mut()`
  (copy-on-write — реальное клонирование только если refcount > 1).
- **canvas_dirty:** текстура холста пересобирается и загружается на GPU только
  при изменении пикселей. На пустом кадре (панорамирование) — ноль аллокаций.
- **composite_display:** шахматка впекается прямо в массив пикселей за один проход,
  без отдельного цикла отрисовки rect_filled.
- **paint_line:** Брезенхем между предыдущей и текущей позицией курсора,
  чтобы не было пропусков при быстром движении мыши.
- **last_px сброс:** при выходе за пределы холста во время драга `last_px` очищается,
  чтобы не рисовать линию-соединитель при повторном входе.

## Замечания

- undo снэпшот сохраняется ТОЛЬКО при начале штриха (`push_undo`), а не на каждый
  кадр драга. Это экономит память, но при отмене откатывается весь штрих.
- Размер undo стека ограничен 50 снапшотами.
- `composite()` (без шахматки) используется только для пипетки и экспорта PNG.
