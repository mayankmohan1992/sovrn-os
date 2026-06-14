# Sovrn — IPC Protocols, Networking & Service Configs

## D-IPC1: sovrnd ↔ Service Communication

### Protocol: Unix Domain Sockets with JSON-RPC 2.0

sovrnd communicates with all mesh services over Unix domain sockets in `/var/lib/sovrn/sockets/`. Each service listens on its own socket.

| Service | Socket Path | Protocol |
|---------|------------|----------|
| sovrn-dht | `/var/lib/sovrn/sockets/dht.sock` | JSON-RPC 2.0 |
| sovrn-identity | `/var/lib/sovrn/sockets/identity.sock` | JSON-RPC 2.0 |
| sovrn-presence | `/var/lib/sovrn/sockets/presence.sock` | JSON-RPC 2.0 |
| sovrn-feed | `/var/lib/sovrn/sockets/feed.sock` | JSON-RPC 2.0 |
| sovrn-message-queue | `/var/lib/sovrn/sockets/mq.sock` | JSON-RPC 2.0 |
| sovrn-cdn-agent | `/var/lib/sovrn/sockets/cdn.sock` | JSON-RPC 2.0 |

All sockets are owned by `sovrn:sovrn` with mode `0660` (group-readable). sovrnd runs as user `sovrn`.

### JSON-RPC 2.0 Message Format

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "dht.lookup",
  "params": { "domain": "bob.sovrn" },
  "id": 1
}

// Success Response
{
  "jsonrpc": "2.0",
  "result": {
    "domain": "bob.sovrn",
    "public_key": "b3k7m1n4q7r2t6w9",
    "ygg_address": "200:abc:...",
    "online": true
  },
  "id": 1
}

// Error Response
{
  "jsonrpc": "2.0",
  "error": { "code": -32600, "message": "Domain not found" },
  "id": 1
}
```

### Rust Service Server Pseudocode

```rust
// Each Rust service uses tokio + serde_json for JSON-RPC over Unix socket
use tokio::net::UnixListener;
use serde_json::Value;

async fn serve(socket_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = UnixListener::bind(socket_path)?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (reader, writer) = stream.into_split();
            // Read JSON-RPC request, dispatch to handler, write response
            handle_json_rpc(reader, writer).await;
        });
    }
}
```

### Python sovrnd Client Pseudocode

```python
# sovrnd ipc_client.py
import json
import socket

SOCKET_DIR = "/var/lib/sovrn/sockets"

class ServiceClient:
    def __init__(self, service_name: str):
        self.socket_path = f"{SOCKET_DIR}/{service_name}.sock"

    def call(self, method: str, params: dict = None) -> dict:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as sock:
            sock.connect(self.socket_path)
            request = {
                "jsonrpc": "2.0",
                "method": method,
                "params": params or {},
                "id": 1
            }
            sock.sendall(json.dumps(request).encode() + b"\n")
            response = json.loads(sock.recv(65536).decode())
            if "error" in response:
                raise ServiceError(response["error"])
            return response.get("result", {})

