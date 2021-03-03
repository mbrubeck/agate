use anyhow::anyhow;
use gemini_fetch::{Header, Page, Status};
use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};
use std::process::{Command, Stdio};
use url::Url;

static BINARY_PATH: &'static str = env!("CARGO_BIN_EXE_agate");

fn addr(port: u16) -> SocketAddr {
    use std::net::{IpAddr, Ipv4Addr};

    (IpAddr::V4(Ipv4Addr::LOCALHOST), port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap()
}

struct Server {
    server: std::process::Child,
    buf: u8,
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
            .spawn()
            .expect("failed to start binary");

        // the first output is when Agate is listening, so we only have to wait
        // until the first byte is output
        let mut buffer = [0; 1];
        server
            .stderr
            .as_mut()
            .unwrap()
            .read_exact(&mut buffer)
            .unwrap();

        Self {
            server,
            buf: buffer[0],
            output: None,
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        // try to stop the server
        if let Some(output) = self.output.clone() {
            return output;
        }

        self.output = Some(match self.server.try_wait() {
            Err(e) => Err(format!("cannot access orchestrated program: {:?}", e)),
            // everything fine, still running as expected, kill it now
            Ok(None) => Ok(self.server.kill().unwrap()),
            Ok(Some(_)) => {
                // forward stderr so we have a chance to understand the problem
                let buffer = std::iter::once(Ok(self.buf))
                    .chain(self.server.stderr.take().unwrap().bytes())
                    .collect::<Result<Vec<u8>, _>>()
                    .unwrap();

                Err(String::from_utf8_lossy(&buffer).into_owned())
            }
        });
        return self.output.clone().unwrap();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if self.output.is_none() && !std::thread::panicking() {
            // a potential error message was not yet handled
            self.stop().unwrap();
        } else if self.output.is_some() {
            // error was already handled, ignore it
            self.stop().unwrap_or(());
        } else {
            // we are panicking and a potential error was not handled
            self.stop().unwrap_or_else(|e| eprintln!("{:?}", e));
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
    let page = get(&[], addr(1965), "gemini://localhost").expect("could not get page");

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
/// - the `--addr` configuration works
/// - MIME media types can be set in the configuration file
fn meta() {
    let page = get(
        &["--addr", "[::]:1966"],
        addr(1966),
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
    let page = get(
        &["--addr", "[::]:1967"],
        addr(1967),
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
    let page = get(
        &["--addr", "[::]:1968"],
        addr(1968),
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
    let page = get(
        &["--addr", "[::]:1969", "-C"],
        addr(1969),
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
    let page = get(
        &["--addr", "[::]:1970"],
        addr(1970),
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
/// - hostname is checked when provided
/// - status for wrong host is "proxy request refused"
fn hostname_check() {
    let page = get(
        &["--addr", "[::]:1971", "--hostname", "example.org"],
        addr(1971),
        "gemini://example.com/",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::ProxyRequestRefused);
}

#[test]
/// - port is checked when hostname is provided
/// - status for wrong port is "proxy request refused"
fn port_check() {
    let page = get(
        &["--addr", "[::]:1972", "--hostname", "example.org"],
        addr(1972),
        "gemini://example.org:1971/",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::ProxyRequestRefused);
}

#[test]
/// - status for paths with hidden segments is "gone" if file does not exist
fn secret_nonexistent() {
    let page = get(
        &["--addr", "[::]:1973"],
        addr(1973),
        "gemini://localhost/.secret",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::Gone);
}

#[test]
/// - status for paths with hidden segments is "gone" if file exists
fn secret_exists() {
    let page = get(
        &["--addr", "[::]:1974"],
        addr(1974),
        "gemini://localhost/.meta",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::Gone);
}

#[test]
/// - secret file served if `--serve-secret` is enabled
fn serve_secret() {
    let page = get(
        &["--addr", "[::]:1975", "--serve-secret"],
        addr(1975),
        "gemini://localhost/.meta",
    )
    .expect("could not get page");

    assert_eq!(page.header.status, Status::Success);
}

#[test]
#[should_panic(expected = "AlertReceived(ProtocolVersion)")]
/// - if TLSv1.3 is selected, does not accept TLSv1.2 connections
///   (lower versions do not have to be tested because rustls does not even
///   support them, making agate incapable of accepting them)
fn explicit_tls_version() {
    use rustls::{ClientSession, ProtocolVersion};
    use std::io::Read;
    use std::net::TcpStream;

    let _server = Server::new(&["--addr", "[::]:1976", "-3"]);

    let mut config = rustls::ClientConfig::new();
    // try to connect using only TLS 1.2
    config.versions = vec![ProtocolVersion::TLSv1_2];

    let dns_name = webpki::DNSNameRef::try_from_ascii_str("localhost").unwrap();
    let mut session = ClientSession::new(&std::sync::Arc::new(config), dns_name);
    let mut tcp = TcpStream::connect(addr(1976)).unwrap();
    let mut tls = rustls::Stream::new(&mut session, &mut tcp);

    let mut buf = [0; 10];
    tls.read(&mut buf).unwrap();
}

mod vhosts {
    use super::*;

    #[test]
    /// - simple vhosts are enabled when multiple hostnames are supplied
    /// - the vhosts access the correct files
    fn example_com() {
        let page = get(
            &[
                "--addr",
                "[::]:1977",
                "--hostname",
                "example.com",
                "--hostname",
                "example.org",
            ],
            addr(1977),
            "gemini://example.com/",
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
        let page = get(
            &[
                "--addr",
                "[::]:1978",
                "--hostname",
                "example.com",
                "--hostname",
                "example.org",
            ],
            addr(1978),
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
    fn cert_missing() {
        let mut server = Server::new(&["--addr", "[::]:1979", "--certs", "cert_missing"]);

        // wait for the server to stop, it should crash
        let _ = server.server.wait();

        assert!(server
            .stop()
            .unwrap_err()
            .to_string()
            .contains("certificate file for fallback is missing"));
    }

    #[test]
    fn key_missing() {
        let mut server = Server::new(&["--addr", "[::]:1980", "--certs", "key_missing"]);

        // wait for the server to stop, it should crash
        let _ = server.server.wait();

        assert!(server
            .stop()
            .unwrap_err()
            .to_string()
            .contains("key file for fallback is missing"));
    }

    #[test]
    fn example_com() {
        use rustls::ClientSession;
        use std::io::{Cursor, Write};
        use std::net::TcpStream;

        let mut server = Server::new(&["--addr", "[::]:1981", "--certs", "multicert"]);

        let mut config = rustls::ClientConfig::new();
        config
            .root_store
            .add_pem_file(&mut Cursor::new(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/multicert/ca_cert.pem"
            ))))
            .unwrap();

        let dns_name = webpki::DNSNameRef::try_from_ascii_str("example.com").unwrap();
        let mut session = ClientSession::new(&std::sync::Arc::new(config), dns_name);
        let mut tcp = TcpStream::connect(addr(1981)).unwrap();
        let mut tls = rustls::Stream::new(&mut session, &mut tcp);

        write!(tls, "gemini://example.com/\r\n").unwrap();

        let mut buf = [0; 10];
        tls.read(&mut buf).unwrap();

        server.stop().unwrap()
    }

    #[test]
    fn example_org() {
        use rustls::ClientSession;
        use std::io::{Cursor, Write};
        use std::net::TcpStream;

        let mut server = Server::new(&["--addr", "[::]:1982", "--certs", "multicert"]);

        let mut config = rustls::ClientConfig::new();
        config
            .root_store
            .add_pem_file(&mut Cursor::new(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/multicert/ca_cert.pem"
            ))))
            .unwrap();

        let dns_name = webpki::DNSNameRef::try_from_ascii_str("example.org").unwrap();
        let mut session = ClientSession::new(&std::sync::Arc::new(config), dns_name);
        let mut tcp = TcpStream::connect(addr(1982)).unwrap();
        let mut tls = rustls::Stream::new(&mut session, &mut tcp);

        write!(tls, "gemini://example.org/\r\n").unwrap();

        let mut buf = [0; 10];
        tls.read(&mut buf).unwrap();

        server.stop().unwrap()
    }
}
