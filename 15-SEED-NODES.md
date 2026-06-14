# Sovrn — Seed Node Infrastructure

## WHY SEED NODES

Sovrn's mesh network needs peers to bootstrap. A fresh installation with zero contacts has no one to connect to. Seed nodes solve the cold start problem.

## WHAT SEED NODES DO

1. **Network bootstrap** — First point of contact for new Sovrn installations
2. **Yggdrasil peering** — Act as stable peers in the mesh routing table
3. **DHT anchor** — Store initial segments of the .sovrn DNS registry
4. **Bootstrap directory** — Host a directory of popular/public nodes for new users to discover content
5. **Content relay** — NOT a CDN (no paid storage), but temporarily caches popular content for availability
6. **Update repository** — Mirror of the APT repository for OS updates

## ARCHITECTURE

### Minimum v1 Deployment: 5 Nodes

| Location | Purpose | Specs |
|---|---|---|
| Mumbai, India | Primary for Indian users | 2 vCPU, 4 GB RAM, 100 GB SSD |
| Singapore | APAC coverage | 2 vCPU, 4 GB RAM, 100 GB SSD |
| Frankfurt, EU | EU coverage | 2 vCPU, 4 GB RAM, 100 GB SSD |
| US East | Americas coverage | 2 vCPU, 4 GB RAM, 100 GB SSD |
| US West | Americas + backup | 2 vCPU, 4 GB RAM, 100 GB SSD |

Estimated cost: ~$100-150/month total (Hetzner/DigitalOcean/Vultr)

### What Runs on Each Seed Node

- **Yggdrasil** — mesh routing
- **Caddy** — HTTP server for bootstrap directory
- **DHT node** — stores a slice of the .sovrn registry
- **APT mirror** — Sovrn package repository mirror
- **Relay agent** — temporary content cache (LRU, auto-evicts after 24h, 10 GB max)

### What Does NOT Run on Seed Nodes

- **No user content hosting** — seed nodes are not CDN
- **No user data** — seed nodes don't store personal data
- **No authentication** — seed nodes are public infrastructure, like DNS root servers

## BOOTSTRAP FLOW

```
New Sovrn installation boots for the first time
        │
        ▼
EEBE wizard starts → User picks name, gets identity
        │
        ▼
Sovrn connects to nearest seed node (hardcoded list)
        │
        ▼
Seed node provides:
  - Yggdrasil peering (mesh connectivity)
  - DHT bootstrap (DNS registry access)
  - Bootstrap directory (discover popular nodes)
  - APT repository mirror (updates)
        │
        ▼
User's node joins the mesh
        │
        ▼
Over time, user's node discovers OTHER peers
  - Direct connections to followed nodes
  - Mesh routing through Yggdrasil
  - Seed nodes are no longer the only path
        │
        ▼
User's node is now self-sufficient
  - Seed nodes remain connected as stable peers
  - But user gets content directly from other nodes
```

## DECENTRALIZATION PATH

### Phase 1 (Launch): Centralized Seed Nodes
- Project runs 5 seed nodes
- Hardcoded into Sovrn installation
- This is how Tor, Bitcoin, and IPFS all started

### Phase 2 (Community): Community Seed Nodes
- Community members can volunteer to run seed nodes
- Nodes are added to the bootstrap list via DHT (not hardcoded)
- Community seed nodes must meet minimum uptime and bandwidth requirements

### Phase 3 (Decentralized): Self-Sustaining Mesh
- Enough regular nodes that seed nodes are no longer critical
- Seed nodes can be shut down without affecting the network
- The mesh routes around any single point of failure

## WHAT IF THE PROJECT STOPS RUNNING SEED NODES?

- Hardcoded seed node list includes all 5 nodes + their Yggdrasil public keys
- If all 5 go down permanently, nodes that are already on the mesh continue working (they know other peers)
- New installations would need a manual peer address to bootstrap
- This is the same failure mode as BitTorrent trackers going down — existing swarms work, new ones need a seeder
- Mitigation: the bootstrap list is updatable via APT updates (the OS can add new community seed nodes via system update)
- Last resort: users can manually enter a peer address during setup

## FUNDING

- Seed node costs (~$100-150/month) covered by Sovrn project revenue (CDN marketplace commissions) or out of pocket initially
- Community seed nodes are volunteer-run (like Tor relays)
- Transparent quarterly cost report published for the community