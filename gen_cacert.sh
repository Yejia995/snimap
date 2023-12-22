#!/usr/bin/env bash
set -eu
cert_dir=private
name=snimap_root_ca

mkdir -p "$cert_dir"

openssl genpkey -algorithm RSA -out "$cert_dir/key.crt"

openssl req -x509 -key "$cert_dir/key.crt" -out "$cert_dir/ca.crt" \
    -days 36500 \
    -subj "/CN=$name" \
    -config <(
        cat <<END
[ req ]
distinguished_name  = req_distinguished_name

[ req_distinguished_name ]

[ x509_ext ]
basicConstraints = critical,CA:true
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always,issuer
END
    ) -extensions x509_ext
