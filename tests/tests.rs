use gemini_fetch::{Header, Page, Status};
use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};
use std::process::{Command, Stdio};
use url::Url;

static BINARY_PATH: &'static str = env!("CARGO_BIN_EXE_agate");

fn addr() -> SocketAddr {
    "[::1]:1965".to_socket_addrs().unwrap().next().unwrap()
}

fn get(args: &[&str], url: &str) -> Result<Page, anyhow::Error> {
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

    // actually perform the request
    let page = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { Page::fetch_from(&Url::parse(url).unwrap(), addr(), None).await });

    // try to stop the server again
    match server.try_wait() {
        Err(e) => panic!("cannot access orchestrated program: {:?}", e),
        // everything fine, still running as expected, kill it now
        Ok(None) => server.kill().unwrap(),
        Ok(Some(_)) => {
            // forward stderr so we have a chance to understand the problem
            let buffer = std::iter::once(Ok(buffer[0]))
                .chain(server.stderr.take().unwrap().bytes())
                .collect::<Result<Vec<u8>, _>>()
                .unwrap();

            eprintln!("{}", String::from_utf8_lossy(&buffer));
            // make the test fail
            panic!("program had crashed");
        }
    }

    page
}

#[test]
fn index_page() {
    let page = get(&[], "gemini://localhost").expect("could not get page");

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

    println!("{:?}", page);
}
