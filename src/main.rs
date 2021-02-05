mod metadata;
use metadata::{FileOptions, PresetMeta};

use {
    once_cell::sync::Lazy,
    percent_encoding::{percent_decode_str, percent_encode, AsciiSet, CONTROLS},
    rustls::{
        internal::pemfile::{certs, pkcs8_private_keys},
        NoClientAuth, ServerConfig,
    },
    std::{
        borrow::Cow, error::Error, ffi::OsStr, fmt::Write, fs::File, io::BufReader,
        net::SocketAddr, path::Path, sync::Arc,
    },
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
        runtime::Runtime,
        sync::Mutex,
    },
    tokio_rustls::{server::TlsStream, TlsAcceptor},
    url::{Host, Url},
};

fn main() -> Result {
    if !ARGS.silent {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .parse_default_env()
            .init();
    }
    Runtime::new()?.block_on(async {
        let mimetypes = Arc::new(Mutex::new(FileOptions::new(PresetMeta::Parameters(
            ARGS.language
                .as_ref()
                .map_or(String::new(), |lang| format!(";lang={}", lang)),
        ))));
        let listener = TcpListener::bind(&ARGS.addrs[..]).await?;
        log::info!("Listening on {:?}...", ARGS.addrs);
        loop {
            let (stream, _) = listener.accept().await?;
            let arc = mimetypes.clone();
            tokio::spawn(async {
                match RequestHandle::new(stream, arc).await {
                    Ok(handle) => match handle.handle().await {
                        Ok(info) => log::info!("{}", info),
                        Err(err) => log::warn!("{}", err),
                    },
                    Err(log_line) => {
                        log::warn!("{}", log_line);
                    }
                }
            });
        }
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
    addrs: Vec<SocketAddr>,
    content_dir: String,
    cert_file: String,
    key_file: String,
    hostname: Option<Host>,
    language: Option<String>,
    silent: bool,
    serve_secret: bool,
    log_ips: bool,
}

fn args() -> Result<Args> {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optopt(
        "",
        "content",
        "Root of the content directory (default ./content)",
        "DIR",
    );
    opts.optopt(
        "",
        "cert",
        "TLS certificate PEM file (default ./cert.pem)",
        "FILE",
    );
    opts.optopt(
        "",
        "key",
        "PKCS8 private key file (default ./key.rsa)",
        "FILE",
    );
    opts.optopt(
        "",
        "addr",
        "Address to listen on (multiple occurences possible, default 0.0.0.0:1965 and [::]:1965)",
        "IP:PORT",
    );
    opts.optopt(
        "",
        "hostname",
        "Domain name of this Gemini server (optional)",
        "NAME",
    );
    opts.optopt(
        "",
        "lang",
        "RFC 4646 Language code(s) for text/gemini documents",
        "LANG",
    );
    opts.optflag("s", "silent", "Disable logging output");
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag(
        "",
        "serve-secret",
        "Enable serving secret files (files/directories starting with a dot)",
    );
    opts.optflag("", "log-ip", "Output IP addresses when logging");

    let matches = opts.parse(&args[1..]).map_err(|f| f.to_string())?;
    if matches.opt_present("h") {
        let usage = opts.usage(&format!("Usage: {} [options]", &args[0]));
        Err(usage)?;
    }
    let hostname = match matches.opt_str("hostname") {
        Some(s) => Some(Host::parse(&s)?),
        None => None,
    };
    let mut addrs = vec![];
    for i in matches.opt_strs("addr") {
        addrs.push(i.parse()?);
    }
    if addrs.is_empty() {
        addrs = vec![
            "[::]:1965".parse().unwrap(),
            "0.0.0.0:1965".parse().unwrap(),
        ];
    }
    Ok(Args {
        addrs,
        content_dir: check_path(matches.opt_get_default("content", "content".into())?)?,
        cert_file: check_path(matches.opt_get_default("cert", "cert.pem".into())?)?,
        key_file: check_path(matches.opt_get_default("key", "key.rsa".into())?)?,
        hostname,
        language: matches.opt_str("lang"),
        silent: matches.opt_present("s"),
        serve_secret: matches.opt_present("serve-secret"),
        log_ips: matches.opt_present("log-ip"),
    })
}

fn check_path(s: String) -> Result<String, String> {
    if Path::new(&s).exists() {
        Ok(s)
    } else {
        Err(format!("No such file: {}", s))
    }
}

/// TLS configuration.
static TLS: Lazy<TlsAcceptor> = Lazy::new(|| acceptor().unwrap());

fn acceptor() -> Result<TlsAcceptor> {
    let cert_file = File::open(&ARGS.cert_file)?;
    let certs = certs(&mut BufReader::new(cert_file)).or(Err("bad cert"))?;

    let key_file = File::open(&ARGS.key_file)?;
    let mut keys = pkcs8_private_keys(&mut BufReader::new(key_file)).or(Err("bad key"))?;

    let mut config = ServerConfig::new(NoClientAuth::new());
    config.set_single_cert(certs, keys.remove(0))?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}

struct RequestHandle {
    stream: TlsStream<TcpStream>,
    log_line: String,
    metadata: Arc<Mutex<FileOptions>>,
}

impl RequestHandle {
    /// Creates a new request handle for the given stream. If establishing the TLS
    /// session fails, returns a corresponding log line.
    async fn new(stream: TcpStream, metadata: Arc<Mutex<FileOptions>>) -> Result<Self, String> {
        let log_line = format!(
            "{} {}",
            stream.local_addr().unwrap(),
            if ARGS.log_ips {
                stream
                    .peer_addr()
                    .expect("could not get peer address")
                    .to_string()
            } else {
                // Do not log IP address, but something else so columns still line up.
                "-".into()
            }
        );

        match TLS.accept(stream).await {
            Ok(stream) => Ok(Self {
                stream,
                log_line,
                metadata,
            }),
            Err(e) => Err(format!("{} error:{}", log_line, e)),
        }
    }

    /// Do the necessary actions to handle this request. Returns a corresponding
    /// log line as Err or Ok, depending on if the request finished with or
    /// without errors.
    async fn handle(mut self) -> Result<String, String> {
        // not already in error condition
        let result = match self.parse_request().await {
            Ok(url) => self.send_response(url).await,
            Err((status, msg)) => self.send_header(status, msg).await,
        };

        if let Err(e) = result {
            Err(format!("{} error:{}", self.log_line, e))
        } else if let Err(e) = self.stream.shutdown().await {
            Err(format!("{} error:{}", self.log_line, e))
        } else {
            Ok(self.log_line)
        }
    }

    /// Return the URL requested by the client.
    async fn parse_request(&mut self) -> std::result::Result<Url, (u8, &'static str)> {
        // Because requests are limited to 1024 bytes (plus 2 bytes for CRLF), we
        // can use a fixed-sized buffer on the stack, avoiding allocations and
        // copying, and stopping bad clients from making us use too much memory.
        let mut request = [0; 1026];
        let mut buf = &mut request[..];
        let mut len = 0;

        // Read until CRLF, end-of-stream, or there's no buffer space left.
        loop {
            let bytes_read = self
                .stream
                .read(buf)
                .await
                .or(Err((59, "Request ended unexpectedly")))?;
            len += bytes_read;
            if request[..len].ends_with(b"\r\n") {
                break;
            } else if bytes_read == 0 {
                return Err((59, "Request ended unexpectedly"));
            }
            buf = &mut request[len..];
        }
        let request =
            std::str::from_utf8(&request[..len - 2]).or(Err((59, "Non-UTF-8 request")))?;

        // log literal request (might be different from or not an actual URL)
        write!(self.log_line, " \"{}\"", request).unwrap();

        let url = Url::parse(request).or(Err((59, "Invalid URL")))?;

        // Validate the URL, host and port.
        if url.scheme() != "gemini" {
            return Err((53, "Unsupported URL scheme"));
        }
        // TODO: Can be simplified by https://github.com/servo/rust-url/pull/651
        if let (Some(Host::Domain(expected)), Some(Host::Domain(host))) =
            (url.host(), &ARGS.hostname)
        {
            if host != expected {
                return Err((53, "Proxy request refused"));
            }
        }
        if let Some(port) = url.port() {
            // Validate that the port in the URL is the same as for the stream this request came in on.
            if port != self.stream.get_ref().0.local_addr().unwrap().port() {
                return Err((53, "proxy request refused"));
            }
        }
        Ok(url)
    }

    /// Send the client the file located at the requested URL.
    async fn send_response(&mut self, url: Url) -> Result {
        let mut path = std::path::PathBuf::from(&ARGS.content_dir);
        if let Some(segments) = url.path_segments() {
            for segment in segments {
                if !ARGS.serve_secret && segment.starts_with('.') {
                    // Do not serve anything that looks like a hidden file.
                    return self
                        .send_header(52, "If I told you, it would not be a secret.")
                        .await;
                }
                path.push(&*percent_decode_str(segment).decode_utf8()?);
            }
        }

        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            if metadata.is_dir() {
                if url.path().ends_with('/') || url.path().is_empty() {
                    // if the path ends with a slash or the path is empty, the links will work the same
                    // without a redirect
                    path.push("index.gmi");
                    if !path.exists() && path.with_file_name(".directory-listing-ok").exists() {
                        path.pop();
                        return self.list_directory(&path).await;
                    }
                } else {
                    // if client is not redirected, links may not work as expected without trailing slash
                    let mut url = url;
                    url.set_path(&format!("{}/", url.path()));
                    return self.send_header(31, url.as_str()).await;
                }
            }
        }

        let data = self.metadata.lock().await.get(&path);

        if let PresetMeta::FullHeader(status, meta) = data {
            self.send_header(status, &meta).await?;
            // do not try to access the file
            return Ok(());
        }

        // Make sure the file opens successfully before sending a success header.
        let mut file = match tokio::fs::File::open(&path).await {
            Ok(file) => file,
            Err(e) => {
                self.send_header(51, "Not found, sorry.").await?;
                Err(e)?
            }
        };

        // Send header.
        let mime = match data {
            // this was already handled before opening the file
            PresetMeta::FullHeader(..) => unreachable!(),
            // treat this as the full MIME type
            PresetMeta::FullMime(mime) => mime.clone(),
            // guess the MIME type and add the parameters
            PresetMeta::Parameters(params) => {
                if path.extension() == Some(OsStr::new("gmi")) {
                    format!("text/gemini{}", params)
                } else {
                    let mime = mime_guess::from_path(&path).first_or_octet_stream();
                    format!("{}{}", mime.essence_str(), params)
                }
            }
        };
        self.send_header(20, &mime).await?;

        // Send body.
        tokio::io::copy(&mut file, &mut self.stream).await?;
        Ok(())
    }

    async fn list_directory(&mut self, path: &Path) -> Result {
        // https://url.spec.whatwg.org/#path-percent-encode-set
        const ENCODE_SET: AsciiSet = CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'#')
            .add(b'<')
            .add(b'>')
            .add(b'?')
            .add(b'`')
            .add(b'{')
            .add(b'}');

        log::info!("Listing directory {:?}", path);
        self.send_header(20, "text/gemini").await?;
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut lines = vec![];
        while let Some(entry) = entries.next_entry().await? {
            let mut name = entry
                .file_name()
                .into_string()
                .or(Err("Non-Unicode filename"))?;
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
            self.stream.write_all(line.as_bytes()).await?;
        }
        Ok(())
    }

    async fn send_header(&mut self, status: u8, meta: &str) -> Result {
        // add response status and response meta
        write!(self.log_line, " {} \"{}\"", status, meta)?;

        self.stream
            .write_all(format!("{} {}\r\n", status, meta).as_bytes())
            .await?;
        Ok(())
    }
}
