#!/bin/bash

mkdir -p example.com example.org

for domain in "example.com" "example.org"
do
# create private key
openssl genpkey -outform DER -out $domain/key.der -algorithm RSA -pkeyopt rsa_keygen_bits:4096

# create config file:
# the generated certificates must not be CA-capable, otherwise rustls complains
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

openssl req -new -sha256 -out request.csr -key $domain/key.der -keyform DER -config openssl.conf

openssl x509 -req -sha256 -days 3650 -in request.csr -outform DER -out $domain/cert.der \
	-extensions req_ext -extfile openssl.conf -signkey $domain/key.der -keyform DER
done

# clean up
rm openssl.conf request.csr
