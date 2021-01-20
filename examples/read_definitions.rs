use std::net::TcpStream;
use redict::protocol::{
    connection::DICTConnection,
    Database
};

fn main() {
    let addr = std::env::args().next_back().unwrap();
    let stream = TcpStream::connect(addr).expect("Invalid socket address");

    let mut conn = DICTConnection::new(stream).unwrap();
    conn.next();

    let (defs, _) = conn.define(Database::all(), String::from("ti")).expect("Damn");

    for def in defs {
        println!("{}", def.source.desc);
    }
}
