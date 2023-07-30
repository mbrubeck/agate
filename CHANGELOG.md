# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
Thank you to Jan Stępień for contributing to this release.

### Fixed
* set permissions for generated key files so only owner can read them

## [3.3.0] - 2023-03-18
Thank you to @equalsraf, @michaelnordmeyer and @wanderer1988 for contributing to this release.

### Added
* listening on unix sockets (#244)

### Fixed
* updated dependencies
* misstyped email address in section on how to report security vulnerabilities (#239)
* wrong language code in README (#189)

## [3.2.4] - 2022-05-18
Thank you to @06kellyjac, @albertlarsan68 and @kahays for contributing to this release.

### Fixed
* removed port collisions in tests, for the last time (#143)
* fixed Dockerfile startup command (#169)
* upated dependencies

## [3.2.3] - 2022-02-04
Thank you to T. Spivey for contributing to this release.

### Fixed
* improper IRIs are handled instead of crashing (bug reported via email)
* updated dependencies

## [3.2.2] - 2022-01-25
Thank you to @Suzie97 for contributing to this release.

### Added
* CI build for `aarch64-apple-darwin` target (#137)

### Fixed
* updated dependencies

## [3.2.1] - 2021-12-02
Thank you to @MatthiasPortzel for contributing to this release.

### Fixed
* host name comparisons are now case insensitive (#115)
* made automatic certificate configuration more prominent in the README
* updated dependencies

## [3.2.0] - 2021-11-15
Thank you to @balazsbtond and @joseph-marques for contributing to this release.

### Added
* you can add header text to a directory listing. See the updated readme for details. (#98)

### Fixed
* updated dependencies
* error pages also send close_notify (#100)

## [3.1.3] - 2021-10-25
Thank you to @FoxKyong for contributing to this release.

### Fixed
* the fix for dual stack listening from 3.1.2 was executed asynchronously and would thus
  sometimes fail. starting the listeners on different socket addresses is now synchronous
  (#79)

## [3.1.2] - 2021-10-15
Thank you to @etam for contributing to this release.

### Fixed
* when starting up on a system that automatically listens in dual stack mode (e.g. some
  linux distributions seem to do this), detect a second unspecified address to not cause
  the "address in use" error with the default listening addresses (#79)
* updated a dependency

## [3.1.1] - 2021-10-14
Thank you to @jgarte and @alvaro-cuesta for contributing to this release.

### Added
* running Agate using GNU Guix (#62)

### Fixed
* actually bind to multiple IP addresses. Despite the documentation saying so,
  Agate would only bind to the first address that did not result in an error. (#63)
* updated dependencies

## [3.1.0] - 2021-06-08
Thank you to Matthew Ingwersen and Oliver Simmons (@GoodClover) for contributing to this release.

### Added
* tests for symlink files (#60)
  Symlinks were already working before.

### Fixed
* A path traversal security issue was closed: Percent-encoded slashes were misunderstood.

### Changed
* Visiting a directory without `index.gmi` and `.directory-listing-ok` now returns a different error message to better show the cause of the error.
  To retain the current behaviour of showing a `51 Not found, sorry.` error, add the following line to the respective directories' `.meta` file:
```
index.gmi: 51 Not found, sorry.
```

## [3.0.3] - 2021-05-24
Thank you to @06kellyjac, @cpnfeeny, @lifelike, @skittlesvampir and @steko for contributing to this release.

### Added
* Dockerfile for compiling Agate from source (#52, #53, #56, #57)

### Fixed
* If the remote IP address can not be fetched, log an error instead of panicking.
  The previous handling could be exploited as a DoS attack vector. (#59)
* Two tests were running on the same port, causing them to fail nondeterministically. (#51)
* Rephrased the changelog for 3.0.0 on continuing to use older certificates. (#55)
* Updated dependencies.

## [3.0.2] - 2021-04-08
Thank you to @kvibber, @lifelike and @pasdechance for contributing to this release.

### Changed
* The new specfication changes are obeyed regarding rejecting request URLs that contain fragments or userinfo parts.
* The default signature algorithm used for generating certificates has been changed to ECDSA since there were multiple complaints about Ed25519.

## [3.0.1] - 2021-03-28
Thank you to @MidAutumnMoon and @steko for contributing to this release.

### Added
* Installation instructions for Arch Linux from Arch User Repositories. (#47)

### Fixed
* The certificate file extensions in the README example. (#45)
* The certificate directory is automatically created if it does not exist. (#44)

## [3.0.0] - 2021-03-27
Thank you to @ddevault for contributing to this release.

### Added
* Support for ECDSA and Ed25519 keys.
* Agate now generates certificates and keys for each `--hostname` that is specified but no matching files exist. (#41)

### Changed
* The ability to specify a certificate and key with `--cert` and `--key` respectively has been replaced with the `--certs` option. (#40)
  Certificates are now stored in a special directory. To migrate to this version, the keys should be stored in the `.certificates` directory (or any other directory you specify).
  This enables us to use multiple certificates for multiple domains.
  Note that if you want to continue to use your old certificates (recommended because of TOFU), they probably lack the `subjectAltName` directive so your old certificates should be placed at the top level of the certificates directory. Otherwise you will get an error similar to this: "The certificate file for example.com is malformed: unexpected error: The server certificate is not valid for the given name"
* The certificate and key file format has been changed from PEM to DER. This simplifies loading certificate and key files without relying on unstable portions of other crates.
  If you want to continue using your existing certificates and keys, please convert them to DER format. You should be able to use these commands if you have openssl installed:
```
openssl x509 -in cert.pem -out cert.der -outform DER
openssl rsa -in key.rsa -out key.der -outform DER
```
  Since agate will automatically generate certificates from now on, the different format should not be a problem because users are not expected to handle certificates unless experienced enough to be able to handle DER formatting as well.

### Fixed
* Agate now requires the use of SNI by any connecting client.
* All log lines are in the same format now:
  `<local ip>:<local port> <remote ip or dash> "<request>" <response status> "<response meta>" [error:<error>]`
  If the connection could not be established correctly (e.g. because of TLS errors), the status code `00` is used.
* Messages from modules other than Agate itself are not logged by default.

## [2.5.3] - 2021-02-27
Thank you to @littleli and @06kellyjac for contributing to this release.

### Added
* Automated tests have been added so things like 2.5.2 should not happen again (#34).
* Version information flag (`-V` or `--version` as conventional with e.g. cargo)

### Changed
* Forbid unsafe code. (There was none before, just make it harder to add some.)
* When logging remote IP addresses, the port is now never logged, which also changes the address format.

### Fixed
* Updated `url` to newest version, which resolves a TODO.
* The help exits successfully with `0` rather than `1` (#37).
* The GitHub workflow has been fixed so Windows binaries are compressed correctly (#36).
* Split out install steps to allow for more options in the future.
* Add install notes for nix/NixOS to the README (#38).
* Updated dependencies.

## [2.5.2] - 2021-02-12

### Fixed
* Semicolons are no longer considered to be starting a comment in `.mime` files.

## [2.5.1] - 2021-02-12
Functionally equivalent to version 2.5.1, only releasing a new version to update README on crates.io.

### Fixed
* Fixed mistakes in the README.

## [2.5.0] - 2021-02-12
Agate now has an explicit code of conduct and contributing guidelines.
Thank you to @ERnsTL, @gegeweb, @SuddenPineapple, and @Ylhp for contributing to this release.

### Added
* You can now supply multiple `--hostname`s to enable basic vhosts (#28, #31).
* Disabling support for TLSv1.2 can now be done using the `--only-tls13` flag, but this is *NOT RECOMMENDED* (#12).
* The tools now also contain a startup script for FreeBSD (#13).
* Using central config mode (flag `-C`), all configuration can be done in one `.meta` file (see README.md for details).
* The `.meta` configuration file now allows for globs to be used.

### Changed
* The `.meta` file parser now uses the `configparser` crate. The syntax does not change.
* The changelog is now also kept in this file in addition to the GitHub releases.
* Certificate chain and key file are now only loaded once at startup, certificate changes need a restart to take effect.
* Hidden files are now served if there is an explicit setting in a `.meta` file for them, regardless of the `--serve-secret` flag.

### Fixed
* The Syntax for the IPv6 address in the README has been corrected.
* Give a better error message when no keys are found in the key file instead of panicking with a range check (#33).

## [2.4.1] - 2020-02-08
### Fixed
* Re-enabled multiple occurrences of `--addr`. This was accidentally disabled by a merge.

## [2.4.0]+podman.build - 2020-02-06
This is the same as [2.4.0], only the build process has been changed so it should accommodate a wider range of architectures and devices.

## [2.4.0] - 2020-02-06
Since there is a new maintainer (@Johann150), the range in pre-compiled binaries has changed a bit.

### Added
* Added some installation tools for Debian.
* Added a sidecar file for specifying languages, MIME media types or complete headers on a per file basis (#16).

### Changed
* Improved logging output. Agate now also respects the `RUST_LOG` environment variable, so you can configure the log level (#22, #23).

## [2.3.0] - 2020-01-17
Thanks to @Johann150.

### Changed
* Combine address and port back into a single command-line argument (#21).

## [2.2.0] - 2020-01-16
Thank you to @gegeweb, @Johann150 and @purexo for contributing to this release.

### Changed
* Split address and port into separate command-line parameters.

### Fixed
* Listen on both IPv6 and IPv4 interfaces by default (#14, #15).
* Do not serve files whose path contains a segment starting with a dot (#17, #20).
* Fix redirects of URLs with query strings (#19).

## [2.1.3] - 2020-01-02
### Changed
* Switch to the Tokio async run time.

### Fixed
* Send TLS close-notify message when closing a connection.
* Require absolute URLs in requests.

## [2.1.2] - 2020-01-01
### Fixed
* More complete percent-encoding of special characters in filenames.
* Minor improvements to error logging.
* Internal code cleanup.

## [2.1.1] - 2020-12-31
### Changed
* List directory content in alphabetical order.

### Fixed
* Handle percent-escaped paths in URLs.
* Percent-escape white space characters in directory listings.

## [2.1.0] - 2020-12-29
* Enabled GitHub Discussions. If you are using Agate, please feel free to leave a comment to let us know about it!
Thank you to @Johann150 and @KilianKemps for contributing to this release.

### Added
* Optional directory listings (#8, #9).

### Fixed
* Updated dependencies.

## [2.0.0] - 2020-12-23
Thank you to @bortzmeyer, @KillianKemps, and @Ylhp for contributing to this release.

### Added
* New `--language` option to add a language tag to the MIME type for text/gemini responses (#6).

### Changed
* New format for command-line options. See the documentation or run `agate --help` for details.
* Logging is enabled by default. Use the `--silent` flag to disable it.
* Pre-compiled binaries are built with the [`cross`](https://github.com/rust-embedded/cross) tool, for better compatibility with older Linux systems.

## [1.3.2] - 2020-12-09
This release is functionally identical to Agate 1.3.1, and users of that version do not need to update.

### Fixed
* Update to async-tls 0.11 because the previous version was [yanked](https://github.com/async-rs/async-tls/issues/42).

## [1.3.1] - 2020-12-08
Thanks @dcreager for contributing this fix.

### Fixed
* Updated dependencies to fix `cargo install` (#7).

## [1.3.0] - 2020-11-20
Thank you @Johann150, @jonhiggs and @tronje for contributing to this release!

### Fixed
* verify hostname and port in request URL (#4).
* improved logging (#2, #3).
* Don't redirect to "/" when the path is empty (#5).
* Update dependencies.

## [1.2.2] - 2020-09-21
Thank you to @m040601 for contributing to this release.

### Changed
* Switch from `tree_magic` to `mime_guess` for simpler MIME type guessing.
* Built both x86_64 and ARM binaries. These binaries are built for Linux operating systems with glibc 2.28 or later, such as Debian 10 ("buster") or newer, Ubuntu 18.10 or newer, and Raspberry Pi OS 2019-06-20 or newer (#1).

### Fixed
* Update dependencies.
* Minor internal code cleanup.

## [1.2.1] - 2020-06-20
### Fixed
* Reduce memory usage when serving large files.
* Update dependencies.

## [1.2.0] - 2020-06-10
### Changed
* text/gemini filename extension from `.gemini` to `.gmi`.

### Fixed
* Handling for requests that exceed 1KB.
* Reduce memory allocations and speed up request parsing.
* Update dependencies.

## [1.1.0] - 2020-05-22
### Added
* Auto-detect MIME types.

## [1.0.1] - 2020-05-21
### Added
* Send more accurate error codes for unsupported requests.
* Do more validation of request URLs.

## [1.0.0] - 2020-05-21

[Unreleased]: https://github.com/mbrubeck/agate/compare/v3.3.0...HEAD
[3.3.0]: https://github.com/mbrubeck/agate/compare/v3.2.4...v3.3.0
[3.2.4]: https://github.com/mbrubeck/agate/compare/v3.2.3...v3.2.4
[3.2.3]: https://github.com/mbrubeck/agate/compare/v3.2.2...v3.2.3
[3.2.2]: https://github.com/mbrubeck/agate/compare/v3.2.1...v3.2.2
[3.2.1]: https://github.com/mbrubeck/agate/compare/v3.2.0...v3.2.1
[3.2.0]: https://github.com/mbrubeck/agate/compare/v3.1.3...v3.2.0
[3.1.3]: https://github.com/mbrubeck/agate/compare/v3.1.2...v3.1.3
[3.1.2]: https://github.com/mbrubeck/agate/compare/v3.1.1...v3.1.2
[3.1.1]: https://github.com/mbrubeck/agate/compare/v3.1.0...v3.1.1
[3.1.0]: https://github.com/mbrubeck/agate/compare/v3.0.3...v3.1.0
[3.0.3]: https://github.com/mbrubeck/agate/compare/v3.0.2...v3.0.3
[3.0.2]: https://github.com/mbrubeck/agate/compare/v3.0.1...v3.0.2
[3.0.1]: https://github.com/mbrubeck/agate/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/mbrubeck/agate/compare/v2.5.3...v3.0.0
[2.5.3]: https://github.com/mbrubeck/agate/compare/v2.5.2...v2.5.3
[2.5.2]: https://github.com/mbrubeck/agate/compare/v2.5.1...v2.5.2
[2.5.1]: https://github.com/mbrubeck/agate/compare/v2.5.0...v2.5.1
[2.5.0]: https://github.com/mbrubeck/agate/compare/v2.4.1...v2.5.0
[2.4.1]: https://github.com/mbrubeck/agate/compare/v2.4.0...v2.4.1
[2.4.0]: https://github.com/mbrubeck/agate/compare/v2.3.0...v2.4.0
[2.3.0]: https://github.com/mbrubeck/agate/compare/v2.2.0...v2.3.0
[2.2.0]: https://github.com/mbrubeck/agate/compare/v2.1.3...v2.2.0
[2.1.3]: https://github.com/mbrubeck/agate/compare/v2.1.2...v2.1.3
[2.1.2]: https://github.com/mbrubeck/agate/compare/v2.1.1...v2.1.2
[2.1.1]: https://github.com/mbrubeck/agate/compare/v2.1.0...v2.1.1
[2.1.0]: https://github.com/mbrubeck/agate/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/mbrubeck/agate/compare/v1.3.2...v2.0.0
[1.3.2]: https://github.com/mbrubeck/agate/compare/v1.3.1...v1.3.2
[1.3.1]: https://github.com/mbrubeck/agate/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/mbrubeck/agate/compare/v1.2.2...v1.3.0
[1.2.2]: https://github.com/mbrubeck/agate/compare/v1.2.1...v1.2.2
[1.2.1]: https://github.com/mbrubeck/agate/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/mbrubeck/agate/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/mbrubeck/agate/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/mbrubeck/agate/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/mbrubeck/agate/releases/tag/v1.0.0
