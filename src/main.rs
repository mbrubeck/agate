use {
    async_std::{
        net::{TcpListener, TcpStream},
        prelude::*,
        task,
    },
    async_tls::TlsAcceptor,
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
    let certs = certs(&mut BufReader::new(File::open("tests/cert.pem")?))
        .expect("Error reading certificate file");
    let mut keys = pkcs8_private_keys(&mut BufReader::new(File::open("tests/key.rsa")?))
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

async fn connection(acceptor: TlsAcceptor, stream: TcpStream) -> Result {
    let stream = acceptor.accept(stream).await?;

    let mut stream = async_std::io::BufReader::new(stream);
    let mut request = String::new();
    stream.read_line(&mut request).await?;
    let url = Url::parse(request.trim())?;
    eprintln!("Got request: {:?}", url);

    let mut stream = stream.into_inner();
    match get(&url) {
        Ok(response) => {
            stream.write_all(b"20 text/gemini\r\n").await?;
            stream.write_all(&response).await?;
        }
        Err(_) => {
            stream.write_all(b"40 Not found, sorry.\r\n").await?;
        }
    }
    Ok(())
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
