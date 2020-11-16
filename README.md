# Agate

## Simple Gemini server for static files

Agate is a server for the [Gemini] network protocol, built with the [Rust] programming language. Agate has very few features, and can only serve static files. It uses async I/O, and should be quite efficient even when running on low-end hardware and serving many concurrent requests.

## Learn more

* Home page: [gemini://gem.limpet.net/agate/][home]
* [Cargo package][crates.io]
* [Source code][source]

## Installation and setup

1. Download and unpack the [pre-compiled binary](https://github.com/mbrubeck/agate/releases).

   Or, if you have the Rust toolchain installed, run `cargo install agate` to
   install agate from crates.io.

   Or download the source code and run `cargo build --release` inside the
   source repository, then find the binary at `target/release/agate`.

2. Generate a self-signed TLS certificate and private key.  For example, if you have OpenSSL 1.1 installed, you can use a command like the following.  (Replace the hostname with the address of your Gemini server.)

```
openssl req -x509 -newkey rsa:4096 -keyout key.rsa -out cert.pem \
    -days 3650 -nodes -subj "/CN=example.com"
```

3. Run the server. The command line arguments are `agate <addr:port> <content_dir> <cert_file> <key_file> [<domain>]`.  For example, to listen on the standard Gemini port (1965) on all interfaces:

```
agate 0.0.0.0:1965 path/to/content/ cert.pem key.rsa
```

Agate will check that the port part of the requested URL matches the port specified in the 1st argument.
If `<domain>` is specified, agate will also check that the host part of the requested URL matches this domain.

When a client requests the URL `gemini://example.com/foo/bar`, Agate will respond with the file at `path/to/content/foo/bar`.  If there is a directory at that path, Agate will look for a file named `index.gmi` inside that directory.

Optionally, set a log level via the `AGATE_LOG` environment variable. Logging is powered by the [env_logger crate](https://crates.io/crates/env_logger):

```
AGATE_LOG=info 0.0.0.0:1965 path/to.content/ cert.pem key.rsa
```

[Gemini]: https://gemini.circumlunar.space/
[Rust]: https://www.rust-lang.org/
[home]: gemini://gem.limpet.net/agate/
[rustup]: https://www.rust-lang.org/tools/install
[source]: https://github.com/mbrubeck/agate
[crates.io]: https://crates.io/crates/agate
