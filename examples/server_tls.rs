use std::{net::TcpListener, thread::spawn, fs::File, io::BufReader, io, net::SocketAddr, net::TcpStream, env};
use rustls::{
    ServerConfig,
};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};
use std::sync::Arc;

fn main() {
    env_logger::init();

    let listener_addr = env::var("WS_LISTENER_ADDR").unwrap_or("0.0.0.0:3012".into());
    println!("Listening at {} (env: WS_LISTENER_ADDR)", listener_addr);

    let cert_path = env::var("WS_LISTENER_PUBLIC_CERT_PATH").unwrap_or("publicCert.pem".into());
    let key_path = env::var("WS_LISTENER_CERT_KEY_PATH").unwrap_or("private.pem".into());

    println!("Using public cert at {} (env: WS_LISTENER_PUBLIC_CERT_PATH)", cert_path);
    println!("Using cert key at {} (env: WS_LISTENER_CERT_KEY_PATH)", key_path);

    let client_no_auth = rustls::NoClientAuth::new();
    let mut server_config = rustls::ServerConfig::new(client_no_auth);
    let mut cert_file = File::open(cert_path).expect("Failed to open cert");
    let mut cert_reader = BufReader::new(cert_file);
    let certs = rustls::internal::pemfile::certs(&mut cert_reader).expect("Failed to parse certificate");

    let keyfile = File::open(key_path).unwrap_or_else(|_| panic!("cannot open private key file"));
    let pri_key = rustls::internal::pemfile::pkcs8_private_keys(&mut BufReader::new(keyfile)).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "File contains invalid pkcs8 private key (encrypted keys not supported)",
        )
    }).expect("failed to read priv key");

    let priv_key = pri_key.first().expect("first priv key does not exist").clone();

    server_config
        .set_single_cert(certs, priv_key)
        .map_err(|_| "couldn't set server config cert").unwrap();

    let server_config_ref = Arc::new(server_config);

    let mut socket = TcpListener::bind(listener_addr.parse::<SocketAddr>().unwrap())
        .expect("can't bind port");

    println!("waiting for connection...");

    for mut stream in socket.incoming() {
        let server_config_ref = server_config_ref.clone();
        spawn(move || {
            println!("creating tls...");
            let mut session = rustls::ServerSession::new(&server_config_ref);
            let mut stream = stream.expect("stream is invalid");
            let tls_stream = rustls::Stream::new(&mut session, &mut stream);

            println!("making websocket handshake...");
            let mut websocket = tungstenite::accept(tls_stream).expect("handshake failed");
            println!("handshaked!");
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
