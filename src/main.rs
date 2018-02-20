extern crate piston_window;
extern crate rand;

use rand::random;
use piston_window::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Miner!", (640, 480))
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut you = (50.0, 50.0);
    let mut clicking = false;
    let mut cursor = (0.0, 0.0);
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| cursor = (x, y));
        if let Some(Button::Mouse(_)) = e.press_args() {
            clicking = true
        }
        if let Some(Button::Mouse(_)) = e.release_args() {
            clicking = false
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Up => you.1 -= 10.0,
                Key::Down => you.1 += 10.0,
                Key::Left => you.0 -= 10.0,
                Key::Right => you.0 += 10.0,
                _ => (),
            };
        }
        window.draw_2d(&e, |c, g| {
            clear([0.5, 1.0, 0.5, 1.0], g);
            rectangle(
                [if clicking { 1.0 } else { 0.0 }, 1.0, 0.5, 1.0],
                [you.0, you.1, 50.0, 50.0],
                c.transform,
                g,
            );
        });
    }
}
