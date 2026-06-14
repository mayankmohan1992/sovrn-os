1|# Sovrn — Network Design
2|
3|## MESH NETWORK ARCHITECTURE
4|
5|### Layer 1: Yggdrasil (Transport)
6|- Each node gets a stable IPv6 address derived from its Ed25519 public key
7|- Addresses are cryptographic — cannot be spoofed
8|- Encrypted end-to-end between any two nodes
9|- Handles NAT traversal automatically (no port forwarding needed)
10|- Routes traffic through the mesh — even if direct connection is impossible
11|
12|### Layer 2: DHT (DNS Registry)
13|- Distributed hash table overlaid on Yggdrasil
14|- Each node stores a SLICE of the total .sovrn registry
15|- Registering a domain: sign a message with your identity key, broadcast to DHT
16|- Resolving a domain: query the DHT for `name.sovrn` → returns Yggdrasil IPv6 address
17|- First-come-first-serve within rate limits (5 domains/month 1, 3 month 2, 2/month after)
18|- Expiration: domains not meeting minimum use threshold expire after 12 months
19|- Conflict resolution: earliest registration timestamp wins (signed and verifiable)
20|
21|### Layer 3: Presence (Online/Offline Detection)
22|- Heartbeat: each online node broadcasts "I'm alive" every 2 minutes
23|- Broadcast mechanism: Yggdrasil multicast or gossip protocol
24|- Offline detection: 
25|  - Graceful: node sends "going offline" signal before shutdown/sleep
26|  - Timeout: if no heartbeat for 6 minutes (3x interval), node marked offline
27|- Feed behavior: offline users' content is silently hidden — no "unavailable" placeholders
28|- When user comes back online, their content reappears in followers' feeds
29|
30|### Layer 4: Social (Content Protocol)
31|- Based on Nostr-like signed events
32|- Event types: text post, image post, follow, unfollow, mute, block, reaction (private)
33|- Each event is signed with the author's Ed25519 key
34|- Events stored locally on the author's node
35|- Followers pull events from nodes they follow (when those nodes are online)
36|- Media (images, video) stored as IPFS content-addressed blobs alongside the event
37|- CDN pushed: author can opt to push media to a CDN node for availability when offline
38|
39|### Layer 5: Messaging (Direct Messages)
40|- Same signed-event protocol as social feed
41|- Event type: encrypted DM (encrypted with recipient's X25519 public key)
42|- Delivered directly to recipient's node via Yggdrasil
43|- If recipient is offline: message queued on sender's node, delivered when both online
44|- No server ever sees plaintext — only encrypted blobs
45|
46|## DUAL NETWORK DESIGN
47|
48|### Regular Internet + Mesh Coexist
49|```
50|User opens browser → types URL:
51|  - "alice.sovrn"     → mesh DNS → Yggdrasil → fetch from Alice's node
52|  - "google.com"   → regular DNS → internet → normal browsing
53|  - "twitter.com"  → regular DNS → internet → normal browsing
54|
55|Both interfaces active simultaneously:
56|  - eth0/wlan0: regular internet (DHCP, default gateway)
57|  - ygg0: Yggdrasil mesh (auto-configured overlay)
58|  
59|Routing:
60|  - .sovrn TLD → ygg0
61|  - everything else → eth0/wlan0
62|```
63|
64|### DNS Resolution
65|```
66|Query: "alice.sovrn"
67|  1. OS checks: is this a .sovrn domain? → YES
68|  2. Query mesh DHT for "alice.sovrn"
69|  3. Get back: Yggdrasil IPv6 address for Alice's node
70|  4. Connect to Alice's node via ygg0
71|  5. Fetch homepage
72|
73|Query: "google.com"
74|  1. OS checks: is this a .sovrn domain? → NO
75|  2. Query regular DNS (Unbound resolver)
76|  3. Normal internet browsing
77|```
78|
79|## CDN NETWORK
80|
81|### Architecture
82|- CDN nodes are NOT mesh nodes — they are traditional servers with high bandwidth
83|- CDN nodes run a mesh node (Yggdrasil) for discovery + a CDN agent for storage
84|- Content is pushed to CDN by the author (not pulled)
85|- CDN stores encrypted, content-addressed blobs
86|- CDN cannot decrypt content — it only stores and serves blobs
87|- Users pay CDN operators for storage + bandwidth (cryptocurrency or traditional payment)
88|- OS has built-in CDN directory: browse available CDN nodes, compare prices, select one
89|
90|### Content Flow
91|```
92|Alice uploads a video:
93|1. Local node: encrypts video, calculates CID (content identifier)
94|2. Local node: pushes encrypted blob to chosen CDN node
95|3. Local node: publishes signed event referencing the CID
96|4. Followers see the event, fetch CID from CDN (or from Alice's node if online)
97|
98|Alice goes offline:
99|1. Bob (follower) sees Alice is offline via presence system
100|2. Bob's node checks: does CDN have this CID? → YES
101|3. Bob fetches encrypted blob from CDN
102|4. Bob decrypts with Alice's public key (for public content) or session key (for follower-only content)
103|```
104|
105|## BANDWIDTH MANAGEMENT
106|
107|### Home Node Constraints
108|- Typical Jaipur home upload: 10-50 Mbps
109|- Text posts: negligible (<1 KB per post)
110|- Images: 100 KB - 5 MB each
111|- Video: 500 MB+ per 10-min video
112|
113|### Strategy
114|- Text and thumbnails always served from home node
115|- Images over 1 MB: CDN-first if available, home node fallback
116|- Video: ALWAYS CDN-first, home node only serves metadata + thumbnail
117|- Swarm caching: followers who've already downloaded content can serve it to nearby nodes (BitTorrent-style, optional)
118|
119|## SECURITY MODEL
120|
121|### Threat Model
122|| Threat | Mitigation |
123||---|---|
124|| Eavesdropping on mesh traffic | Yggdrasil E2E encryption |
125|| Node impersonation | Ed25519 identity keys, signing required |
126|| CDN content tampering | Content-addressed (CID hashes verified) |
127|| DNS hijacking | DHT records signed by registrar's key |
128|| Local data theft | LUKS2 full disk encryption |
129|| Lost identity | BIP-39 seed phrase backup |
130|| Spam in feed | User-controlled mute/block/rate-limit |
131|| Malicious apps | Flatpak sandboxing + AppArmor |
132|| OS supply chain attack | Signed updates (APT secure) |