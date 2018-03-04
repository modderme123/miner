use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
pub enum Message {
    Move(SocketAddr, Point),
    Add(SocketAddr, Point),
    Terrain(Vec<Vec<bool>>),
    Remove(SocketAddr),
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub pos: (f64, f64),
    pub vel: (f64, f64),
}
