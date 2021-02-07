# Agate

## Simple Gemini server for static files

Agate is a server for the [Gemini] network protocol, built with the [Rust] programming language. Agate has very few features, and can only serve static files. It uses async I/O, and should be quite efficient even when running on low-end hardware and serving many concurrent requests.

Since Agate by default uses port 1965, you should be able to run other servers (like e.g. Apache or nginx) on the same device.

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

***
You can use the install script in the `tools` directory for the remaining steps if there is one for your system.  
If there is none, please consider contributing one to make it easier for less tech-savvy users!
***

2. Generate a self-signed TLS certificate and private key.  For example, if you have OpenSSL 1.1 installed, you can use a command like the following.  (Replace the hostname `example.com` with the address of your Gemini server.)

```
openssl req -x509 -newkey rsa:4096 -keyout key.rsa -out cert.pem \
    -days 3650 -nodes -subj "/CN=example.com"
```

3. Run the server. You can use the following arguments to specify the locations of the content directory, certificate and key files, IP address and port to listen on, host name to expect in request URLs, and default language code(s) to include in the MIME type for for text/gemini files: (Again replace the hostname `example.com` with the address of your Gemini server.)

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

When a client requests the URL `gemini://example.com/foo/bar`, Agate will respond with the file at `path/to/content/foo/bar`. If any segment of the requested path starts with a dot, agate will respond with a status code 52, wether the file exists or not (this behaviour can be disabled with `--serve-secret`). If there is a directory at that path, Agate will look for a file named `index.gmi` inside that directory.

## Configuration

### Directory listing

You can enable a basic directory listing for a directory by putting a file called `.directory-listing-ok` in that directory. This does not have an effect on subdirectories.
The directory listing will hide files and directories whose name starts with a dot (e.g. the `.directory-listing-ok` file itself or also the `.meta` configuration file).

A file called `index.gmi` will always take precedence over a directory listing.

### Meta-Presets

You can put a file called `.meta` in a directory that stores some metadata about these files which Agate will use when serving these files. The file should be UTF-8 encoded. Like the `.directory-listing-ok` file, this file does not have an effect on subdirectories.
Lines starting with a `#` are comments and will be ignored like empty lines. All other lines must start with a file name (not a path), followed by a colon and then the metadata.

The metadata can take one of four possible forms:
1. empty  
    Agate will not send a default language parameter, even if it was specified on the command line.
2. starting with a semicolon followed by MIME parameters  
    Agate will append the specified string onto the MIME type, if the file is found.
3. starting with a gemini status code (i.e. a digit 1-6 inclusive followed by another digit) and a space  
    Agate will send the metadata wether the file exists or not. The file will not be sent or accessed.
4. a MIME type, may include parameters  
    Agate will use this MIME type instead of what it would guess, if the file is found.
    The default language parameter will not be used, even if it was specified on the command line.

If a line violates the format or looks like case 3, but is incorrect, it might be ignored. You should check your logs. Please know that this configuration file is first read when a file from the respective directory is accessed. So no log messages after startup does not mean the `.meta` file is okay.

Such a configuration file might look like this:
```text
# This line will be ignored.
index.gmi:;lang=en-UK
LICENSE:text/plain;charset=UTF-8
gone.gmi:52 This file is no longer here, sorry.
```

### Logging Verbosity

Agate uses the `env_logger` crate and allows you to set the logging verbosity by setting the default `RUST_LOG` environment variable. For more information, please see the [documentation of `env_logger`].

[Gemini]: https://gemini.circumlunar.space/
[Rust]: https://www.rust-lang.org/
[home]: gemini://gem.limpet.net/agate/
[rustup]: https://www.rust-lang.org/tools/install
[source]: https://github.com/mbrubeck/agate
[crates.io]: https://crates.io/crates/agate
[documentation of `env_logger`]: https://docs.rs/env_logger/0.8
