use {
    async_std::{
        net::TcpListener,
        prelude::*,
        task::block_on,
    },
    rustls::{
        internal::pemfile::{certs, rsa_private_keys},
    },
    std::{
        error::Error,
        fs::File,
        io::BufReader,
    },
};

pub type Result<T=()> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result {
    let certs = certs(&mut BufReader::new(File::open("tests/cert.pem")?))
        .expect("Error reading certificate file");
    let mut keys = rsa_private_keys(&mut BufReader::new(File::open("tests/key.pem")?))
        .expect("Error reading private key file");

    let mut config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    config.set_single_cert(certs, keys.remove(0))?;

    let addr = "0.0.0.0:1965";

    block_on(async {
        let listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
        }

        Ok(())
    })
}
