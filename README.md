# Agate

## Simple Gemini server for static files

Agate is a server for the [Gemini] network protocol, built with the [Rust] programming language. Agate has very few features, and can only serve static files. It uses async I/O, and should be quite efficient even when running on low-end hardware and serving many concurrent requests.

## Learn more

* Home page: [gemini://gem.limpet.net/agate/][home]
* [Cargo package][crates.io]
* [Source code][source]

## Installation and setup

1. [Install the Rust toolchain][rustup].

2. Run `cargo install agate` to install agate from crates.io, or clone the [source], run `cargo build --release`, and then copy the compiled binary from `target/release/agate` to any location you want.  (You can also use `cargo run --release <args>` to run Agate from within the source directory.)

3. Generate a self-signed TLS certificate and private key.  For example, if you have OpenSSL 1.1 installed, you can use a command like the following.  (Replace the hostname with the address of your Gemini server.)

```
openssl req -x509 -newkey rsa:4096 -keyout key.rsa -out cert.pem \
    -days 3650 -nodes -subj "/CN=example.com"
```

4. Run the server. The command line arguments are `agate <address:port> <content_dir> <cert_file> <key_file>`.  For example, to listen on the standard Gemini port (1965) on all network interfaces:

```
agate 0.0.0.0:1965 path/to/content/ cert.pem key.rsa
```

When a client requests the URL `gemini://example.com/foo/bar`, Agate will respond with the file at `path/to/content/foo/bar`.  If there is a directory at that path, Agate will look for a file named `index.gemini` inside that directory.  Currently, Agate sends all responses with the `text/gemini` MIME type.  (Support for other MIME types may be added in the future.)

[Gemini]: https://gemini.circumlunar.space/
[Rust]: https://www.rust-lang.org/
[home]: gemini://gem.limpet.net/agate/
[rustup]: https://www.rust-lang.org/tools/install
[source]: https://github.com/mbrubeck/agate
[crates.io]: https://crates.io/crates/agate
