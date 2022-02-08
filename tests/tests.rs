use anyhow::anyhow;
use gemini_fetch::{Header, Page, Status};
use std::convert::TryInto;
use std::io::{BufRead, BufReader, Read};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use url::Url;

static BINARY_PATH: &str = env!("CARGO_BIN_EXE_agate");

static DEFAULT_PORT: u16 = 1965;
// this is our atomic port that increments for each test that needs one
// doing it this way avoids port collisions from manually setting ports
static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT + 1);

// this is only used by index_page as it uses the default port
// use gen_addr_port
fn addr(port: u16) -> SocketAddr {
    use std::net::{IpAddr, Ipv4Addr};

    (IpAddr::V4(Ipv4Addr::LOCALHOST), port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap()
}

fn gen_addr_port() -> (SocketAddr, u16) {
    let port = PORT.fetch_add(1, Ordering::SeqCst);
    (addr(port), port)
}

fn gen_port() -> u16 {
    let (_, port) = gen_addr_port();
    port
}

struct Server {
    server: std::process::Child,
    // is set when output is collected by stop()
    output: Option<Result<(), String>>,
}

impl Server {
    pub fn new(args: &[&str]) -> Self {
        // start the server
        let mut server = Command::new(BINARY_PATH)
            .stderr(Stdio::piped())
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data"))
            .args(args)
            .env("RUST_LOG", "debug")
            .spawn()
            .expect("failed to start binary");

        // We can be sure that agate is listening because it logs a message saying so.
        let mut reader = BufReader::new(server.stderr.as_mut().unwrap());
        let mut buffer = String::new();
        while matches!(reader.read_line(&mut buffer), Ok(i) if i>0) {
            print!("log: {}", buffer);
            if buffer.contains("Started") {
                break;
            }

            buffer.clear();
        }

        if matches!(server.try_wait(), Ok(Some(_)) | Err(_)) {
            panic!("Server did not start properly");
        }

        Self {
            server,
            output: None,
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        // try to stop the server
        if let Some(output) = self.output.as_ref() {
            return output.clone();
        }

        self.output = Some(match self.server.try_wait() {
            Err(e) => Err(format!("cannot access orchestrated program: {:?}", e)),
            Ok(None) => {
                // everything fine, still running as expected, kill it now
                self.server.kill().unwrap();

                let mut reader = BufReader::new(self.server.stderr.as_mut().unwrap());
                let mut buffer = String::new();
                while matches!(reader.read_line(&mut buffer), Ok(i) if i>0) {
                    print!("log: {}", buffer);
                    if buffer.contains("Listening") {
                        break;
                    }
                }
                Ok(())
            }
            Ok(Some(_)) => {
                let mut reader = BufReader::new(self.server.stderr.as_mut().unwrap());
                let mut buffer = String::new();
                while matches!(reader.read_line(&mut buffer), Ok(i) if i>0) {
                    print!("log: {}", buffer);
                    if buffer.contains("Listening") {
                        break;
                    }
                }
                Err(buffer)
            }
        });
        self.output.clone().unwrap()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if self.output.is_none() && !std::thread::panicking() {
            // a potential error message was not yet handled
            self.stop().unwrap();
        } else if self.output.is_some() {
            // server was already stopped
        } else {
            // we are panicking and a potential error was not handled
            self.stop().unwrap_or_else(|e| eprintln!("{}", e));
        }
    }
}

fn get(args: &[&str], addr: SocketAddr, url: &str) -> Result<Page, anyhow::Error> {
    let mut server = Server::new(args);

    // actually perform the request
    let page = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { Page::fetch_from(&Url::parse(url).unwrap(), addr, None).await });

    server.stop().map_err(|e| anyhow!(e)).and(page)
}

#[test]
/// - serves index page for a directory
/// - serves the correct content
fn index_page() {
    let page = get(&[], addr(DEFAULT_PORT), "gemini://localhost").expect("could not get page");

    assert_eq!(
        page.header,
        Header {
            status: Status::Success,
            meta: "text/gemini".to_string(),
        }
    );

    assert_eq!(
        page.body,
        Some(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/content/index.gmi"
            ))
            .unwrap()
        )
    );
}

#[test]
/// - symlinked files are followed correctly
fn symlink_page() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port)],
        addr,
        "gemini://localhost/symlink.gmi",
    )
    .expect("could not get page");

    assert_eq!(
        page.header,
        Header {
            status: Status::Success,
            meta: "text/gemini".to_string(),
        }
    );

    assert_eq!(
        page.body,
        Some(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/content/index.gmi"
            ))
            .unwrap()
        )
    );
}

