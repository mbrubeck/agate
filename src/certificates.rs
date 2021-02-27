use {
    rustls::{
        internal::pemfile::{certs, pkcs8_private_keys},
        sign::{CertifiedKey, RSASigningKey},
        ResolvesServerCert,
    },
    std::{fs::File, io::BufReader, path::PathBuf, sync::Arc},
    webpki::DNSNameRef,
};

/// A struct that holds all loaded certificates and the respective domain
/// names.
pub(crate) struct CertStore {
    // use a Vec of pairs instead of a HashMap because order matters
    certs: Vec<(String, CertifiedKey)>,
}

static CERT_FILE_NAME: &str = "cert.pem";
static KEY_FILE_NAME: &str = "key.rsa";

impl CertStore {
    /// Load certificates from a certificate directory.
    /// Certificates should be stored in a folder for each hostname, for example
    /// the certificate and key for `example.com` should be in the files
    /// `certs_dir/example.com/{cert.pem,key.rsa}` respectively.
    pub fn load_from(certs_dir: PathBuf) -> Result<Self, String> {
        // load all certificates from directories
        let mut certs = certs_dir
            .read_dir()
            .expect("could not read from certificate directory")
            .filter_map(Result::ok)
            .filter_map(|entry| {
                if !entry.metadata().map_or(false, |data| data.is_dir()) {
                    None
                } else if !entry.file_name().to_str().map_or(false, |s| s.is_ascii()) {
                    Some(Err(
                        "domain for certificate is not US-ASCII, must be punycoded".to_string(),
                    ))
                } else {
                    let filename = entry.file_name();
                    let dns_name = match DNSNameRef::try_from_ascii_str(filename.to_str().unwrap())
                    {
                        Ok(name) => name,
                        Err(e) => return Some(Err(e.to_string())),
                    };

                    // load certificate from file
                    let mut path = entry.path();
                    path.push(CERT_FILE_NAME);
                    if !path.is_file() {
                        return Some(Err(format!("expected certificate {:?}", path)));
                    }
                    let cert_chain = match certs(&mut BufReader::new(File::open(&path).unwrap())) {
                        Ok(cert) => cert,
                        Err(_) => return Some(Err("bad cert file".to_string())),
                    };

                    // load key from file
                    path.set_file_name(KEY_FILE_NAME);
                    if !path.is_file() {
                        return Some(Err(format!("expected key {:?}", path)));
                    }
                    let key =
                        match pkcs8_private_keys(&mut BufReader::new(File::open(&path).unwrap())) {
                            Ok(mut keys) if !keys.is_empty() => keys.remove(0),
                            Ok(_) => return Some(Err(format!("key file empty {:?}", path))),
                            Err(_) => return Some(Err("bad key file".to_string())),
                        };

                    // transform key to correct format
                    let key = match RSASigningKey::new(&key) {
                        Ok(key) => key,
                        Err(_) => return Some(Err("bad key".to_string())),
                    };
                    let key = CertifiedKey::new(cert_chain, Arc::new(Box::new(key)));
                    if let Err(e) = key.cross_check_end_entity_cert(Some(dns_name)) {
                        return Some(Err(e.to_string()));
                    }
                    Some(Ok((entry.file_name().to_str().unwrap().to_string(), key)))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        certs.sort_unstable_by(|(a, _), (b, _)| {
            // try to match as many as possible. If one is a substring of the other,
            // the `zip` will make them look equal and make the length decide.
            for (a_part, b_part) in a.split('.').rev().zip(b.split('.').rev()) {
                if a_part != b_part {
                    return a_part.cmp(b_part);
                }
            }
            // longer domains first
            a.len().cmp(&b.len()).reverse()
        });
        Ok(Self { certs })
    }
}

impl ResolvesServerCert for CertStore {
    fn resolve(&self, client_hello: rustls::ClientHello<'_>) -> Option<CertifiedKey> {
        if let Some(name) = client_hello.server_name() {
            let name: &str = name.into();
            self.certs
                .iter()
                .find(|(s, _)| name.ends_with(s))
                .map(|(_, k)| k)
                .cloned()
        } else {
            // This kind of resolver requires SNI
            None
        }
    }
}
