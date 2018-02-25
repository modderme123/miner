#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate piston_window;
extern crate rand;

use rand::random;
use piston_window::*;

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::io;

enum Action {
    Add(SocketAddr, TcpStream),
    Remove(SocketAddr),
    Broadcast(Vec<u8>),
}

struct Server {
    connections: HashMap<SocketAddr, TcpStream>,
}

impl Server {
    fn broadcast(&mut self, msg: &Vec<u8>) {
        for (_, mut connection) in self.connections.iter_mut() {
            connection.write(msg).ok();
            connection.flush().ok();
        }
    }

    fn add_connection(&mut self, addr: &SocketAddr, stream: TcpStream) {
        self.connections.insert(*addr, stream);
        let msg = format!(
            "({} connections) ----- new connection from {} -----",
            self.connections.len(),
            addr
        );
        println!("{}", msg);
    }

    fn remove_connection(&mut self, addr: &SocketAddr) {
        self.connections.remove(addr);
        let msg = format!(
            "({} connections) ----- {} is disconnected -----",
            self.connections.len(),
            addr
        );
        println!("{}", msg);
        self.broadcast(&serde_json::to_vec(&Message::Remove(*addr)).unwrap())
    }
}

fn handle_client(mut stream: TcpStream, addr: SocketAddr, sender: Sender<Action>) {
    'read: loop {
        let mut buf = [0; 4096];
        if let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break 'read;
            }
            sender.send(Action::Broadcast(buf[0..n].to_vec())).ok();
        }
    }
    sender.send(Action::Remove(addr)).ok();
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Point {
    pos: (f64, f64),
    vel: (f64, f64),
}

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    Move(SocketAddr, Point),
    Remove(SocketAddr),
}

const SCREEN: (u32, u32) = (720, 450);

fn main() {
    println!("try connecting via `telnet localhost 8080`");
    println!("Would you like to \n 1) Connect to an existing socket? \n 2) Create a new socket?");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let (listener, mut reader) = match &*input {
        "1\n" => {
            println!("What is the ip of the socket you would like to connect to?");
            input.clear();
            if let Ok(n) = io::stdin().read_line(&mut input) {
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
    let l = listener.is_some();
    let mut reader2 = reader.try_clone().unwrap();

    let (tx, rx): (Sender<Action>, Receiver<Action>) = mpsc::channel();
    thread::spawn(move || {
        for l in listener.iter() {
            loop {
                if let Ok((stream, addr)) = l.accept() {
                    {
                        tx.send(Action::Add(addr, stream.try_clone().unwrap())).ok();
                    }
                    let thread_tx = tx.clone();
                    thread::spawn(move || {
                        handle_client(stream, addr, thread_tx);
                    });
                }
            }
        }
    });

    let (reader_send, reader_read): (
        Sender<([u8; 4096], usize)>,
        Receiver<([u8; 4096], usize)>,
    ) = mpsc::channel();
    thread::spawn(move || 'read: loop {
        let mut buf = [0; 4096];
        if let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break 'read;
            }
            reader_send.send((buf, n)).ok();
        }
    });

    let mut connections = Server {
        connections: HashMap::new(),
    };
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
        if l {
            if let Ok(message) = rx.try_recv() {
                match message {
                    Action::Add(addr, mut stream) => {
                        for (addr, p) in players.iter() {
                            stream
                                .write(&serde_json::to_vec(&Message::Move(*addr, *p)).unwrap())
                                .ok();
                        }
                        connections.add_connection(&addr, stream);
                    }
                    Action::Remove(addr) => connections.remove_connection(&addr),
                    Action::Broadcast(msg) => connections.broadcast(&msg),
                }
            }
        }
        if let Ok(message) = reader_read.try_recv() {
            match serde_json::from_slice(&message.0[0..message.1]) {
                Ok(Message::Remove(addr)) => players.remove(&addr),
                Ok(Message::Move(addr, p)) if addr != local_addr => players.insert(addr, p),
                _ => None,
            };
        }
        window.draw_2d(&e, |c, g| {
            if clicking {
                if let Some(you) = players.get_mut(&local_addr) {
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
                    });
                    let a = &Message::Move(local_addr, you.clone());
                    reader2.write(&serde_json::to_vec(a).unwrap()).ok();
                    reader2.flush().ok();
                }
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

            for (_, you) in players.iter_mut() {
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
            }

            clear([0.95, 0.95, 0.95, 1.0], g);
            for (_, p) in players.iter() {
                rectangle(
                    [0.0, 0.0, 0.0, 1.0],
                    [p.pos.0 - 10.0, p.pos.1 - 10.0, 20.0, 20.0],
                    c.transform,
                    g,
                );
            }
        });
    }
}
