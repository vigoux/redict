use redict::reply::Reply;
use redict::commands::{Definition, define};
use std::io::prelude::*;
use std::net::TcpStream;
use std::io::BufReader;

fn main() {
    let addr = std::env::args().next_back().unwrap();
    let mut stream = TcpStream::connect(addr).unwrap();


    // First answer
    if !Reply::from_reader(&mut BufReader::new(&mut stream)).unwrap().status.is_start() {
        panic!("Wowowo");
    }

    for def in define(&mut stream, String::from("revo_eo"), String::from("miro")).unwrap() {
        println!("From {}", def.source);

        for line in def.text {
            println!("{}", line);
        }
    }
}
