1|# Sovrn — Architecture Decisions (Round 2)
2|
3|## RESOLVED FROM ROUND 2
4|
5|### D1: Heartbeat Frequency → 2 minutes + explicit offline signal
6|- Heartbeat every 2 minutes
7|- Graceful shutdown/sleep sends explicit "going offline" message
8|- Timeout: 6 minutes (3 missed heartbeats) = offline
9|- This means followers see you disappear within ~2 minutes of unexpected disconnection
10|
11|### D2: Follow Limit → No cap
12|- No hard limit on follows
13|- Performance impact explained below in "Follow Count Performance" section
14|- Architecture designed to handle unlimited follows efficiently
15|
16|### D3: Cold Start → Official bootstrap nodes + offline-first tools
17|- Project runs official bootstrap nodes with welcome content, guides, community
18|- OS ships with useful offline apps pre-installed (file manager, editor, media player, terminal)
19|- No internet dependency for basic use
20|
21|### D4: CDN Payments → Direct, project takes no cut by default
22|- Users buy directly from CDN providers (like choosing a cloud provider)
23|- Project takes a cut only for negotiated partnerships with specific CDN providers
24|- Payments: credit card, UPI, cryptocurrency (privacy-preserving options)
25|- Built-in CDN directory in OS settings (compare providers, prices, reliability)
26|
27|### D5: Spam Control → User-controlled tools only
28|- Mute, block, per-user rate limits
29|- No protocol-level rate limiting on posts (future option preserved if needed)
30|- Users curate their own feed: "Don't show more than X posts per hour from user Y" (optional setting)
31|
32|### D6: Content Format → One app, multiple channels
33|- Unified app with content type as a field (Micro, Media, Video, Article)
34|- One identity across all channels
35|- User can filter feed by channel type or see everything mixed
36|- This is like Nostr event kinds but in a single cohesive UI
37|
38|### D7: Feature Scope → v1 first, iterate
39|- v1: DMs + public feed (text + images)
40|- v2: Groups, video, email gateway, Raspberry Pi support
41|- v3: Communities, advanced CDN marketplace, mobile companion
42|
43|### D8: OS Updates → Signed repos + P2P seeding (see UPDATE-SYSTEM.md)
44|- Centralized signed repository metadata (trusted)
45|- P2P package distribution (bandwidth-efficient)
46|- Multiple security layers against supply chain attacks
47|
48|### D9: Hardware → x86_64 for v1, ARM later
49|- v1: x86_64 laptops and desktops only
50|- v2: Raspberry Pi 4/5 as always-on companion node
51|- v3: ARM laptops
52|
53|### D10: .sovrn TLD → Mesh-only, dual-browser
54|- .sovrn resolves only on mesh network
55|- Regular internet DNS works alongside
56|- Both Chromium-based and Firefox-based browsers included
57|- Auto-routing: .sovrn → mesh DNS, everything else → internet DNS
58|
59|### D11: Video → CDN-only delivery
60|- Video uploads go directly to CDN (never served from home node)
61|- Home node only stores metadata + thumbnail
62|- Users pay CDN provider for video hosting
63|- Text + images served from home node directly
64|
65|### D12: v1/v2/v3 scope → Accepted as proposed
66|- See 00-VISION.md for full scope breakdown