#[test]
/// - symlinked directories are followed correctly
fn symlink_directory() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port)],
        addr,
        "gemini://localhost/symlinked_dir/file.gmi",
    )
    .expect("could not get page");

    assert_eq!(
        page.header,
        Header {
            status: Status::Success,
            meta: "text/gemini".to_string(),
        }
    );

    assert_eq!(
        page.body,
        Some(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/symlinked_dir/file.gmi"
            ))
            .unwrap()
        )
    );
}

#[test]
/// - the `--addr` configuration works
/// - MIME media types can be set in the configuration file
fn meta() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port)],
        addr,
        "gemini://localhost/test",
    )
    .expect("could not get page");

    assert_eq!(
        page.header,
        Header {
            status: Status::Success,
            meta: "text/html".to_string(),
        }
    );
}

#[test]
/// - MIME type is correctly guessed for `.gmi` files
/// - MIME media type parameters can be set in the configuration file
fn meta_param() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port)],
        addr,
        "gemini://localhost/test.gmi",
    )
    .expect("could not get page");

    assert_eq!(
        page.header,
        Header {
            status: Status::Success,
            meta: "text/gemini;lang=en ;charset=us-ascii".to_string(),
        }
    );
}

#[test]
/// - globs in the configuration file work correctly
/// - distributed configuration file is used when `-C` flag not used
fn glob() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port)],
        addr,
        "gemini://localhost/testdir/a.nl.gmi",
    )
    .expect("could not get page");

    assert_eq!(
        page.header,
        Header {
            status: Status::Success,
            meta: "text/plain;lang=nl".to_string(),
        }
    );
}

#[test]
/// - double globs (i.e. `**`) work correctly in the configuration file
/// - central configuration file is used when `-C` flag is used
fn doubleglob() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port), "-C"],
        addr,
        "gemini://localhost/testdir/a.nl.gmi",
    )
    .expect("could not get page");

    assert_eq!(
        page.header,
        Header {
            status: Status::Success,
            meta: "text/gemini;lang=nl".to_string(),
        }
    );
}

#[test]
/// - full header lines can be set in the configuration file
fn full_header_preset() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port)],
        addr,
        "gemini://localhost/gone.txt",
    )
    .expect("could not get page");

    assert_eq!(
        page.header,
        Header {
            status: Status::Gone,
            meta: "This file is no longer available.".to_string(),
        }
    );
}

#[test]
/// - URLS with fragments are rejected
fn fragment() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &[
            "--addr",
            &format!("[::]:{}", port),
            "--hostname",
            "example.com",
        ],
        addr,
        "gemini://example.com/#fragment",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::BadRequest);
}

#[test]
/// - URLS with username are rejected
fn username() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &[
            "--addr",
            &format!("[::]:{}", port),
            "--hostname",
            "example.com",
        ],
        addr,
        "gemini://user@example.com/",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::BadRequest);
}

#[test]
/// - URLS with invalid hostnames are rejected
fn percent_encode() {
    use rustls::{Certificate, ClientConnection, RootCertStore};
    use std::io::Write;
    use std::net::TcpStream;

    let (addr, port) = gen_addr_port();
    // let page = get(
    //     &["--addr", &format!("[::]:{}", port), "--hostname", "example.com"],
    //     addr,
    let mut server = Server::new(&["--addr", &format!("[::]:{}", port), "--certs", "multicert"]);

    let mut certs = RootCertStore::empty();
    certs
        .add(&Certificate(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/multicert/example.com/cert.der"
            ))
            .to_vec(),
        ))
        .unwrap();
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(certs)
        .with_no_client_auth();

    let mut session = ClientConnection::new(
        std::sync::Arc::new(config),
        "example.com".try_into().unwrap(),
    )
    .unwrap();
    let mut tcp = TcpStream::connect(addr).unwrap();
    let mut tls = rustls::Stream::new(&mut session, &mut tcp);

    write!(tls, "gemini://%/\r\n").unwrap();

    let mut buf = [0; 10];
    let _ = tls.read(&mut buf);

    assert_eq!(&buf[0..2], b"59");

    server.stop().unwrap();
}

#[test]
/// - URLS with password are rejected
fn password() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &[
            "--addr",
            &format!("[::]:{}", port),
            "--hostname",
            "example.com",
        ],
        addr,
        "gemini://:secret@example.com/",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::BadRequest);
}

#[test]
/// - hostname is checked when provided
/// - status for wrong host is "proxy request refused"
fn hostname_check() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &[
            "--addr",
            &format!("[::]:{}", port),
            "--hostname",
            "example.org",
        ],
        addr,
        "gemini://example.com/",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::ProxyRequestRefused);
}

