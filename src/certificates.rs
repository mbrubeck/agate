use {
    rustls::{
        internal::pemfile::{certs, pkcs8_private_keys},
        sign::{CertifiedKey, RSASigningKey},
        ResolvesServerCert,
    },
    std::{
        ffi::OsStr,
        fmt::{Display, Formatter},
        fs::File,
        io::BufReader,
        path::Path,
        sync::Arc,
    },
    webpki::DNSNameRef,
};

/// A struct that holds all loaded certificates and the respective domain
/// names.
pub(crate) struct CertStore {
    /// Stores the certificates and the domains they apply to, sorted by domain
    /// names, longest matches first
    certs: Vec<(String, CertifiedKey)>,
}

static CERT_FILE_NAME: &str = "cert.pem";
static KEY_FILE_NAME: &str = "key.rsa";

#[derive(Debug)]
pub enum CertLoadError {
    /// could not access the certificate root directory
    NoReadCertDir,
    /// the specified domain name cannot be processed correctly
    BadDomain(String),
    /// The key file for the given domain does not contain any suitable keys.
    NoKeys(String),
    /// the key file for the specified domain is bad (e.g. does not contain a
    /// key or is invalid)
    BadKey(String),
    /// The certificate file for the specified domain is bad (e.g. invalid)
    /// The second parameter is the error message.
    BadCert(String, String),
    /// the key file for the specified domain is missing (but a certificate
    /// file was present)
    MissingKey(String),
    /// the certificate file for the specified domain is missing (but a key
    /// file was present)
    MissingCert(String),
    /// neither a key file nor a certificate file were present for the given
    /// domain (but a folder was present)
    EmptyDomain(String),
}

impl Display for CertLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoReadCertDir => write!(f, "Could not read from certificate directory."),
            Self::BadDomain(domain) if !domain.is_ascii() => write!(
                f,
                "The domain name {} cannot be processed, it must be punycoded.",
                domain
            ),
            Self::BadDomain(domain) => write!(f, "The domain name {} cannot be processed.", domain),
            Self::NoKeys(domain) => write!(
                f,
                "The key file for {} does not contain any suitable key.",
                domain
            ),
            Self::BadKey(domain) => write!(f, "The key file for {} is malformed.", domain),
            Self::BadCert(domain, e) => {
                write!(f, "The certificate file for {} is malformed: {}", domain, e)
            }
            Self::MissingKey(domain) => write!(f, "The key file for {} is missing.", domain),
            Self::MissingCert(domain) => {
                write!(f, "The certificate file for {} is missing.", domain)
            }
            Self::EmptyDomain(domain) => write!(
                f,
                "A folder for {} exists, but there is no certificate or key file.",
                domain
            ),
        }
    }
}

impl std::error::Error for CertLoadError {}

fn load_domain(certs_dir: &Path, domain: String) -> Result<CertifiedKey, CertLoadError> {
    let mut path = certs_dir.to_path_buf();
    path.push(&domain);
    // load certificate from file
    path.push(CERT_FILE_NAME);
    if !path.is_file() {
        return Err(if !path.with_file_name(KEY_FILE_NAME).is_file() {
            CertLoadError::EmptyDomain(domain)
        } else {
            CertLoadError::MissingCert(domain)
        });
    }

    let cert_chain = match certs(&mut BufReader::new(File::open(&path).unwrap())) {
        Ok(cert) => cert,
        Err(()) => return Err(CertLoadError::BadCert(domain, String::new())),
    };

    // load key from file
    path.set_file_name(KEY_FILE_NAME);
    if !path.is_file() {
        return Err(CertLoadError::MissingKey(domain));
    }
    let key = match pkcs8_private_keys(&mut BufReader::new(File::open(&path).unwrap())) {
        Ok(mut keys) if !keys.is_empty() => keys.remove(0),
        Ok(_) => return Err(CertLoadError::NoKeys(domain)),
        Err(()) => return Err(CertLoadError::BadKey(domain)),
    };

    // transform key to correct format
    let key = match RSASigningKey::new(&key) {
        Ok(key) => key,
        Err(()) => return Err(CertLoadError::BadKey(domain)),
    };
    Ok(CertifiedKey::new(cert_chain, Arc::new(Box::new(key))))
}

