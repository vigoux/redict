use std::net::TcpStream;
use redict::protocol::{
    connection::DICTConnection,
    Database, Strategy
};

fn main() {
    let addr = std::env::args().next_back().unwrap();
    let stream = TcpStream::connect(addr).expect("Invalid socket address");

    let mut conn = DICTConnection::new(stream).unwrap();
    conn.next();

    let (matches, _) = conn.match_db(Database::all(), Strategy::default(), String::from("ti")).expect("Damn");

    for m in matches {
        println!("{} : {}", m.source.desc, m.word);
    }
}
