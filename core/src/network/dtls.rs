use std::{net::SocketAddr, pin::Pin, str::FromStr, time::Duration};
use udp_stream::{UdpListener, UdpStream};

use openssl::{
    pkey::PKey,
    ssl::{Ssl, SslAcceptor, SslContext, SslMethod, SslVerifyMode, SslConnector},
    x509::X509,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::timeout,
};

use crate::structs::{GDPChannel, GDPName, GDPPacket, GdpAction};
use tokio::{
    sync::mpsc::{self, Sender},
}; 

const UDP_BUFFER_SIZE: usize = 17480; // 17kb
const UDP_TIMEOUT: u64 = 10 * 1000; // 10sec

static SERVER_CERT: &'static [u8] = include_bytes!("../../resources/router.pem");
static SERVER_KEY: &'static [u8] = include_bytes!("../../resources/router-private.pem");
const SERVER_DOMAIN: &'static str = "pourali.com";

fn ssl_acceptor(certificate: &[u8], private_key: &[u8]) -> std::io::Result<SslContext> {
    let mut acceptor_builder = SslAcceptor::mozilla_intermediate(SslMethod::dtls())?;
    acceptor_builder.set_certificate(&&X509::from_pem(certificate)?)?;
    acceptor_builder.set_private_key(&&PKey::private_key_from_pem(private_key)?)?;
    acceptor_builder.check_private_key()?;
    let acceptor = acceptor_builder.build();
    Ok(acceptor.into_context())
}

pub async fn dtls_listener(
    addr: &'static str, rib_tx: Sender<GDPPacket>, channel_tx: Sender<GDPChannel>,
)  -> std::io::Result<()> {
    let listener = UdpListener::bind(SocketAddr::from_str("127.0.0.1:8989").unwrap()).await?;
    let acceptor = ssl_acceptor(SERVER_CERT, SERVER_KEY).unwrap();
    loop {
        let (socket, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            let ssl = Ssl::new(&acceptor).unwrap();
            let mut stream = tokio_openssl::SslStream::new(ssl, socket).unwrap();
            Pin::new(&mut stream).accept().await.unwrap();
            let mut buf = vec![0u8; UDP_BUFFER_SIZE];
            loop {
                let n = match timeout(Duration::from_millis(UDP_TIMEOUT), stream.read(&mut buf))
                    .await
                    .unwrap()
                {
                    Ok(len) => len,
                    Err(_) => {
                        return;
                    }
                };
                stream.write_all(&buf[0..n]).await.unwrap();
            }
        });
    }
}



async fn dtls_test_client() -> std::io::Result<SslContext>{
    let stream = UdpStream::connect(SocketAddr::from_str("127.0.0.1:8080").unwrap()).await?;

    let mut connector_builder = SslConnector::builder(SslMethod::dtls())?;
    connector_builder.set_verify(SslVerifyMode::NONE);
    let connector = connector_builder.build().configure().unwrap();
    let ssl = connector.into_ssl(SERVER_DOMAIN).unwrap();
    let mut stream = tokio_openssl::SslStream::new(ssl, stream).unwrap();
    Pin::new(&mut stream).connect().await.unwrap();
    let mut buffer = String::new();
    loop {
        std::io::stdin().read_line(&mut buffer)?;
        stream.write_all(buffer.as_bytes()).await?;
        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).await?;
        print!("-> {}", String::from_utf8_lossy(&buf[..n]));
    }
}