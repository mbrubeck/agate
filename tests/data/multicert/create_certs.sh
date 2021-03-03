#!/bin/bash

mkdir -p example.com example.org

# create our own CA so we can use rustls without it complaining about using a
# CA cert as end cert
openssl req -x509 -newkey rsa:4096 -keyout ca_key.rsa -out ca_cert.pem -days 3650 -nodes -subj "/CN=example CA"

for domain in "example.com" "example.org"
do
openssl genpkey -out $domain/key.rsa -algorithm RSA -pkeyopt rsa_keygen_bits:4096

cat >openssl.conf <<EOT
[req]
default_bits       = 4096
distinguished_name = req_distinguished_name
req_extensions     = req_ext
prompt             = no

[req_distinguished_name]
countryName                 = US
stateOrProvinceName         = CA
localityName                = Playa Vista
organizationName            = IANA
commonName                  = $domain

[req_ext]
subjectAltName = DNS:$domain
EOT

openssl req -new -sha256 -out request.csr -key $domain/key.rsa -config openssl.conf

openssl x509 -req -sha256 -days 3650 -in request.csr -CA ca_cert.pem -CAkey ca_key.rsa \
    -CAcreateserial -out $domain/cert.pem -extensions req_ext -extfile openssl.conf
done

# clean up
rm openssl.conf request.csr ca_cert.srl
