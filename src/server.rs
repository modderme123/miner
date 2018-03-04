use serde_json;
use client::{Message, Point};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::io::{BufRead, BufReader, Write};

pub enum Action {
    Add(SocketAddr, TcpStream),
    Remove(SocketAddr),
    Broadcast(Vec<u8>),
}

pub struct Server {
    connections: HashMap<SocketAddr, TcpStream>,
    rx: Receiver<Action>,
}

impl Server {
    pub fn new(listener: TcpListener) -> Server {
        Server {
            connections: HashMap::new(),
            rx: listen(listener),
        }
    }
    pub fn broadcast(&mut self, msg: &[u8]) {
        for connection in self.connections.values_mut() {
            connection.write(&[msg, &[0xa]].concat()).ok();
            connection.flush().ok();
        }
    }

    pub fn add_connection(&mut self, addr: &SocketAddr, stream: TcpStream) {
        self.connections.insert(*addr, stream);
        let msg = format!(
            "({} connections) ----- new connection from {} -----",
            self.connections.len(),
            addr
        );
        println!("{}", msg);
    }

    pub fn remove_connection(&mut self, addr: &SocketAddr) {
        self.connections.remove(addr);
        let msg = format!(
            "({} connections) ----- {} is disconnected -----",
            self.connections.len(),
            addr
        );
        println!("{}", msg);
        self.broadcast(&serde_json::to_vec(&Message::Remove(*addr)).unwrap())
    }

    pub fn handle(
        &mut self,
        players: &HashMap<SocketAddr, Point>,
        grains: &HashMap<SocketAddr, Vec<Point>>,
        terrain: &[[bool; 113]],
    ) {
        if let Ok(mut message) = self.rx.try_recv() {
            match message {
                Action::Add(addr, ref mut stream) => {
                    for (addr, p) in players {
                        stream
                            .write(&[
                                serde_json::to_vec(&Message::Move(*addr, *p)).unwrap(),
                                vec![0xa],
                            ].concat())
                            .ok();
                    }
                    for (addr, s) in grains {
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
                    self.add_connection(&addr, stream.try_clone().unwrap());
                }
                Action::Remove(addr) => self.remove_connection(&addr),
                Action::Broadcast(msg) => self.broadcast(&msg),
            }
        }
    }
}

fn handle_client(stream: TcpStream, addr: SocketAddr, sender: &Sender<Action>) {
    sender
        .send(Action::Add(addr, stream.try_clone().unwrap()))
        .ok();
    let mut r = BufReader::new(stream);
    'read: loop {
        let mut buf = String::new();
        if let Ok(n) = r.read_line(&mut buf) {
            if n == 0 {
                break 'read;
            }
            sender.send(Action::Broadcast(buf.into_bytes())).ok();
        }
    }
    sender.send(Action::Remove(addr)).ok();
}

fn listen(listener: TcpListener) -> Receiver<Action> {
    let (tx, rx): (Sender<Action>, Receiver<Action>) = mpsc::channel();
    thread::spawn(move || loop {
        if let Ok((stream, addr)) = listener.accept() {
            let thread_tx = tx.clone();
            thread::spawn(move || {
                handle_client(stream, addr, &thread_tx);
            });
        }
    });
    rx
}
