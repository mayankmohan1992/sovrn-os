#!/usr/bin/env bash
# Sovrn OS Certificate Authority Bootstrap
# Generates self-signed CA for .sovrn TLD TLS

set -euo pipefail

CA_DIR="/etc/sovrn/ca"
CA_KEY="$CA_DIR/ca.key"
CA_CERT="$CA_DIR/ca.crt"
VALIDITY_DAYS=3650  # 10 years

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${GREEN}[sovrn-ca]${NC} $*"; }
err() { echo -e "${RED}[sovrn-ca ERROR]${NC} $*" >&2; exit 1; }

# Create CA directory
mkdir -p "$CA_DIR"

# Generate CA private key
if [ -f "$CA_KEY" ]; then
    log "CA key already exists at $CA_KEY"
else
    log "Generating CA private key..."
    openssl genpkey -algorithm ED25519 -out "$CA_KEY" 2>/dev/null || \
    openssl ecparam -genkey -name prime256v1 -out "$CA_KEY" 2>/dev/null || \
    openssl genrsa -out "$CA_KEY" 4096 2>/dev/null
    chmod 600 "$CA_KEY"
    log "CA key generated at $CA_KEY"
fi

# Generate CA certificate
if [ -f "$CA_CERT" ]; then
    log "CA certificate already exists at $CA_CERT"
else
    log "Generating CA certificate..."
    openssl req -new -x509 -key "$CA_KEY" \
        -out "$CA_CERT" \
        -days "$VALIDITY_DAYS" \
        -subj "/C=IN/ST=Rajasthan/L=Jaipur/O=Sovrn OS/OU=Mesh CA/CN=Sovrn Mesh Root CA/emailAddress=ca@sovrn.local" \
        -addext "basicConstraints=critical,CA:TRUE" \
        -addext "keyUsage=critical,digitalSignature,keyCertSign,cRLSign" \
        -addext "subjectAltName=DNS:sovrn.local,DNS:*.sovrn,IP:127.0.0.1,IP:::1"

    chmod 644 "$CA_CERT"
    log "CA certificate generated at $CA_CERT"
fi

# Install CA into system trust store
log "Installing CA into system trust store..."
if command -v update-ca-certificates &>/dev/null; then
    cp "$CA_CERT" /usr/local/share/ca-certificates/sovrn-ca.crt
    update-ca-certificates --fresh 2>/dev/null || true
    log "CA installed (Debian/Ubuntu)"
elif command -v trust &>/dev/null; then
    cp "$CA_CERT" /etc/pki/ca-trust/source/anchors/sovrn-ca.crt
    update-ca-trust extract 2>/dev/null || true
    log "CA installed (Fedora/RedHat)"
else
    log "Warning: Could not auto-install CA into system trust store"
fi

# Generate server certificate for sovrnd
log "Generating server certificate..."
SERVER_KEY="$CA_DIR/server.key"
SERVER_CSR="$CA_DIR/server.csr"
SERVER_CERT="$CA_DIR/server.crt"

openssl genpkey -algorithm ED25519 -out "$SERVER_KEY" 2>/dev/null || \
openssl ecparam -genkey -name prime256v1 -out "$SERVER_KEY" 2>/dev/null

openssl req -new -key "$SERVER_KEY" \
    -out "$SERVER_CSR" \
    -subj "/C=IN/ST=Rajasthan/L=Jaipur/O=Sovrn OS/OU=Server/CN=localhost"

openssl x509 -req -in "$SERVER_CSR" \
    -CA "$CA_CERT" -CAkey "$CA_KEY" \
    -CAcreateserial \
    -out "$SERVER_CERT" \
    -days 365 \
    -extfile <(cat <<EOF
basicConstraints = CA:FALSE
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = localhost
DNS.2 = *.sovrn
DNS.3 = sovrn.local
IP.1 = 127.0.0.1
IP.2 = ::1
EOF
)

chmod 644 "$SERVER_CERT"
chmod 600 "$SERVER_KEY"

log "Server certificate generated at $SERVER_CERT"
log "CA bootstrap complete!"
log "  CA Certificate: $CA_CERT"
log "  Server Key:      $SERVER_KEY"
log "  Server Cert:     $SERVER_CERT"
