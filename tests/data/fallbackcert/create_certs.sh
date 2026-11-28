#!/bin/bash

# The test associated with this cert (fallback_for_suffix_lookalike)
# attempts to connect with sni name notexample.com
#
# This script ensures that the fallback cert has that domain in its common name
# so that the tests passes if the fallback is served rather than example.com's
#
# This script isn't invoked by the tests themselves and was only used to set the
# test data up

create_cert() {

dir="$1"
domain="$2"

mkdir -p "$dir"

# create private key
# and config file
openssl genpkey -outform DER -out $dir/key.der -algorithm RSA -pkeyopt rsa_keygen_bits:4096
cat >openssl.conf <<EOT
[req]
default_bits = 4096
distinguished_name = req_distinguished_name
req_extensions = req_ext
prompt = no

[v3_ca]
basicConstraints = critical, CA:false

[req_distinguished_name]
commonName = $domain

[req_ext]
subjectAltName = DNS:$domain
EOT

openssl req -new -sha256 -out request.csr -key $dir/key.der -keyform DER -config openssl.conf
openssl x509 -req -sha256 -days 3650 -in request.csr -outform DER -out $dir/cert.der \
	-extensions req_ext -extfile openssl.conf -signkey $dir/key.der -keyform DER

}

# make the fallback certificate
create_cert . notexample.com
# make a cert that shares a textual suffix
create_cert example.com example.com

# tidy up
rm openssl.conf request.csr
