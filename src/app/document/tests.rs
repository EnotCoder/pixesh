use eframe::egui::Color32;

use crate::app::document::{Document, MASCOT_SIDE};

#[test]
fn default_document_carries_the_mascot() {
    let doc = Document::new_logo("Untitled");
    assert_eq!(
        (doc.width, doc.height),
        (MASCOT_SIDE, MASCOT_SIDE),
        "стартовый холст подгонен под размер совы"
    );
    let px = &doc.layers[0].cels[0];
    assert_eq!(px.len(), MASCOT_SIDE * MASCOT_SIDE, "залит весь холст");
    assert!(px.iter().any(|c| c.a() > 0), "сова нарисована, а не пустой холст");
    assert!(!doc.unsaved, "совы по умолчанию — не правка пользователя");
}

#[test]
fn blank_document_stays_blank() {
    let doc = Document::new("Untitled");
    assert!(
        doc.layers[0].cels[0].iter().all(|c| c.a() == 0),
        "Document::new по-прежнему даёт пустой холст — на нём держатся другие тесты"
    );
}

#[test]
fn mascot_is_a_faithful_copy_of_the_source_png() {
    let doc = Document::new_logo("t");
    let src = image::load_from_memory(include_bytes!("../../../img/pixel_logo.png"))
        .expect("совы должна декодироваться")
        .to_rgba8();
    assert_eq!(
        src.dimensions(),
        (MASCOT_SIDE as u32, MASCOT_SIDE as u32),
        "источник обязан совпадать с холстом — иначе маскот начнёт ужиматься"
    );
    let px = &doc.layers[0].cels[0];
    for y in 0..MASCOT_SIDE {
        for x in 0..MASCOT_SIDE {
            let p = src.get_pixel(x as u32, y as u32);
            let want = Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]);
            assert_eq!(px[y * MASCOT_SIDE + x], want, "пиксель ({x},{y}) разошёлся");
        }
    }
}
