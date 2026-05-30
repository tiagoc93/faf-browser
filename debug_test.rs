// Debug test - render a simple red rectangle and save as PNG
// to verify tiny_skia Pixmap is working correctly
use tiny_skia::{Pixmap, Paint, Rect, Transform};

fn main() {
    // Create a 200x100 pixmap
    let mut pixmap = Pixmap::new(200, 100).unwrap();
    
    // Fill with white
    let mut white = Paint::default();
    white.set_color_rgba8(255, 255, 255, 255);
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, 200.0, 100.0).unwrap(),
        &white,
        Transform::identity(),
        None,
    );
    
    // Draw a red rectangle at (20, 20) size 80x40
    let mut red = Paint::default();
    red.set_color_rgba8(255, 0, 0, 255);
    pixmap.fill_rect(
        Rect::from_xywh(20.0, 20.0, 80.0, 40.0).unwrap(),
        &red,
        Transform::identity(),
        None,
    );
    
    // Draw a blue rectangle at (120, 30) size 60x50
    let mut blue = Paint::default();
    blue.set_color_rgba8(0, 0, 255, 255);
    pixmap.fill_rect(
        Rect::from_xywh(120.0, 30.0, 60.0, 50.0).unwrap(),
        &blue,
        Transform::identity(),
        None,
    );
    
    // Save
    pixmap.save_png("debug_test.png").unwrap();
    println!("Saved debug_test.png");
    
    // Verify by reading back pixels
    let pixels = pixmap.pixels();
    let w = pixmap.width();
    let h = pixmap.height();
    println!("Pixmap size: {}x{}", w, h);
    
    // Check center pixel of red rect (should be red)
    let cx = 60u32;
    let cy = 40u32;
    let idx = (cy * w + cx) as usize;
    let p = pixels[idx];
    println!("Pixel at ({},{}): RGBA=({},{},{},{})", cx, cy, p.red(), p.green(), p.blue(), p.alpha());
    
    // Check center pixel of blue rect (should be blue)
    let cx2 = 150u32;
    let cy2 = 55u32;
    let idx2 = (cy2 * w + cx2) as usize;
    let p2 = pixels[idx2];
    println!("Pixel at ({},{}): RGBA=({},{},{},{})", cx2, cy2, p2.red(), p2.green(), p2.blue(), p2.alpha());
    
    // Check white background pixel
    let cx3 = 10u32;
    let cy3 = 10u32;
    let idx3 = (cy3 * w + cx3) as usize;
    let p3 = pixels[idx3];
    println!("Pixel at ({},{}): RGBA=({},{},{},{})", cx3, cy3, p3.red(), p3.green(), p3.blue(), p3.alpha());
}