# Usage in sovrnd:
dht = ServiceClient("dht")
result = dht.call("dht.lookup", {"domain": "bob.sovrn"})
```

### Service Method Registry

#### sovrn-dht methods
| Method | Params | Returns |
|--------|--------|---------|
| `dht.lookup` | `{domain}` | `{domain, public_key, ygg_address, online}` |
| `dht.register` | `{name, public_key, signature}` | `{domain, registered: true}` |
| `dht.check_available` | `{name}` | `{available, suggestions}` |
| `dht.get_peers` | `{limit}` | `{peers: [{public_key, domain, ygg_address, latency_ms}]}` |
| `dht.put` | `{key, value, ttl}` | `{stored: true}` |
| `dht.get` | `{key}` | `{value, found: true}` |
| `dht.health` | `{}` | `{status, nodes_count, uptime_seconds}` |

#### sovrn-identity methods
| Method | Params | Returns |
|--------|--------|---------|
| `identity.get_profile` | `{public_key}` | `{id, name, domain, about, avatar_cid}` |
| `identity.update_profile` | `{name?, about?, avatar_cid?}` | `{updated_profile}` |
| `identity.list_aliases` | `{}` | `{aliases: [{name, domain, ...}]}` |
| `identity.create_alias` | `{name}` | `{name, domain, registered}` |
| `identity.delete_alias` | `{name}` | `{deleted: true}` |
| `identity.export_identity` | `{password}` | `{seed_phrase, master_keypair, aliases}` |
| `identity.import_identity` | `{seed_phrase}` | `{id, name, domain}` |
| `identity.sign_event` | `{event_json}` | `{signature}` |
| `identity.verify_event` | `{event_json, signature}` | `{valid: true}` |
| `identity.derive_app_key` | `{app_id}` | `{username, password}` |

#### sovrn-presence methods
| Method | Params | Returns |
|--------|--------|---------|
| `presence.get_online` | `{limit}` | `{peers: [{public_key, domain, status}]}` |
| `presence.set_status` | `{status}` | `{status}` |
| `presence.heartbeat` | `{}` | `{acknowledged: true}` |
| `presence.get_status` | `{public_key}` | `{public_key, status, last_heartbeat}` |

#### sovrn-feed methods
| Method | Params | Returns |
|--------|--------|---------|
| `feed.get_timeline` | `{cursor, limit}` | `{events, cursor, has_more}` |
| `feed.get_event` | `{event_id}` | `{event}` |
| `feed.create_event` | `{kind, content, tags, media}` | `{event}` |
| `feed.delete_event` | `{event_id}` | `{deleted: true}` |
| `feed.push_event` | `{event_json}` | `{received: true}` |
| `feed.get_reactions` | `{event_id, cursor, limit}` | `{reactions, cursor, has_more}` |
| `feed.add_reaction` | `{event_id, type}` | `{reaction}` |
| `feed.search` | `{query, kind, limit}` | `{events}` |
| `feed.health` | `{}` | `{status, events_count, uptime}` |

#### sovrn-message-queue methods
| Method | Params | Returns |
|--------|--------|---------|
| `mq.get_conversations` | `{cursor, limit}` | `{conversations, cursor}` |
| `mq.get_messages` | `{peer_id, cursor, limit}` | `{messages, cursor, has_more}` |
| `mq.send_dm` | `{peer_id, content, media}` | `{message}` |
| `mq.mark_read` | `{peer_id}` | `{updated: true}` |
| `mq.receive_dm` | `{from_id, content, sig}` | `{received: true}` |
| `mq.get_delivery_status` | `{message_id}` | `{status}` |
| `mq.health` | `{}` | `{status, queue_depth, uptime}` |

#### sovrn-cdn-agent methods
| Method | Params | Returns |
|--------|--------|---------|
| `cdn.push` | `{cid, local_path, size, checksum}` | `{push_id, status}` |
| `cdn.get_status` | `{push_id}` | `{status, providers}` |
| `cdn.get_providers` | `{}` | `{providers}` |
| `cdn.subscribe` | `{provider_id, plan, payment_method}` | `{subscription}` |
| `cdn.health` | `{}` | `{status, queue_depth, uptime}` |

---

## D-IPC2: Node-to-Node Protocols

### Protocol: Yggdrasil TCP with Length-Prefixed JSON

All mesh communication happens over Yggdrasil IPv6. Messages are length-prefixed JSON frames:

```
[4 bytes: big-endian length][JSON payload]
```

### Feed Push Protocol (node-to-node)

When Alice's node creates a post, it pushes to followers' nodes:

```json
{
  "type": "feed_push",
  "version": 1,
  "event": {
    "id": "sha256abc",
    "kind": 1,
    "author": "a7x3k9m2p8f1q4b5",
    "content": "Hello Sovrn!",
    "created_at": 1749000060,
    "tags": [],
    "sig": "edb0de...hex..."
  },
  "push_id": "push_a7x3k9_1749000060"
}
```

**Ack:**
```json
{ "type": "feed_push_ack", "push_id": "push_a7x3k9_1749000060", "status": "stored" }
```

### DM Delivery Protocol (node-to-node)

```json
{
  "type": "dm_delivery",
  "version": 1,
  "message_id": "msg_sha256_1",
  "from": "a7x3k9m2p8f1q4b5",
  "to": "b3k7m1n4q7r2t6w9",
  "content": "...encrypted_x25519...",
  "created_at": 1748999400,
  "sig": "...ed25519_sig..."
}
```

**Ack:**
```json
{ "type": "dm_ack", "message_id": "msg_sha256_1", "status": "delivered" }
```

### DHT Protocol (Kademlia-based)

**FIND_NODE:** Find a node responsible for a key
```json
{
  "type": "dht_find_node",
  "version": 1,
  "query_id": "a7x3k9m2p8f1q4b5",
  "target_key": "domain:bob.sovrn",
  "sender": "a7x3k9m2p8f1q4b5",
  "sender_addr": "200:abc:..."
}
```

**FIND_NODE_RESPONSE:**
```json
{
  "type": "dht_find_node_response",
  "query_id": "a7x3k9m2p8f1q4b5",
  "nodes": [
    { "public_key": "b3k7m1...", "addr": "200:def:...", "distance": 128 }
  ]
}
```

**STORE:** Store a value in the DHT
```json
{
  "type": "dht_store",
  "key": "domain:alice.sovrn",
  "value": "{\"public_key\":\"a7x3k9...\",\"ygg_address\":\"200:abc:...\"}",
  "ttl": 86400,
  "signature": "...ed25519_sig..."
}
```

### Heartbeat Gossip (node-to-node, ephemeral)

Every 2 minutes, each node multicasts its presence:

```json
{
  "type": "heartbeat",
  "version": 1,
  "public_key": "a7x3k9m2p8f1q4b5",
  "domain": "alice.sovrn",
  "status": "online",
  "timestamp": 1749000030,
  "sig": "...ed25519_sig..."
}
```

Heartbeats are propagated with TTL=3 (max 3 hops) and cached for 180 seconds.

---

## D-NET1: Caddy Configuration (Production)

File: `/etc/caddy/Caddyfile`

```caddy
# Sovrn OS - Caddy Configuration
# Serves PWA, API, and self-hosted app subdomains