impl CertStore {
    /// Load certificates from a certificate directory.
    /// Certificates should be stored in a folder for each hostname, for example
    /// the certificate and key for `example.com` should be in the files
    /// `certs_dir/example.com/{cert.pem,key.rsa}` respectively.
    ///
    /// If there are `cert.pem` and `key.rsa` directly in certs_dir, these will be
    /// loaded as default certificates.
    pub fn load_from(certs_dir: &Path) -> Result<Self, CertLoadError> {
        // load all certificates from directories
        let mut certs = vec![];

        // Try to load fallback certificate and key directly from the top level
        // certificate directory. It will be loaded as the `.` domain.
        match load_domain(certs_dir, ".".to_string()) {
            Err(CertLoadError::EmptyDomain(_)) => { /* there are no fallback keys */ }
            Err(CertLoadError::NoReadCertDir) => unreachable!(),
            Err(CertLoadError::BadDomain(_)) => unreachable!(),
            Err(CertLoadError::NoKeys(_)) => {
                return Err(CertLoadError::NoKeys("fallback".to_string()))
            }
            Err(CertLoadError::BadKey(_)) => {
                return Err(CertLoadError::BadKey("fallback".to_string()))
            }
            Err(CertLoadError::BadCert(_, e)) => {
                return Err(CertLoadError::BadCert("fallback".to_string(), e))
            }
            Err(CertLoadError::MissingKey(_)) => {
                return Err(CertLoadError::MissingKey("fallback".to_string()))
            }
            Err(CertLoadError::MissingCert(_)) => {
                return Err(CertLoadError::MissingCert("fallback".to_string()))
            }
            // For the fallback keys there is no domain name to verify them
            // against, so we can skip that step and only have to do it for the
            // other keys below.
            Ok(key) => certs.push((String::new(), key)),
        }

        for file in certs_dir
            .read_dir()
            .or(Err(CertLoadError::NoReadCertDir))?
            .filter_map(Result::ok)
            .filter(|x| x.path().is_dir())
        {
            let path = file.path();

            // the filename should be the domain name
            let filename = path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap()
                .to_string();

            let dns_name = match DNSNameRef::try_from_ascii_str(&filename) {
                Ok(name) => name,
                Err(_) => return Err(CertLoadError::BadDomain(filename)),
            };

            let key = load_domain(certs_dir, filename.clone())?;
            key.cross_check_end_entity_cert(Some(dns_name))
                .map_err(|e| CertLoadError::BadCert(filename.clone(), e.to_string()))?;

            certs.push((filename, key));
        }

        certs.sort_unstable_by(|(a, _), (b, _)| {
            // Try to match as many domain segments as possible. If one is a
            // substring of the other, the `zip` will only compare the smaller
            // length of either a or b and the for loop will not decide.
            for (a_part, b_part) in a.split('.').rev().zip(b.split('.').rev()) {
                if a_part != b_part {
                    // Here we have to make sure that the empty string will
                    // always be sorted to the end, so we reverse the usual
                    // ordering of str.
                    return a_part.cmp(b_part).reverse();
                }
            }
            // Sort longer domains first.
            a.len().cmp(&b.len()).reverse()
        });

        log::debug!(
            "certs loaded for {:?}",
            certs.iter().map(|t| &t.0).collect::<Vec<_>>()
        );

        Ok(Self { certs })
    }

    /// Checks if a certificate fitting a specific domain has been loaded.
    /// The same rules about using a certificate at the level above apply.
    pub fn has_domain(&self, domain: &str) -> bool {
        self.certs.iter().any(|(s, _)| domain.ends_with(s))
    }
}

impl ResolvesServerCert for CertStore {
    fn resolve(&self, client_hello: rustls::ClientHello<'_>) -> Option<CertifiedKey> {
        if let Some(name) = client_hello.server_name() {
            let name: &str = name.into();
            // The certificate list is sorted so the longest match will always
            // appear first. We have to find the first that is either this
            // domain or a parent domain of the current one.
            self.certs
                .iter()
                .find(|(s, _)| name.ends_with(s))
                // only the key is interesting
                .map(|(_, k)| k)
                .cloned()
        } else {
            // This kind of resolver requires SNI.
            None
        }
    }
}
