use async_std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    stream::StreamExt,
    task,
};
use async_tls::TlsAcceptor;
use once_cell::sync::Lazy;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, percent_encode};
use rustls::{
    internal::pemfile::{certs, pkcs8_private_keys},
    NoClientAuth, ServerConfig,
};
use std::{
    borrow::Cow,
    error::Error,
    ffi::OsStr,
    fs::File,
    io::BufReader,
    marker::Unpin,
    path::Path,
    sync::Arc,
};
use url::{Host, Url};

fn main() -> Result {
    if !ARGS.silent {
        env_logger::Builder::new().parse_filters("info").init();
    }
    task::block_on(async {
        let listener = TcpListener::bind(&ARGS.sock_addr).await?;
        let mut incoming = listener.incoming();
        log::info!("Listening on {}...", ARGS.sock_addr);
        while let Some(Ok(stream)) = incoming.next().await {
            task::spawn(async {
                if let Err(e) = handle_request(stream).await {
                    log::error!("{:?}", e);
                }
            });
        }
        Ok(())
    })
}

type Result<T = (), E = Box<dyn Error + Send + Sync>> = std::result::Result<T, E>;

static ARGS: Lazy<Args> = Lazy::new(|| {
    args().unwrap_or_else(|s| {
        eprintln!("{}", s);
        std::process::exit(1);
    })
});

struct Args {
    sock_addr: String,
    content_dir: String,
    cert_file: String,
    key_file: String,
    hostname: Option<Host>,
    language: Option<String>,
    silent: bool,
}

fn args() -> Result<Args> {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optopt("", "content", "Root of the content directory (default ./content)", "DIR");
    opts.optopt("", "cert", "TLS certificate PEM file (default ./cert.pem)", "FILE");
    opts.optopt("", "key", "PKCS8 private key file (default ./key.rsa)", "FILE");
    opts.optopt("", "addr", "Address to listen on (default 0.0.0.0:1965)", "IP:PORT");
    opts.optopt("", "hostname", "Domain name of this Gemini server (optional)", "NAME");
    opts.optopt("", "lang", "RFC 4646 Language code(s) for text/gemini documents", "LANG");
    opts.optflag("s", "silent", "Disable logging output");
    opts.optflag("h", "help", "Print this help menu");

    let usage = opts.usage(&format!("Usage: {} FILE [options]", &args[0]));
    let matches = opts.parse(&args[1..]).map_err(|f| f.to_string())?;
    if matches.opt_present("h") {
        Err(usage)?;
    }
    let hostname = match matches.opt_str("hostname") {
        Some(s) => Some(Host::parse(&s)?),
        None => None,
    };
    Ok(Args {
        sock_addr: matches.opt_get_default("addr", "0.0.0.0:1965".into())?,
        content_dir: check_path(matches.opt_get_default("content", "content".into())?)?,
        cert_file: check_path(matches.opt_get_default("cert", "cert.pem".into())?)?,
        key_file: check_path(matches.opt_get_default("key", "key.rsa".into())?)?,
        language: matches.opt_str("lang"),
        silent: matches.opt_present("s"),
        hostname,
    })
}

fn check_path(s: String) -> Result<String, String> {
    if Path::new(&s).exists() {
        Ok(s)
    } else {
        Err(format!("No such file: {}", s))
    }
}

