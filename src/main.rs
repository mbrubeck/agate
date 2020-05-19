use {
    async_std::{
        net::{TcpListener, TcpStream},
        prelude::*,
        task,
    },
    async_tls::{TlsAcceptor, server::TlsStream},
    lazy_static::lazy_static,
    rustls::internal::pemfile::{certs, pkcs8_private_keys},
    std::{
        error::Error,
        fs::{File, read},
        io::BufReader,
        path::{Path, PathBuf},
        sync::Arc,
    },
    url::Url,
};

pub type Result<T=()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result {
    let addr = "localhost:1965";

    task::block_on(async {
        let listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();

        while let Some(Ok(stream)) = incoming.next().await {
            task::spawn(async {
                if let Err(e) = connection(stream).await {
                    eprintln!("Error: {:?}", e);
                }
            });
        }
        Ok(())
    })
}

async fn connection(stream: TcpStream) -> Result {
    let mut stream = TLS_ACCEPTOR.accept(stream).await?;
    let url = match parse_request(&mut stream).await {
        Ok(url) => url,
        Err(e) => {
            stream.write_all(b"50 Invalid request.\r\n").await?;
            return Err(e)
        }
    };
    match get(&url) {
        Ok(response) => {
            stream.write_all(b"20 text/gemini\r\n").await?;
            stream.write_all(&response).await?;
        }
        Err(e) => {
            stream.write_all(b"40 Not found, sorry.\r\n").await?;
            return Err(e)
        }
    }
    Ok(())
}

lazy_static! {
    static ref TLS_ACCEPTOR: TlsAcceptor = {
        let cert_file = File::open("tests/cert.pem").unwrap();
        let certs = certs(&mut BufReader::new(cert_file)).unwrap();

        let key_file = File::open("tests/key.rsa").unwrap();
        let mut keys = pkcs8_private_keys(&mut BufReader::new(key_file)).unwrap();

        let mut config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        config.set_single_cert(certs, keys.remove(0)).unwrap();
        TlsAcceptor::from(Arc::new(config))
    };
}

async fn parse_request(stream: &mut TlsStream<TcpStream>) -> Result<Url> {
    let mut stream = async_std::io::BufReader::new(stream);
    let mut request = String::new();
    stream.read_line(&mut request).await?;
    let url = Url::parse(request.trim())?;
    Ok(url)
}

fn get(url: &Url) -> Result<Vec<u8>> {
    let path: PathBuf = url.path_segments().unwrap().collect();
    let path = Path::new(".").join(path).canonicalize()?;
    if !path.starts_with(std::env::current_dir()?) {
        Err("invalid path")?
    }
    let response = read(path)?;
    Ok(response)
}