#[test]
/// - port is checked when hostname is provided
/// - status for wrong port is "proxy request refused"
fn port_check() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &[
            "--addr",
            &format!("[::]:{}", port),
            "--hostname",
            "example.org",
        ],
        addr,
        &format!("gemini://example.org:{}/", 0000),
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::ProxyRequestRefused);
}

#[test]
/// - port is not checked if the skip option is passed.
fn port_check_skipped() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &[
            "--addr",
            &format!("[::]:{}", port),
            "--hostname",
            "example.org",
            "--skip-port-check",
        ],
        addr,
        &format!("gemini://example.org:{}/", port),
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::Success);
}

#[test]
/// - status for paths with hidden segments is "gone" if file does not exist
fn secret_nonexistent() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port)],
        addr,
        "gemini://localhost/.secret",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::Gone);
}

#[test]
/// - status for paths with hidden segments is "gone" if file exists
fn secret_exists() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port)],
        addr,
        "gemini://localhost/.meta",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::Gone);
}

#[test]
/// - secret file served if `--serve-secret` is enabled
fn serve_secret() {
    let (addr, port) = gen_addr_port();
    let page = get(
        &["--addr", &format!("[::]:{}", port), "--serve-secret"],
        addr,
        "gemini://localhost/.meta",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::Success);
}

#[test]
/// - directory traversal attacks using percent-encoded path separators
///   fail (this addresses a previous vulnerability)
fn directory_traversal_regression() {
    let base = Url::parse("gemini://localhost/").unwrap();

    let mut absolute = base.clone();
    absolute
        .path_segments_mut()
        .unwrap()
        .push(env!("CARGO_MANIFEST_DIR")) // separators will be percent-encoded
        .push("tests")
        .push("data")
        .push("directory_traversal.gmi");

    let mut relative_escape_path = PathBuf::new();
    relative_escape_path.push("testdir");
    relative_escape_path.push("..");
    relative_escape_path.push("..");
    let mut relative = base;
    relative
        .path_segments_mut()
        .unwrap()
        .push(relative_escape_path.to_str().unwrap()) // separators will be percent-encoded
        .push("directory_traversal.gmi");

    let (addr, port) = gen_addr_port();
    let urls = [absolute, relative];
    for url in urls.iter() {
        let page = get(&["--addr", &format!("[::]:{}", port)], addr, url.as_str())
            .expect("could not get page");
        assert_eq!(page.header.status, Status::NotFound);
    }
}

#[test]
/// - if TLSv1.3 is selected, does not accept TLSv1.2 connections
///   (lower versions do not have to be tested because rustls does not even
///   support them, making agate incapable of accepting them)
fn explicit_tls_version() {
    use rustls::{ClientConnection, Error, RootCertStore};
    use std::io::Read;
    use std::net::TcpStream;

    let (addr, port) = gen_addr_port();
    let _server = Server::new(&["--addr", &format!("[::]:{}", port), "-3"]);

    let config = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        // try to connect using only TLS 1.2
        .with_protocol_versions(&[&rustls::version::TLS12])
        .unwrap()
        .with_root_certificates(RootCertStore::empty())
        .with_no_client_auth();

    let mut session =
        ClientConnection::new(std::sync::Arc::new(config), "localhost".try_into().unwrap())
            .unwrap();
    let mut tcp = TcpStream::connect(addr).unwrap();
    let mut tls = rustls::Stream::new(&mut session, &mut tcp);

    let mut buf = [0; 10];
    assert_eq!(
        *tls.read(&mut buf)
            .unwrap_err()
            .into_inner()
            .unwrap()
            .downcast::<Error>()
            .unwrap(),
        Error::AlertReceived(rustls::internal::msgs::enums::AlertDescription::ProtocolVersion)
    )
}

mod vhosts {
    use super::*;

    #[test]
    /// - simple vhosts are enabled when multiple hostnames are supplied
    /// - the vhosts access the correct files
    /// - the hostname comparison is case insensitive
    /// - the hostname is converted to lower case to access certificates
    fn example_com() {
        let (addr, port) = gen_addr_port();
        let page = get(
            &[
                "--addr",
                &format!("[::]:{}", port),
                "--hostname",
                "example.com",
                "--hostname",
                "example.org",
            ],
            addr,
            "gemini://Example.com/",
        )
        .expect("could not get page");

        assert_eq!(page.header.status, Status::Success);

        assert_eq!(
            page.body,
            Some(
                std::fs::read_to_string(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/data/content/example.com/index.gmi"
                ))
                .unwrap()
            )
        );
    }

    #[test]
    /// - the vhosts access the correct files
    fn example_org() {
        let (addr, port) = gen_addr_port();
        let page = get(
            &[
                "--addr",
                &format!("[::]:{}", port),
                "--hostname",
                "example.com",
                "--hostname",
                "example.org",
            ],
            addr,
            "gemini://example.org/",
        )
        .expect("could not get page");

        assert_eq!(page.header.status, Status::Success);

        assert_eq!(
            page.body,
            Some(
                std::fs::read_to_string(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/data/content/example.org/index.gmi"
                ))
                .unwrap()
            )
        );
    }
}