/// Handle a single client session (request + response).
async fn handle_request(stream: TcpStream) -> Result {
    // Perform handshake.
    static TLS: Lazy<TlsAcceptor> = Lazy::new(|| acceptor().unwrap());
    let stream = &mut TLS.accept(stream).await?;

    let url = match parse_request(stream).await {
        Ok(url) => url,
        Err((status, msg)) => {
            send_header(stream, status, &[&msg]).await?;
            Err(msg)?
        }
    };
    if let Err(e) = send_response(url, stream).await {
        send_header(stream, 51, &["Not found, sorry."]).await?;
        Err(e)?
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
async fn parse_request<R: Read + Unpin>(
    stream: &mut R,
) -> std::result::Result<Url, (u8, &'static str)> {
    // Because requests are limited to 1024 bytes (plus 2 bytes for CRLF), we
    // can use a fixed-sized buffer on the stack, avoiding allocations and
    // copying, and stopping bad clients from making us use too much memory.
    let mut request = [0; 1026];
    let mut buf = &mut request[..];
    let mut len = 0;

    // Read until CRLF, end-of-stream, or there's no buffer space left.
    loop {
        let bytes_read = stream
            .read(buf)
            .await
            .map_err(|_| (59, "Request ended unexpectedly"))?;
        len += bytes_read;
        if request[..len].ends_with(b"\r\n") {
            break;
        } else if bytes_read == 0 {
            return Err((59, "Request ended unexpectedly"));
        }
        buf = &mut request[len..];
    }
    let request = std::str::from_utf8(&request[..len - 2]).map_err(|_| (59, "Invalid URL"))?;
    log::info!("Got request for {:?}", request);

    // Handle scheme-relative URLs.
    let url = if request.starts_with("//") {
        Url::parse(&format!("gemini:{}", request)).map_err(|_| (59, "Invalid URL"))?
    } else {
        Url::parse(request).map_err(|_| (59, "Invalid URL"))?
    };

    // Validate the URL, host and port.
    if url.scheme() != "gemini" {
        return Err((53, "unsupported URL scheme"));
    }
    // TODO: Can be simplified by https://github.com/servo/rust-url/pull/651
    if let (Some(Host::Domain(expected)), Some(Host::Domain(host))) = (url.host(), &ARGS.hostname) {
        if host != expected {
            return Err((53, "proxy request refused"));
        }
    }
    if let Some(port) = url.port() {
        if !ARGS.sock_addr.ends_with(&format!(":{}", port)) {
            return Err((53, "proxy request refused"));
        }
    }
    Ok(url)
}

/// Send the client the file located at the requested URL.
async fn send_response<W: Write + Unpin>(url: Url, stream: &mut W) -> Result {
    let mut path = std::path::PathBuf::from(&ARGS.content_dir);
    if let Some(segments) = url.path_segments() {
        for segment in segments {
            path.push(&*percent_decode_str(segment).decode_utf8()?);
        }
    }
    if async_std::fs::metadata(&path).await?.is_dir() {
        if url.path().ends_with('/') || url.path().is_empty() {
            // if the path ends with a slash or the path is empty, the links will work the same
            // without a redirect
            path.push("index.gmi");
            if !path.exists() && path.with_file_name(".directory-listing-ok").exists() {
                path.pop();
                return list_directory(stream, &path).await;
            }
        } else {
            // if client is not redirected, links may not work as expected without trailing slash
            return send_header(stream, 31, &[url.as_str(), "/"]).await;
        }
    }

    // Make sure the file opens successfully before sending the success header.
    let mut file = async_std::fs::File::open(&path).await?;

    // Send header.
    if path.extension() == Some(OsStr::new("gmi")) {
        send_text_gemini_header(stream).await?;
    } else {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        send_header(stream, 20, &[mime.essence_str()]).await?;
    }

    // Send body.
    async_std::io::copy(&mut file, stream).await?;
    Ok(())
}

async fn send_header<W: Write + Unpin>(stream: &mut W, status: u8, meta: &[&str]) -> Result {
    use std::fmt::Write;
    let mut response = String::with_capacity(64);
    write!(response, "{} ", status)?;
    response.extend(meta.iter().copied());
    log::info!("Responding with status {:?}", response);
    response.push_str("\r\n");
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn list_directory<W: Write + Unpin>(stream: &mut W, path: &Path) -> Result {
    // https://url.spec.whatwg.org/#path-percent-encode-set
    const ENCODE_SET: AsciiSet = CONTROLS.add(b' ')
        .add(b'"').add(b'#').add(b'<').add(b'>')
        .add(b'?').add(b'`').add(b'{').add(b'}');

    log::info!("Listing directory {:?}", path);
    send_text_gemini_header(stream).await?;
    let mut entries = async_std::fs::read_dir(path).await?;
    let mut lines = vec![];
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let mut name = entry.file_name().into_string().or(Err("Non-Unicode filename"))?;
        if name.starts_with('.') {
            continue;
        }
        if entry.file_type().await?.is_dir() {
            name += "/";
        }
        let line = match percent_encode(name.as_bytes(), &ENCODE_SET).into() {
            Cow::Owned(url) => format!("=> {} {}\n", url, name),
            Cow::Borrowed(url) => format!("=> {}\n", url), // url and name are identical
        };
        lines.push(line);
    }
    lines.sort();
    for line in lines {
        stream.write_all(line.as_bytes()).await?;
    }
    Ok(())
}

async fn send_text_gemini_header<W: Write + Unpin>(stream: &mut W) -> Result {
    if let Some(lang) = ARGS.language.as_deref() {
        send_header(stream, 20, &["text/gemini;lang=", lang]).await
    } else {
        send_header(stream, 20, &["text/gemini"]).await
    }
}
