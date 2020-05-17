use {
    async_std::{
        net::{TcpListener, TcpStream},
        prelude::*,
        task,
    },
    async_tls::TlsAcceptor,
    rustls::{
        internal::pemfile::{certs, rsa_private_keys},
    },
    std::{
        error::Error,
        fs::File,
        io::{BufReader, self},
        sync::Arc,
    },
};

pub type Result<T=()> = std::result::Result<T, Box<dyn Error>>;

async fn connection(acceptor: TlsAcceptor, stream: TcpStream) -> io::Result<()> {
    let stream = acceptor.accept(stream).await?;
    let mut stream = async_std::io::BufReader::new(stream);
    let mut body = String::new();
    stream.read_line(&mut body).await?;
    let mut stream = stream.into_inner();
    stream.write_all(b"20 text/plain\r\n").await?;
    stream.write_all(body.as_bytes()).await?;
    Ok(())
}

fn main() -> Result {
    env_logger::init();

    let certs = certs(&mut BufReader::new(File::open("tests/cert.pem")?))
        .expect("Error reading certificate file");
    let mut keys = rsa_private_keys(&mut BufReader::new(File::open("tests/key.rsa")?))
        .expect("Error reading private key file");

    let mut config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    config.set_single_cert(certs, keys.remove(0))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let addr = "localhost:1965";

    task::block_on(async {
        let listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let acceptor = acceptor.clone();
            let stream = stream?;
            task::spawn(async {
                if let Err(e) = connection(acceptor, stream).await {
                    eprintln!("Error: {:?}", e);
                }
            });
        }

        Ok(())
    })
}
