use {
    async_std::{
        io::prelude::*,
        net::{TcpListener, TcpStream},
        path::PathBuf,
        stream::StreamExt,
        task::{block_on, spawn},
    },
    async_tls::TlsAcceptor,
    once_cell::sync::Lazy,
    std::{error::Error, ffi::OsStr, marker::Unpin, str, sync::Arc},
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
    block_on(async {
        let listener = TcpListener::bind(&ARGS.sock_addr).await?;
        let mut incoming = listener.incoming();
        while let Some(Ok(stream)) = incoming.next().await {
            spawn(async {
                if let Err(e) = connection(stream).await {
                    eprintln!("Error: {:?}", e);
                }
            });
        }
        Ok(())
    })
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

static ARGS: Lazy<Args> =
    Lazy::new(|| args().expect("usage: agate <addr:port> <dir> <cert> <key>"));

fn acceptor() -> Result<TlsAcceptor> {
    use rustls::{ServerConfig, NoClientAuth, internal::pemfile::{certs, pkcs8_private_keys}};
    use std::{io::BufReader, fs::File};

    let cert_file = File::open(&ARGS.cert_file)?;
    let certs = certs(&mut BufReader::new(cert_file)).or(Err("bad cert"))?;

    let key_file = File::open(&ARGS.key_file)?;
    let mut keys = pkcs8_private_keys(&mut BufReader::new(key_file)).or(Err("bad key"))?;

    let mut config = ServerConfig::new(NoClientAuth::new());
    config.set_single_cert(certs, keys.remove(0))?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Handle a single client session (request + response).
async fn connection(stream: TcpStream) -> Result {
    static ACCEPTOR: Lazy<TlsAcceptor> = Lazy::new(|| acceptor().unwrap());

    let mut stream = ACCEPTOR.accept(stream).await?;
    match parse_request(&mut stream).await {
        Ok(url) => {
            eprintln!("Got request for {:?}", url);
            send_response(&url, &mut stream).await
        }
        Err(e) => {
            stream.write_all(b"59 Invalid request.\r\n").await?;
            Err(e)
        }
    }
}

/// Return the URL requested by the client.
async fn parse_request<R: Read + Unpin>(mut stream: R) -> Result<Url> {
    // Because requests are limited to 1024 bytes (plus 2 bytes for CRLF), we
    // can use a fixed-sized buffer on the stack, avoiding allocations and
    // copying, and stopping bad clients from making us use too much memory.
    let mut request = [0; 1026];
    let mut buf = &mut request[..];
    let mut len = 0;

    // Read until CRLF, end-of-stream, or there's no buffer space left.
    loop {
        let bytes_read = stream.read(buf).await?;
        len += bytes_read;
        if request[..len].ends_with(b"\r\n") {
            break;
        } else if bytes_read == 0 {
            Err("Request ended unexpectedly")?
        }
        buf = &mut request[len..];
    }
    let request = str::from_utf8(&request[..len - 2])?;

    // Handle scheme-relative URLs.
    let url = if request.starts_with("//") {
        Url::parse(&format!("gemini:{}", request))?
    } else {
        Url::parse(request)?
    };

    // Validate the URL. TODO: Check the hostname and port.
    if url.scheme() != "gemini" {
        Err("unsupported URL scheme")?
    }
    Ok(url)
}

/// Send the client the file located at the requested URL.
async fn send_response<W: Write + Unpin>(url: &Url, mut stream: W) -> Result {
    let mut path = PathBuf::from(&ARGS.content_dir);
    if let Some(segments) = url.path_segments() {
        path.extend(segments);
    }
    if path.is_dir().await {
        if url.as_str().ends_with('/') {
            path.push("index.gmi");
        } else {
            return redirect_slash(url, stream).await;
        }
    }
    match async_std::fs::read(&path).await {
        Ok(body) => {
            if path.extension() == Some(OsStr::new("gmi")) {
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

/// Send a redirect when the URL for a directory is missing a trailing slash.
async fn redirect_slash<W: Write + Unpin>(url: &Url, mut stream: W) -> Result {
    stream.write_all(b"31 ").await?;
    stream.write_all(url.as_str().as_bytes()).await?;
    stream.write_all(b"/\r\n").await?;
    return Ok(())
}
