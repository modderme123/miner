extern crate serde_json;

use client::Message;
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
    listener: TcpListener,
}

impl Server {
    pub fn new(listener: TcpListener) -> Server {
        Server {
            connections: HashMap::new(),
            listener,
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
    pub fn listen(&mut self) -> Receiver<Action> {
        let (tx, rx): (Sender<Action>, Receiver<Action>) = mpsc::channel();
        let l = self.listener.try_clone().unwrap();
        thread::spawn(move || loop {
            if let Ok((stream, addr)) = l.accept() {
                let thread_tx = tx.clone();
                thread::spawn(move || {
                    handle_client(stream, addr, &thread_tx);
                });
            }
        });
        rx
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
