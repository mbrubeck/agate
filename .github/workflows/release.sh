#!/bin/bash
# This is used to build cross platform linux binaries for a release in CI.
# Since this is not supervised, abort if anything does not work.
set -e

sudo apt update
# Cross-compiling needs a linker for the respective platforms. If you are on a Debian-based x86_64 Linux,
# you can install them with:
sudo apt -y install podman gcc-arm-linux-gnueabihf gcc-aarch64-linux-gnu
# Also install cross compilation tool for cargo
cargo install cross

for i in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu arm-unknown-linux-gnueabihf armv7-unknown-linux-gnueabihf
do
	cross build --verbose --release --target $i
	cp target/$i/release/agate agate.$i
done

# Strip all the binaries.
strip agate.x86_64-unknown-linux-gnu
aarch64-linux-gnu-strip agate.aarch64-unknown-linux-gnu
arm-linux-gnueabihf-strip agate.arm-unknown-linux-gnueabihf agate.armv7-unknown-linux-gnueabihf

# compress the binaries.
gzip agate.*
