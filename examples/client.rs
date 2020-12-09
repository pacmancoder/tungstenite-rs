use tungstenite::{connect, Message};
use url::Url;
use std::env;

fn main() {
    env_logger::init();

    let listener_addr = env::var("WS_PROXY_ADDR").unwrap_or("127.0.0.1:8999".into());

    println!("Connecting to proxy at {} (env: WS_PROXY_ADDR)", listener_addr);

    let (mut socket, response) =
        connect(Url::parse(&format!("ws://{}", listener_addr)).unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    socket.write_message(Message::Text("Hello WebSocket".into())).unwrap();
    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
}