mod multicert {
    use super::*;

    #[test]
    #[should_panic]
    fn cert_missing() {
        let port = gen_port();
        let mut server = Server::new(&[
            "--addr",
            &format!("[::]:{}", port),
            "--certs",
            "cert_missing",
        ]);

        // wait for the server to stop, it should crash
        let _ = server.server.wait();
    }

    #[test]
    #[should_panic]
    fn key_missing() {
        let port = gen_port();
        let mut server = Server::new(&[
            "--addr",
            &format!("[::]:{}", port),
            "--certs",
            "key_missing",
        ]);

        // wait for the server to stop, it should crash
        let _ = server.server.wait();
    }

    #[test]
    fn example_com() {
        use rustls::{Certificate, ClientConnection, RootCertStore};
        use std::io::Write;
        use std::net::TcpStream;

        let (addr, port) = gen_addr_port();
        let mut server =
            Server::new(&["--addr", &format!("[::]:{}", port), "--certs", "multicert"]);

        let mut certs = RootCertStore::empty();
        certs
            .add(&Certificate(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/data/multicert/example.com/cert.der"
                ))
                .to_vec(),
            ))
            .unwrap();
        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(certs)
            .with_no_client_auth();

        let mut session = ClientConnection::new(
            std::sync::Arc::new(config),
            "example.com".try_into().unwrap(),
        )
        .unwrap();
        let mut tcp = TcpStream::connect(addr).unwrap();
        let mut tls = rustls::Stream::new(&mut session, &mut tcp);

        write!(tls, "gemini://example.com/\r\n").unwrap();

        let mut buf = [0; 10];
        let _ = tls.read(&mut buf);

        server.stop().unwrap();
    }

    #[test]
    fn example_org() {
        use rustls::{Certificate, ClientConnection, RootCertStore};
        use std::io::Write;
        use std::net::TcpStream;

        let (addr, port) = gen_addr_port();
        let mut server =
            Server::new(&["--addr", &format!("[::]:{}", port), "--certs", "multicert"]);

        let mut certs = RootCertStore::empty();
        certs
            .add(&Certificate(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/data/multicert/example.org/cert.der"
                ))
                .to_vec(),
            ))
            .unwrap();
        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(certs)
            .with_no_client_auth();

        let mut session = ClientConnection::new(
            std::sync::Arc::new(config),
            "example.org".try_into().unwrap(),
        )
        .unwrap();
        let mut tcp = TcpStream::connect(addr).unwrap();
        let mut tls = rustls::Stream::new(&mut session, &mut tcp);

        write!(tls, "gemini://example.org/\r\n").unwrap();

        let mut buf = [0; 10];
        let _ = tls.read(&mut buf);

        server.stop().unwrap();
    }
}

mod directory_listing {
    use super::*;

    #[test]
    /// - shows directory listing when enabled
    /// - shows directory listing preamble correctly
    /// - encodes link URLs correctly
    fn with_preamble() {
        let (addr, port) = gen_addr_port();
        let page = get(
            &[
                "--addr",
                &format!("[::]:{}", port),
                "--content",
                "dirlist-preamble",
            ],
            addr,
            "gemini://localhost/",
        )
        .expect("could not get page");

        assert_eq!(
            page.header,
            Header {
                status: Status::Success,
                meta: "text/gemini".into(),
            }
        );

        assert_eq!(
            page.body,
            Some("This is a directory listing\n=> a\n=> b\n=> wao%20spaces wao spaces\n".into())
        );
    }

    #[test]
    fn empty_preamble() {
        let (addr, port) = gen_addr_port();
        let page = get(
            &["--addr", &format!("[::]:{}", port), "--content", "dirlist"],
            addr,
            "gemini://localhost/",
        )
        .expect("could not get page");

        assert_eq!(
            page.header,
            Header {
                status: Status::Success,
                meta: "text/gemini".into(),
            }
        );

        assert_eq!(page.body, Some("=> a\n=> b\n".into()),);
    }
}
