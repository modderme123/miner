extern crate piston_window;
extern crate rand;

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
        window.draw_2d(&e, |c, g| {
            if clicking {
                let x = (you.0 - cursor.0, you.1 - cursor.1);
                let l = (x.0 * x.0 + x.1 * x.1).sqrt();
                you.0 += 5.0 * x.0 / l;
                you.1 += 5.0 * x.1 / l;
            }
            clear([0.95, 0.95, 0.95, 1.0], g);
            rectangle(
                [0.0, 0.0, 0.0, 1.0],
                [you.0 - 25.0, you.1 - 25.0, 50.0, 50.0],
                c.transform,
                g,
            );
        });
    }
}
