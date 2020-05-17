use {
    agate::{Result},
    rustls::{NoClientAuth, ServerConfig, ServerSession},
    std::{
        net::TcpListener,
        sync::Arc,
    },
};

fn main() -> Result {
    let tls_config = Arc::new(ServerConfig::new(NoClientAuth::new()));
    // TODO: configure a certificate

    let listener = TcpListener::bind("0.0.0.0:1965")?;
    for stream in listener.incoming() {
        let session = ServerSession::new(&tls_config);
    }
    Ok(())
}
