use async_std::{io::prelude::*, net::{TcpListener, TcpStream}, stream::StreamExt, task};
use async_tls::TlsAcceptor;
use once_cell::sync::Lazy;
use rustls::{ServerConfig, NoClientAuth, internal::pemfile::{certs, pkcs8_private_keys}};
use std::{error::Error, ffi::OsStr, fs::File, io::BufReader, marker::Unpin, sync::Arc};
use url::Url;

fn main() -> Result {
    task::block_on(async {
        let listener = TcpListener::bind(&ARGS.sock_addr).await?;
        let mut incoming = listener.incoming();
        while let Some(Ok(stream)) = incoming.next().await {
            task::spawn(async {
                if let Err(e) = handle_request(stream).await {
                    eprintln!("Error: {:?}", e);
                }
            });
        }
        Ok(())
    })
}

type Result<T=()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

static ARGS: Lazy<Args> = Lazy::new(|| args().unwrap_or_else(|| {
    eprintln!("usage: agate <addr:port> <dir> <cert> <key>");
    std::process::exit(1);
}));

struct Args {
    sock_addr: String,
    content_dir: String,
    cert_file: String,
    key_file: String,
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

/// Handle a single client session (request + response).
async fn handle_request(stream: TcpStream) -> Result {
    // Perform handshake.
    static TLS: Lazy<TlsAcceptor> = Lazy::new(|| acceptor().unwrap());
    let mut stream = TLS.accept(stream).await?;

    match parse_request(&mut stream).await {
        Ok(url) => {
            eprintln!("Got request for {:?}", url);
            if let Err(e) = send_response(&url, &mut stream).await {
                stream.write_all(b"51 Not found, sorry.\r\n").await?;
                return Err(e)
            }
        }
        Err(e) => {
            stream.write_all(b"59 Invalid request.\r\n").await?;
            return Err(e)
        }
    }
    Ok(())
}

/// TLS configuration.
fn acceptor() -> Result<TlsAcceptor> {
    let cert_file = File::open(&ARGS.cert_file)?;
    let certs = certs(&mut BufReader::new(cert_file)).or(Err("bad cert"))?;

    let key_file = File::open(&ARGS.key_file)?;
    let mut keys = pkcs8_private_keys(&mut BufReader::new(key_file)).or(Err("bad key"))?;

    let mut config = ServerConfig::new(NoClientAuth::new());
    config.set_single_cert(certs, keys.remove(0))?;
    Ok(TlsAcceptor::from(Arc::new(config)))
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
    let request = std::str::from_utf8(&request[..len - 2])?;

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
    let mut path = std::path::PathBuf::from(&ARGS.content_dir);
    if let Some(segments) = url.path_segments() {
        path.extend(segments);
    }
    if async_std::fs::metadata(&path).await?.is_dir() {
        if url.as_str().ends_with('/') {
            path.push("index.gmi");
        } else {
            return redirect_slash(url, stream).await;
        }
    }

    let mut file = async_std::fs::File::open(&path).await?;
    
    // Send header.
    if path.extension() == Some(OsStr::new("gmi")) {
        stream.write_all(b"20 text/gemini\r\n").await?;
    } else {
        let mime = tree_magic_mini::from_filepath(&path).ok_or("Can't read file")?;
        stream.write_all(b"20 ").await?;
        stream.write_all(mime.as_bytes()).await?;
        stream.write_all(b"\r\n").await?;
    }
    
    // Send body.
    async_std::io::copy(&mut file, &mut stream).await?;
    Ok(())
}

/// Send a redirect when the URL for a directory is missing a trailing slash.
async fn redirect_slash<W: Write + Unpin>(url: &Url, mut stream: W) -> Result {
    stream.write_all(b"31 ").await?;
    stream.write_all(url.as_str().as_bytes()).await?;
    stream.write_all(b"/\r\n").await?;
    Ok(())
}
