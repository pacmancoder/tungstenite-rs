use std::{net::TcpListener, thread::spawn, env};

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};

fn main() {
    env_logger::init();

    let listener_addr = env::var("WS_LISTENER_ADDR").unwrap_or("0.0.0.0:3012".into());
    println!("Listening at {} (env: WS_LISTENER_ADDR)", listener_addr);

    let server = TcpListener::bind(&listener_addr).unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = tungstenite::accept(stream.unwrap()).expect("handshake failed");

            println!("Handshake completed");

            loop {
                let msg = websocket.read_message().unwrap();
                if msg.is_binary() || msg.is_text() {
                    let new_msg = format!("Got message: {}", msg);
                    println!("{}", new_msg);
                    websocket.write_message(msg).unwrap();
                }
            }
        });
    }
}
