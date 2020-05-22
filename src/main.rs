use {
    async_std::{
        io::prelude::*,
        net::{TcpListener, TcpStream},
        path::PathBuf,
        stream::StreamExt,
        task,
    },
    async_tls::{TlsAcceptor, server::TlsStream},
    lazy_static::lazy_static,
    std::{error::Error, ffi::OsStr, fs::File, io::BufReader, str, sync::Arc},
    url::Url,
};

pub type Result<T=()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

struct Args {
    sock_addr: String,
    content_dir: String,
    cert_file: String,
    key_file: String,
}

fn main() -> Result {
    task::block_on(async {
        let listener = TcpListener::bind(&ARGS.sock_addr).await?;
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

lazy_static! {
    static ref ARGS: Args = args()
        .expect("usage: agate <addr:port> <dir> <cert> <key>");
    static ref ACCEPTOR: TlsAcceptor = acceptor().unwrap();
}

fn args() -> Option<Args> {
    let mut args = std::env::args().skip(1);
    Some(Args {
        sock_addr: args.next()?,
        content_dir: args.next()?,
        cert_file: args.next()?,
        key_file: args.next()?,
    })
}

fn acceptor() -> Result<TlsAcceptor> {
    use rustls::{ServerConfig, NoClientAuth, internal::pemfile::{certs, pkcs8_private_keys}};

    let cert_file = File::open(&ARGS.cert_file)?;
    let certs = certs(&mut BufReader::new(cert_file)).or(Err("bad cert"))?;

    let key_file = File::open(&ARGS.key_file)?;
    let mut keys = pkcs8_private_keys(&mut BufReader::new(key_file)).or(Err("bad key"))?;

    let mut config = ServerConfig::new(NoClientAuth::new());
    config.set_single_cert(certs, keys.remove(0))?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}

async fn connection(stream: TcpStream) -> Result {
    use async_std::io::prelude::*;
    let mut stream = ACCEPTOR.accept(stream).await?;
    match parse_request(&mut stream).await {
        Ok(url) => {
            eprintln!("Got request for {:?}", url);
            get(&url, &mut stream).await
        }
        Err(e) => {
            stream.write_all(b"59 Invalid request.\r\n").await?;
            Err(e)
        }
    }
}

async fn parse_request(stream: &mut TlsStream<TcpStream>) -> Result<Url> {
    let mut stream = async_std::io::BufReader::new(stream);
    let mut request = Vec::new();
    stream.read_until(b'\r', &mut request).await?;

    // Check line ending.
    let eol = &mut [0];
    stream.read_exact(eol).await?;
    if eol != b"\n" {
        Err("CR without LF")?
    }
    // Check request length.
    if request.len() > 1026 {
        Err("Too long")?
    }
    // Handle scheme-relative URLs.
    if request.starts_with(b"//") {
        request.splice(..0, "gemini:".bytes());
    }
    // Parse URL.
    let url = Url::parse(str::from_utf8(&request)?.trim_end())?;
    if url.scheme() != "gemini" {
        Err("unsupported URL scheme")?
    }
    // TODO: Validate hostname and port.
    Ok(url)
}

async fn get(url: &Url, stream: &mut TlsStream<TcpStream>) -> Result {
    let mut path = PathBuf::from(&ARGS.content_dir);
    if let Some(segments) = url.path_segments() {
        path.extend(segments);
    } else {
        return redirect_slash(url, stream).await;
    }
    if path.is_dir().await {
        if url.as_str().ends_with('/') {
            path.push("index.gemini");
        } else {
            return redirect_slash(url, stream).await;
        }
    }
    match async_std::fs::read(&path).await {
        Ok(body) => {
            if path.extension() == Some(OsStr::new("gemini")) {
                stream.write_all(b"20 text/gemini\r\n").await?;
            } else {
                let mime = tree_magic::from_u8(&body);
                let header = format!("20 {}\r\n", mime);
                stream.write_all(header.as_bytes()).await?;
            }
            stream.write_all(&body).await?;
        }
        Err(e) => {
            stream.write_all(b"51 Not found, sorry.\r\n").await?;
            Err(e)?
        }
    }
    Ok(())
}

async fn redirect_slash(url: &Url, stream: &mut TlsStream<TcpStream>) -> Result {
    stream.write_all(b"31 ").await?;
    stream.write_all(url.as_str().as_bytes()).await?;
    stream.write_all(b"/\r\n").await?;
    return Ok(())
}