sovrn.local {
    # PWA static files
    root * /var/lib/sovrn/pwa
    file_server

    # SPA routing — serve index.html for all non-file paths
    try_files {path} /index.html

    # API reverse proxy to sovrnd
    reverse_proxy /api/v1/* 127.0.0.1:54771 {
        header_up X-Request-ID {uuid}
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-Proto {scheme}

        # WebSocket support
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
    }

    # WebSocket specific route
    reverse_proxy /api/v1/ws 127.0.0.1:54771 {
        # Force WebSocket upgrade
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
    }

    # Self-signed TLS (generated on first boot)
    tls /var/lib/sovrn/ca/sovrn.local.pem /var/lib/sovrn/ca/sovrn.local.key

    # Security headers
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "SAMEORIGIN"
        Referrer-Policy "strict-origin-when-cross-origin"
        Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self' wss://sovrn.local"
    }
}

# Self-hosted app subdomains — dynamically managed by sovrnd
# sovrnd adds/removes these via Caddy API

{# Templates for app subdomains — managed by sovrnd #}
# nextcloud.sovrn.local {
#     reverse_proxy 10.47.0.2:8080
#     tls /var/lib/sovrn/ca/sovrn.local.pem /var/lib/sovrn/ca/sovrn.local.key
# }
```

### sovrnd Caddy Route Management

sovrnd dynamically adds/removes app routes via Caddy's admin API:

```python
# sovrnd caddy_manager.py
import json
import urllib.request

CADDY_API = "http://localhost:2019"

def add_app_route(subdomain: str, upstream: str):
    """Add a reverse proxy route for an app subdomain."""
    route = {
        "@id": f"sovrn-app-{subdomain}",
        "match": [{"host": [f"{subdomain}.sovrn.local"]}],
        "handle": [{
            "handler": "reverse_proxy",
            "upstreams": [{"dial": upstream}],
            "headers": {
                "request": {"set": {"X-Forwarded-Proto": ["https"]}}
            }
        }],
        "tls": {
            "certificates": [{"certificate": "/var/lib/sovrn/ca/sovrn.local.pem",
                            "key": "/var/lib/sovrn/ca/sovrn.local.key"}]
        }
    }
    # POST to Caddy API
    req = urllib.request.Request(
        f"{CADDY_API}/config/apps/http/servers/srv0/routes",
        data=json.dumps(route).encode(),
        method="POST",
        headers={"Content-Type": "application/json"}
    )
    urllib.request.urlopen(req)

def remove_app_route(subdomain: str):
    """Remove a reverse proxy route."""
    req = urllib.request.Request(
        f"{CADDY_API}/config/apps/http/servers/srv0/routes/sovrn-app-{subdomain}",
        method="DELETE"
    )
    urllib.request.urlopen(req)
```

---

## D-NET2: DNS Dispatcher Configuration

### systemd-resolved + Custom Dispatcher

File: `/etc/systemd/resolved.conf.d/sovrn.conf`

```ini
[Resolve]
# Use our custom DNS dispatcher as the primary resolver
DNS=127.0.0.1
# Fallback to ISP DNS
FallbackDNS=1.1.1.1#cloudflare 9.9.9.9#quad9
# Search domain for local services
Domains=~sovrn.local
# Never use LLMNR
LLMNR=no
# Never use mDNS for .sovrn
MulticastDNS=no
```

### DNS Dispatcher Script

File: `/usr/local/bin/sovrn-dns-dispatcher`

```python
#!/usr/bin/env python3
"""Sovrn DNS Dispatcher — routes .sovrn, .sovrn.local, and internet queries."""
import socketserver
import struct
import json
import re

DNS_PORT = 53535  # Our custom DNS resolver port

def resolve_sovrn(domain: str) -> list:
    """Resolve .sovrn domain via DHT."""
    # Connect to sovrn-dht Unix socket
    client = ServiceClient("dht")
    result = client.call("dht.lookup", {"domain": domain})
    if result:
        return [result["ygg_address"]]
    return []

def resolve_sovrn_local(domain: str) -> list:
    """Resolve .sovrn.local domain via local /etc/hosts + mDNS."""
    subdomain = domain.replace(".sovrn.local", "")
    if subdomain == "sovrn":
        return ["127.0.0.1"]  # PWA + API
    # Check /var/lib/sovrn/hosts.local for app subdomains
    # (managed by sovrnd when apps are installed)
    try:
        with open("/var/lib/sovrn/hosts.local") as f:
            for line in f:
                if line.strip() and not line.startswith("#"):
                    ip, *names = line.split()
                    if domain in names:
                        return [ip]
    except FileNotFoundError:
        pass
    return []

def resolve_internet(domain: str) -> list:
    """Resolve internet domains via Unbound."""
    # Forward to Unbound on 127.0.0.1:53535
    # (actual implementation uses dns.resolver library)
    pass

def dispatch(query_name: str) -> list:
    if query_name.endswith(".sovrn") and not query_name.endswith(".sovrn.local"):
        return resolve_sovrn(query_name)
    elif query_name.endswith(".sovrn.local"):
        return resolve_sovrn_local(query_name)
    else:
        return resolve_internet(query_name)
```

### Local Hosts File (managed by sovrnd)

File: `/var/lib/sovrn/hosts.local`

```
# Sovrn local service hosts — managed by sovrnd, do not edit manually
127.0.0.1  sovrn.local        # PWA + API
10.47.0.2  nextcloud.sovrn.local  # Nextcloud
10.47.0.3  vaultwarden.sovrn.local  # Vaultwarden
# ... entries added/removed by sovrnd when apps are installed/uninstalled
```

---

## D-NET3: nftables Ruleset

File: `/etc/nftables/sovrn.nft`

```nftables
#!/usr/sbin/nft -f

# Sovrn OS Firewall Rules
# Flush existing rules
flush ruleset

table inet sovrn {
    # Loopback — allow all
    chain input {
        type filter hook input priority 0; policy drop;

        # Allow established/related connections
        ct state established,related accept
        ct state invalid drop

        # Loopback
        iif lo accept

        # ICMP (ping)
        ip protocol icmp accept
        ip6 nexthdr icmpv6 accept

        # Yggdrasil mesh — allow all on Yggdrasil interface
        iifname "ygg0" accept

        # Local services — only from localhost
        # sovrnd API
        ip saddr 127.0.0.1 tcp dport 54771 accept
        # Notify bridge
        ip saddr 127.0.0.1 tcp dport 54773 accept
        # Caddy (HTTPS)
        tcp dport 443 accept
        tcp dport 80 accept

        # Podman containers bridge
        iifname "podman0" accept
        iifname "podman*" accept

        # SSH (from local network only)
        ip saddr 192.168.0.0/16 tcp dport 22 accept
        ip saddr 10.0.0.0/8 tcp dport 22 accept

        # Drop everything else
        log prefix "nftables-drop: " drop
    }

    chain forward {
        type filter hook forward priority 0; policy drop;

        # Allow established
        ct state established,related accept

        # Allow Podman containers to reach internet (for image pulls)
        iifname "podman*" oifname != "ygg0" accept

        # Allow Podman containers to reach Yggdrasil mesh
        iifname "podman*" oifname "ygg0" accept

        # Drop everything else
        log prefix "nftables-fwd-drop: " drop
    }

    chain output {
        type filter hook output priority 0; policy accept;
        # Allow all outbound — Sovrn is a client OS
    }

    # NAT for Podman containers to access internet
    chain postrouting {
        type nat hook postrouting priority 100;
        # Masquerade Podman container traffic going to internet
        iifname "podman*" oifname != "ygg0" masquerade
    }
}
```

---

## D-NET4: AppArmor Profiles

### sovrnd Profile

File: `/etc/apparmor.d/usr.bin.sovrnd`

```
#include <tunables/global>

/usr/bin/python3 flags=(complain) {
  #include <abstractions/base>
  #include <abstractions/python>
  #include <abstractions/nameservice>

  # Read sovrn config and data
  /var/lib/sovrn/** rwk,
  /etc/sovrn/** r,

  # Unix domain sockets for IPC
  /var/lib/sovrn/ockets/*.sock rw,

  # Caddy admin API
  network inet tcp,

  # Podman management
  /run/podman/** rw,
  /usr/bin/podman Px,

  # systemd interaction
  /run/systemd/journal/socket rw,

  # Deny everything else
  deny /home/** rwlx,
  deny /root/** rwlx,
}
```

### Yggdrasil Profile

File: `/etc/apparmor.d/usr.bin.yggdrasil`

```
#include <tunables/global>

/usr/bin/yggdrasil flags=(enforce) {
  #include <abstractions/base>
  #include <abstractions/nameservice>

  # Config file
  /etc/yggdrasil/yggdrasil.conf r,

  # Yggdrasil interface
  network inet6,
  network inet,

  # Data directory
  /var/lib/yggdrasil/ r,
  /var/lib/yggdrasil/** rwk,

  # TUN device
  /dev/net/tun rw,

  deny /home/** rwlx,
}
```

---

## D-NET5: Self-Signed CA Bootstrapping

File: `/usr/local/bin/sovrn-bootstrap-ca`

```bash
#!/bin/bash
# Sovrn CA Bootstrap Script — runs on first boot
# Generates root CA and signs certificates for sovrn.local

set -euo pipefail

CA_DIR="/var/lib/sovrn/ca"
HOSTNAME=$(hostname)

# Create CA directory
mkdir -p "$CA_DIR"

# Generate root CA (if not exists)
if [ ! -f "$CA_DIR/root-ca.key" ]; then
    openssl genpkey -algorithm Ed25519 -out "$CA_DIR/root-ca.key"
    openssl req -new -x509 -key "$CA_DIR/root-ca.key" \
        -out "$CA_DIR/root-ca.pem" \
        -days 3650 \
        -subj "/CN=Sovrn Root CA/O=Sovrn/C=IN"

    # Trust root CA system-wide
    cp "$CA_DIR/root-ca.pem" /usr/local/share/ca-certificates/sovrn-root-ca.crt
    update-ca-certificates 2>/dev/null || true
fi

# Generate server certificate for sovrn.local
if [ ! -f "$CA_DIR/sovrn.local.key" ]; then
    # Create extensions file
    cat > "$CA_DIR/sovrn.local.ext" <<EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage=digitalSignature,keyAgreement,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=@alt_names

[alt_names]
DNS.1=sovrn.local
DNS.2=*.sovrn.local
DNS.3=localhost
IP.1=127.0.0.1
IP.2=::1
EOF

    openssl genpkey -algorithm Ed25519 -out "$CA_DIR/sovrn.local.key"
    openssl req -new -key "$CA_DIR/sovrn.local.key" \
        -out "$CA_DIR/sovrn.local.csr" \
        -subj "/CN=sovrn.local/O=Sovrn/C=IN"

    openssl x509 -req -in "$CA_DIR/sovrn.local.csr" \
        -CA "$CA_DIR/root-ca.pem" \
        -CAkey "$CA_DIR/root-ca.key" \
        -CAcreateserial \
        -out "$CA_DIR/sovrn.local.pem" \
        -days 365 \
        -extfile "$CA_DIR/sovrn.local.ext"

    echo "Sovrn CA and server certificate generated."
else
    echo "Sovrn CA already exists, skipping."
fi
```

---

## D-NET6: Yggdrasil Configuration

File: `/etc/yggdrasil/yggdrasil.conf`

```yaml
# Sovrn OS — Yggdrasil Configuration
# Auto-generated by sovrnd on first boot

IfName: ygg0
AdminListen: unix:///var/run/yggdrasil/yggdrasil.sock

# Public bootstrap peers (Sovrn seed nodes)
Peers:
  - tls://seed1.sovrn.org:443
  - tls://seed2.sovrn.org:443
  - tls://seed3.sovrn.org:443
  - tls://seed4.sovrn.org:443
  - tls://seed5.sovrn.org:443

# Local peering (mDNS discovery)
InterfacePeers:
  _sovrn._tcp: []

# Allowed encryption public keys (empty = allow all)
AllowedEncryptionPublicKeys: []

# Multicast interface discovery
MulticastInterfaces:
  - Regex: .*
    Beacon: true
    Listen: true
    Port: 54321
    Priority: 0

# Session firewall (allow all mesh traffic by default)
SessionFirewall:
  Enable: false

# Tunnel routing
TunnelRouting:
  Enable: false

# Node info — auto-populated from Sovrn identity
NodeInfoPrivacy: false

# Logging
LogLevel: info
```