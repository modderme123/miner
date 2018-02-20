extern crate piston_window;
extern crate rand;

use rand::random;
use piston_window::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", (640, 480))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
    let mut you = (50.0, 50.0);
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([0.5, 1.0, 0.5, 1.0], g);
            rectangle([1.0, 1.0, 0.5, 1.0],[you.0,you.1,50.0,50.0],c.transform, g);
        });
    }
    println!("{} Hello world, I LOST THE GAME!", random::<i64>());
}
