extern crate piston_window;
extern crate rand;

use piston_window::*;
use rand::random;

const SCREEN: (u32, u32) = (720, 450);
struct Point {
    pos: (f64, f64),
    vel: (f64, f64),
}
fn main() {
    let mut window: PistonWindow = WindowSettings::new("Miner!", SCREEN)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut you = Point {
        pos: (50.0, 50.0),
        vel: (0.0, 0.0),
    };
    let mut clicking = false;
    let mut cursor = (0.0, 0.0);
    let mut spray = vec![];
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
                let x = (you.pos.0 - 10.0 - cursor.0, you.pos.1 - 10.0 - cursor.1);
                let l = (x.0 * x.0 + x.1 * x.1).sqrt();
                you.vel.0 += 0.1 * x.0 / l;
                you.vel.1 += 0.1 * x.1 / l;

                spray.push(Point {
                    pos: (you.pos.0 - x.0 / l, you.pos.1 - x.1 / l),
                    vel: (
                        -10.0 * x.0 / l + 3.0 * random::<f64>(),
                        -10.0 * x.1 / l + 3.0 * random::<f64>(),
                    ),
                })
            }
            for grain in spray.iter_mut() {
                rectangle(
                    [0.0, 0.0, 0.0, 1.0],
                    [grain.pos.0 - 3.0, grain.pos.1 - 3.0, 6.0, 6.0],
                    c.transform,
                    g,
                );
                grain.pos.0 += grain.vel.0;
                grain.pos.1 += grain.vel.1;
                grain.vel.0 *= 0.99;
                grain.vel.1 *= 0.99;
                grain.vel.1 += 0.05;
            }
            spray.retain(|grain| grain.pos.1 < SCREEN.1 as f64 && grain.pos.1 > 0.0);
            you.vel.1 += 0.05;
            you.vel.0 *= 0.99;
            you.vel.1 *= 0.99;
            if you.pos.0 >= SCREEN.0 as f64 && you.vel.0 >= 0.0 {
                you.vel.0 = 0.0
            }
            if you.pos.0 <= 0.0 && you.vel.0 <= 0.0 {
                you.vel.0 = 0.0
            }
            if you.pos.1 >= SCREEN.1 as f64 && you.vel.1 >= 0.0 {
                you.vel.1 = 0.0
            }
            if you.pos.1 <= 0.0 && you.vel.1 <= 0.0 {
                you.vel.1 = 0.0
            }
            you.pos.0 = (you.pos.0 + you.vel.0).max(0.0).min(SCREEN.0 as f64);
            you.pos.1 = (you.pos.1 + you.vel.1).max(0.0).min(SCREEN.1 as f64);

            clear([0.95, 0.95, 0.95, 1.0], g);
            rectangle(
                [0.0, 0.0, 0.0, 1.0],
                [you.pos.0 - 10.0, you.pos.1 - 10.0, 20.0, 20.0],
                c.transform,
                g,
            );
        });
    }
}
