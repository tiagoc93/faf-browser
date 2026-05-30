use tiny_skia::{Paint, Pixmap, Rect, Transform};

fn main() {
    let mut pixmap = Pixmap::new(400, 300).expect("create pixmap");

    let mut white = Paint::default();
    white.set_color_rgba8(255, 255, 255, 255);
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, 400.0, 300.0).unwrap(),
        Transform::identity(),
        &white,
        None,
    );

    let mut red = Paint::default();
    red.set_color_rgba8(255, 0, 0, 255);
    pixmap.fill_rect(
        Rect::from_xywh(50.0, 50.0, 100.0, 80.0).unwrap(),
        Transform::identity(),
        &red,
        None,
    );

    let mut green = Paint::default();
    green.set_color_rgba8(0, 255, 0, 255);
    for i in 0..20 {
        for j in 0..20 {
            pixmap.fill_rect(
                Rect::from_xywh(200.0 + i as f32 * 2.5, 100.0 + j as f32 * 2.5, 2.5, 2.5).unwrap(),
                Transform::identity(),
                &green,
                None,
            );
        }
    }

    pixmap.save_png("debug_render.png").expect("save");
    println!("Saved debug_render.png");

    let pixels = pixmap.pixels();
    let w = pixmap.width();

    let idx_red = (60 * w + 80) as usize;
    let p = pixels[idx_red];
    println!(
        "Pixel (80,60) [red area]: ({},{},{},{})",
        p.red(),
        p.green(),
        p.blue(),
        p.alpha()
    );

    let idx_green = (110 * w + 220) as usize;
    let p2 = pixels[idx_green];
    println!(
        "Pixel (220,110) [green area]: ({},{},{},{})",
        p2.red(),
        p2.green(),
        p2.blue(),
        p2.alpha()
    );

    let idx_white = (10 * w + 10) as usize;
    let p3 = pixels[idx_white];
    println!(
        "Pixel (10,10) [white bg]: ({},{},{},{})",
        p3.red(),
        p3.green(),
        p3.blue(),
        p3.alpha()
    );
}