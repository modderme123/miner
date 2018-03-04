#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate piston_window;
extern crate rand;

mod server;
mod client;

use server::{Action, Server};
use client::*;

use rand::random;
use piston_window::*;

use std::net::{TcpListener, TcpStream};
use std::io::Write;
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::io::{stdin, BufRead, BufReader};

const SCREEN: (u32, u32) = (1080, 678);

fn main() {
    println!("Would you like to \n 1) Connect to an existing socket? \n 2) Create a new socket?");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let (listener, reader) = match &*input {
        "1\n" => {
            println!("What is the ip of the socket you would like to connect to?");
            input.clear();
            if let Ok(n) = stdin().read_line(&mut input) {
                input.remove(n - 1);
            }
            (None, TcpStream::connect(input + ":8080").unwrap())
        }
        "2\n" => (
            Some(TcpListener::bind("0.0.0.0:8080").unwrap()),
            TcpStream::connect("0.0.0.0:8080").unwrap(),
        ),
        _ => (None, TcpStream::connect("0.0.0.0:8080").unwrap()),
    };
    let mut reader2 = reader.try_clone().unwrap();

    let (reader_send, reader_read): (Sender<String>, Receiver<String>) = mpsc::channel();
    thread::spawn(move || {
        for buf in BufReader::new(reader).lines() {
            if let Ok(b) = buf {
                if b == "" {
                    continue;
                }
                reader_send.send(b).ok();
            }
        }
    });

    let mut connections = listener.map(|l| Server::new(l));
    let rx = connections.as_mut().map(|c| c.listen());

    let mut window: PistonWindow = WindowSettings::new("Miner!", SCREEN)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let local_addr = reader2.local_addr().unwrap();

    let mut players = HashMap::new();
    players.insert(
        local_addr,
        Point {
            pos: (50.0, 50.0),
            vel: (0.0, 0.0),
        },
    );

    let mut grains = HashMap::new();
    grains.insert(local_addr, vec![]);

    let mut clicking = false;
    let mut cursor = (0.0, 0.0);
    let mut terrain = vec![[false; SCREEN.1 as usize / 6]; SCREEN.0 as usize / 6];
    {
        let mut rects = vec![];

        for _ in 1..100 {
            let x = (
                random::<f64>() * SCREEN.0 as f64 / 6.0,
                random::<f64>() * SCREEN.1 as f64 / 6.0,
            );
            rects.push((
                x.0,
                x.1,
                x.0 + random::<f64>() * 20.0,
                x.1 + random::<f64>() * 20.0,
            ))
        }
        for (x, a) in (0..).zip(terrain.iter_mut()) {
            for (y, val) in (0..).zip(a.iter_mut()) {
                *val = rects.iter().any(|r| {
                    (x as f64) > r.0 && (x as f64) < r.2 && (y as f64) > r.1 && (y as f64) < r.3
                });
            }
        }
    }
    let mut time = 0;
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| cursor = (x, y));
        match e.press_args() {
            Some(Button::Mouse(_)) | Some(Button::Keyboard(Key::Space)) => clicking = true,
            _ => (),
        }
        match e.release_args() {
            Some(Button::Mouse(_)) | Some(Button::Keyboard(Key::Space)) => clicking = false,
            _ => (),
        }
        if let Some(Ok(mut message)) = rx.as_ref().map(|r| r.try_recv()) {
            if let &mut Some(ref mut c) = &mut connections {
                match message {
                    Action::Add(addr, ref mut stream) => {
                        for (addr, p) in &players {
                            stream
                                .write(&[
                                    serde_json::to_vec(&Message::Move(*addr, *p)).unwrap(),
                                    vec![0xa],
                                ].concat())
                                .ok();
                        }
                        for (addr, s) in &grains {
                            for x in s {
                                stream
                                    .write(&[
                                        serde_json::to_vec(&Message::Add(*addr, *x)).unwrap(),
                                        vec![0xa],
                                    ].concat())
                                    .ok();
                            }
                        }
                        stream
                            .write(&[
                                serde_json::to_vec(&Message::Terrain(
                                    terrain.iter().map(|x| x.to_vec()).collect(),
                                )).unwrap(),
                                vec![0xa],
                            ].concat())
                            .ok();
                        stream.flush().ok();
                        c.add_connection(&addr, stream.try_clone().unwrap());
                    }
                    Action::Remove(addr) => c.remove_connection(&addr),
                    Action::Broadcast(msg) => c.broadcast(&msg),
                }
            }
        }
        if let Ok(message) = reader_read.try_recv() {
            match serde_json::from_str(&message) {
                Ok(Message::Remove(addr)) => {
                    players.remove(&addr);
                }
                Ok(Message::Move(addr, p)) if addr != local_addr => {
                    players.insert(addr, p);
                }
                Ok(Message::Add(addr, ref g)) if addr != local_addr => {
                    grains.entry(addr).or_insert_with(Vec::new).push(*g);
                }
                Ok(Message::Terrain(t)) => {
                    terrain = t.iter()
                        .map(|x| {
                            let mut y = [false; SCREEN.1 as usize / 6];
                            y.clone_from_slice(x);
                            y
                        })
                        .collect()
                }
                _ => (),
            };
        }
        if time % 10 == 0 {
            if let Some(you) = players.get(&local_addr) {
                reader2
                    .write(&[
                        serde_json::to_vec(&Message::Move(local_addr, *you)).unwrap(),
                        vec![0xa],
                    ].concat())
                    .ok();
                reader2.flush().ok();
            }
        }
        time += 1;
        window.draw_2d(&e, |c, g| {
            if clicking {
                if let Some(you) = players.get_mut(&local_addr) {
                    let x = (SCREEN.0 as f64 / 2.0 - cursor.0, you.pos.1 - cursor.1);
                    let l = (x.0 * x.0 + x.1 * x.1).sqrt();

                    you.vel.0 += 0.5 * x.0 / l;
                    you.vel.1 += 0.5 * x.1 / l;
                    if let Some(mut spray) = grains.get_mut(&local_addr) {
                        for _ in 1..5 {
                            let p = Point {
                                pos: (you.pos.0 - x.0 / l, you.pos.1 - x.1 / l),
                                vel: (
                                    -10.0 * x.0 / l + 3.0 * random::<f64>(),
                                    -10.0 * x.1 / l + 3.0 * random::<f64>(),
                                ),
                            };
                            spray.push(p);
                            reader2
                                .write(&[
                                    serde_json::to_vec(&Message::Add(local_addr, p)).unwrap(),
                                    vec![0xa],
                                ].concat())
                                .ok();
                            reader2.flush().ok();
                        }
                    }
                }
            }

            for spray in grains.values_mut() {
                for grain in spray.iter_mut() {
                    grain.pos.0 += grain.vel.0;
                    grain.pos.1 += grain.vel.1;
                    grain.vel.0 *= 0.99;
                    grain.vel.1 *= 0.99;
                    grain.vel.1 += 0.05;
                }
                spray.retain(|grain| {
                    grain.pos.0 < terrain.len() as f64 * 6.0 && grain.pos.0 > 0.0
                        && grain.pos.1 < SCREEN.1 as f64 && grain.pos.1 > 0.0
                });
                spray.retain(|grain| {
                    let x = terrain[grain.pos.0 as usize / 6][grain.pos.1 as usize / 6];
                    terrain[grain.pos.0 as usize / 6][grain.pos.1 as usize / 6] = false;
                    !x
                });
            }
            for you in players.values_mut() {
                you.vel.1 += 0.06;
                you.vel.0 *= 0.95;
                you.vel.1 *= 0.95;
                if you.pos.0 >= SCREEN.0 as f64 && you.vel.0 >= 0.0
                    && terrain.len() as f64 <= you.pos.0 / 6.0
                {
                    if let &mut Some(ref mut c) = &mut connections {
                        let mut rects = vec![];

                        for _ in 1..100 {
                            let x = (
                                random::<f64>() * SCREEN.0 as f64 / 6.0 + terrain.len() as f64,
                                random::<f64>() * SCREEN.1 as f64 / 6.0,
                            );
                            rects.push((
                                x.0,
                                x.1,
                                x.0 + random::<f64>() * 20.0,
                                x.1 + random::<f64>() * 20.0,
                            ))
                        }
                        for _ in 0..SCREEN.0 as usize / 6 {
                            terrain.push([false; SCREEN.1 as usize / 6]);
                        }
                        for (x, a) in (0..).zip(terrain.iter_mut()) {
                            for (y, val) in (0..).zip(a.iter_mut()) {
                                *val |= rects.iter().any(|r| {
                                    (x as f64) > r.0 && (x as f64) < r.2 && (y as f64) > r.1
                                        && (y as f64) < r.3
                                });
                            }
                        }
                        c.broadcast(&[
                            serde_json::to_vec(&Message::Terrain(
                                terrain.iter().map(|x| x.to_vec()).collect(),
                            )).unwrap(),
                            vec![0xa],
                        ].concat());
                    }
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

                you.pos.0 = (you.pos.0 + you.vel.0)
                    .max(0.0)
                    .min(terrain.len() as f64 * 6.0);
                you.pos.1 = (you.pos.1 + you.vel.1).max(0.0).min(SCREEN.1 as f64);
            }

            clear([0.95, 0.95, 0.95, 1.0], g);

            let yp = players[&local_addr].pos.0 - SCREEN.0 as f64 / 2.0;

            for (x, a) in (0..).zip(terrain.iter()) {
                for (y, val) in (0..).zip(a.iter()) {
                    let a = (x * 6) as f64 - yp;
                    if a > 0.0 && a < SCREEN.0 as f64 && *val {
                        rectangle(
                            [0.0, 0.0, 0.0, 1.0],
                            [x as f64 * 6.0 - yp, y as f64 * 6.0, 6.0, 6.0],
                            c.transform,
                            g,
                        );
                    }
                }
            }
            for (addr, spray) in &grains {
                for grain in spray.iter() {
                    rectangle(
                        if addr == &local_addr {
                            [0.0, 0.0, 0.0, 1.0]
                        } else {
                            [0.7, 0.7, 0.7, 1.0]
                        },
                        [grain.pos.0 - yp, grain.pos.1, 6.0, 6.0],
                        c.transform,
                        g,
                    );
                }
            }
            for (addr, p) in &players {
                rectangle(
                    if addr == &local_addr {
                        [0.0, 0.0, 0.0, 1.0]
                    } else {
                        [0.7, 0.7, 0.7, 1.0]
                    },
                    [p.pos.0 - 10.0 - yp, p.pos.1 - 10.0, 20.0, 20.0],
                    c.transform,
                    g,
                );
            }
        });
    }
}
