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

3. Run the server. You can use the following arguments to specify the locations of the content directory, certificate and key files, IP address and port to listen on, host name to expect in request URLs, and default language code(s) to include in the MIME type for for text/gemini files:

```
agate --content path/to/content/ \
      --key key.rsa \
      --cert cert.pem \
      --addr :::1965 \
      --addr 0.0.0.0:1965 \
      --hostname example.com \
      --lang en-US
```

All of the command-line arguments are optional.  Run `agate --help` to see the default values used when arguments are omitted.

When a client requests the URL `gemini://example.com/foo/bar`, Agate will respond with the file at `path/to/content/foo/bar`. If any segment of the requested path starts with a dot, agate will respond with a status code 52, even if the file does not exist (this behaviour can be disabled with `--serve-secret`). If there is a directory at that path, Agate will look for a file named `index.gmi` inside that directory. If there is no such file, but a file named `.directory-listing-ok` exists inside that directory, a basic directory listing is displayed. Files or directories whose name starts with a dot (e.g. the `.directory-listing-ok` file itself) are omitted from the list.

Agate will look for a file called `.lang` in the same directory as the file currently being served. If this file exists and has an entry for the current file, the respective data will be used to formulate the response header.
The lines of the file should have this format:

```text
<filename>:<metadata>
```

Where `<filename>` is just a filename (not a path) of a file in the same directory, and `<metadata>` is the metadata to be stored.
Lines that start with optional whitespace and `#` are ignored, as are lines that do not fit the above basic format.
Both parts are stripped of any leading and/or trailing whitespace.

[Gemini]: https://gemini.circumlunar.space/
[Rust]: https://www.rust-lang.org/
[home]: gemini://gem.limpet.net/agate/
[rustup]: https://www.rust-lang.org/tools/install
[source]: https://github.com/mbrubeck/agate
[crates.io]: https://crates.io/crates/agate
