use tungstenite::{connect, client, Message};
use url::Url;
use rustls::{ClientConfig, Session};
use std::{
    sync::Arc,
    net::{
        TcpStream,
        SocketAddr,
    },
};

pub struct NoCertificateVerification;

impl rustls::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef<'_>,
        _ocsp: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}

fn main() {
    env_logger::init();

    let mut client_config = ClientConfig::new();
    client_config.dangerous().set_certificate_verifier(Arc::new(NoCertificateVerification));
    let config_ref = Arc::new(client_config);
    let dns_name = webpki::DNSNameRef::try_from_ascii_str("stub_string").unwrap();
    let mut session = rustls::ClientSession::new(&config_ref, dns_name);
    let mut socket = TcpStream::connect("127.0.0.1:8999".parse::<SocketAddr>().unwrap())
        .expect("Connection failed");

    let tls_stream = rustls::Stream::new(&mut session, &mut socket);


    let (mut socket, response) =
        client(Url::parse("wss://localhost:1613").unwrap(), tls_stream)
            .expect("can't perform websocket handshake");

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
    // socket.close(None);
}
