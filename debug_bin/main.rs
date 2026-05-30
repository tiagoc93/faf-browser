use tiny_skia::{Pixmap, Paint, Rect, Transform};

fn main() {
    let mut pixmap = Pixmap::new(200, 100).unwrap();
    
    let mut white = Paint::default();
    white.set_color_rgba8(255, 255, 255, 255);
    pixmap.fill_rect(Rect::from_xywh(0.0, 0.0, 200.0, 100.0).unwrap(), &white, Transform::identity(), None);
    
    let mut red = Paint::default();
    red.set_color_rgba8(255, 0, 0, 255);
    pixmap.fill_rect(Rect::from_xywh(20.0, 20.0, 80.0, 40.0).unwrap(), &red, Transform::identity(), None);
    
    pixmap.save_png("debug_bin/test.png").unwrap();
    println!("Saved");
    
    let pixels = pixmap.pixels();
    let w = pixmap.width();
    let idx = (40 * w + 60) as usize;
    let p = pixels[idx];
    println!("Red pixel at (60,40): ({},{},{},{})", p.red(), p.green(), p.blue(), p.alpha());
}
