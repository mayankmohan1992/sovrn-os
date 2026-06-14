1|# Sovrn — Open Questions Round 2
2|# Deeper technical questions after initial decisions resolved
3|
4|## TECHNICAL VALIDATION ✓
5|All 10 initial decisions are technically feasible. No blocking issues.
6|
7|## DEEPER QUESTIONS — Need Your Input
8|
9|### D1: Presence System — How Does the Network Know You're Online?
10|
11|You said "the network must know when a user is online." This needs a **presence/heartbeat system**:
12|- Your node broadcasts "I'm alive" to the mesh periodically
13|- Followers poll your node for content only when you're confirmed online
14|- When your heartbeat stops, you're marked offline and your content drops from feeds
15|
16|**Options for heartbeat frequency:**
17|- (a) Every 30 seconds — fast detection, but higher battery/data usage
18|- (b) Every 2 minutes — reasonable balance
19|- (c) Every 5 minutes — slow detection, but very efficient
20|
21|**Also:** What about laptops that sleep? When you close the lid, should Sovrn broadcast "going offline" so followers see you disappear immediately? Or just let the heartbeat expire?
22|
23|**My take:** 2-minute heartbeat + explicit "going offline" signal when shutting down/sleeping. This means followers see you disappear within 2 minutes of you going offline unexpectedly, or instantly if you shut down cleanly.
24|
25|---
26|
27|### D2: The "500 Follows" Problem
28|
29|If Alice follows 500 people, her node needs to check: are these 500 nodes online? Then fetch content from the ones that are. This is manageable with:
30|- Presence aggregation (track who's online via mesh gossip, not individual polling)
31|- Content caching (fetch content once, display from cache)
32|
33|**Question:** Should there be a follow limit? Or let users follow unlimited people and Sovrn just handles it?
34|**My take:** No hard cap, but Sovrn should warn you if follow count degrades performance. Realistically most people follow 50-200.
35|
36|---
37|
38|### D3: The Cold Start Problem
39|
40|Day 1: You install Sovrn. You're the only node. Your feed is empty. Your mesh domain resolves only to yourself. This is lonely and useless.
41|
42|**Solutions:**
43|- (a) **Bootstrap directory** — A curated list of official nodes (run by the project) that provide initial content: news, guides, community
44|- (b) **Internet bridge** — The OS shows regular internet content alongside mesh content until the mesh is populated
45|- (c) **Pre-seed popular content** — The OS image comes with cached content from early adopters/community
46|- (d) **Local-first content** — The OS ships with useful local tools (notes, calculator, media player, code editor) so it's useful even with zero mesh connections
47|
48|**My take:** (a) + (d). Ship official bootstrap nodes (the project team runs a few nodes with community content, documentation, and a "welcome to the mesh" portal). But also make Sovrn incredibly useful offline — all core apps (file manager, text editor, terminal, media tools) work without any network.
49|
50|---
51|
52|### D4: Decentralized CDN — How Does Payment Work?
53|
54|You said "anyone can run a CDN node, users pay the CDN provider." This means:
55|- CDN Node Operator A charges ₹100/GB
56|- CDN Node Operator B charges ₹80/GB
57|- Users choose a CDN node (or Sovrn auto-selects cheapest/fastest)
58|
59|**Questions:**
60|- **Payment method:** Cryptocurrency (privacy-preserving, borderless)? UPI/card (accessible, familiar)? Both?
61|- **Trust model:** How does the user know the CDN node isn't tampering with content? (Answer: content is content-addressed and signed — tampering would be detected by hash verification. CDN can't read or modify encrypted content.)
62|- **Discovery:** How do users find CDN nodes? A mesh directory? Something built into Sovrn?
63|- **Your revenue:** Do you take a cut of CDN transactions? Or is your revenue purely from running your own CDN nodes?
64|
65|**My take:** Accept both crypto and traditional payments. Built-in CDN directory in Sovrn settings. You make money by running the largest/most reliable CDN nodes. Marketplace takes 5-10% facilitation fee optionally.
66|
67|---
68|
69|### D5: Spam and Abuse on a Chronological Feed
70|
71|Pure chronological, no algorithm. This means: if someone you follow spams 100 posts per hour, you see 100 posts per hour. There's no algorithm to filter them.
72|
73|**Proposed tools for users (not algorithmic, user-controlled):**
74|- **Mute** — Stop seeing someone's posts without unfollowing
75|- **Block** — They can't see your content, you can't see theirs
76|- **Rate limit indicator** — The OS shows you "This user posts X times per hour" so you can make informed follow decisions
77|- **Post frequency limit** — You can set a personal rule: "Don't show more than 5 posts per hour from any single user"
78|
79|Is this sufficient? Or do you want ANY anti-spam mechanism in the protocol itself (e.g., the mesh rejects more than 10 posts per hour from a single node)?
80|
81|**My take:** User-controlled tools only. No protocol-level rate limiting — that contradicts "your node, your rules." Give users the mute/block/rate-limit tools to curate their own feed.
82|
83|---
84|
85|### D6: Content Types — What Can You Post?
86|
87|You mentioned clones of Twitter, YouTube, Instagram, Substack. Are these:
88|- **(a) Separate apps** — A "Twitter clone" app, a "YouTube clone" app, etc. Each with its own UI.
89|- **(b) One unified app** — You follow people, you see their posts. Text, images, video, long-form — all in one feed. The UI adapts based on content type.
90|- **(c) Channels** — Like Discord channels or Reddit subreddits. You can post in "Micro" (Twitter-like), "Media" (Instagram-like), "Video" (YouTube-like), "Longform" (Substack-like). But it's all one identity.
91|
92|**My take:** (c). One identity, multiple "channels" or "modes." This matches your "one login, one identity" vision. The same app handles all content types, but you can choose to view only "short posts" or only "videos" or only "articles." The content type is a field in the signed event (like Nostr event kinds).
93|
94|This also solves your CDN problem naturally — video and image channels are CDN candidates, text posts almost never need CDN.
95|
96|---
97|
98|### D7: Group Chat and Communities
99|
100|Mesh messaging covers DMs. But what about:
101|- **Group chats** — Multiple people in one conversation (like Signal groups)
102|- **Communities** — A topic-based space where multiple people post (like Reddit subreddits or Discord servers)
103|- **Events** — Temporary groups (like a watch party or live discussion)
104|
105|Do you want any of these in v1, or is DMs + public feed enough?
106|
107|**My take:** DMs + public feed for v1. Group chat in v2. Communities in v3. Don't overbuild the first version.
108|
109|---
110|
111|### D8: OS Updates — How Does Security Patching Work?
112|
113|An OS that never updates becomes insecure. But updates need to come from somewhere trusted.
114|
115|**Options:**
116|- (a) **Central update server** — We run a Debian mirror + our own repo. Users get updates from us. Simple, trusted, but centralized.
117|- (b) **Mesh-distributed updates** — Updates propagate through the mesh network (like BitTorrent). Decentralized, but how do you verify authenticity?
118|- (c) **Hybrid** — Metadata (package lists, checksums) from our server. Actual packages via mesh P2P. Signed and verified.
119|
120|**My take:** (c). The package metadata comes from our signed repo (centralized, trusted). The actual package downloads can come from mesh peers who already have the update (like apt-p2p). Best of both worlds — trusted provenance, decentralized bandwidth.
121|
122|---
123|
124|### D9: Hardware Targets
125|
126|What hardware should Sovrn run on?
127|- **v1:** x86_64 laptops and desktops (only)
128|- **v2:** ARM (Raspberry Pi 4/5) — natural "always-on node"
129|- **v3:** ARM laptops (Apple Silicon? Unlikely due to closed ecosystem)
130|
131|Should we officially support Raspberry Pi as an "always-on companion node" from day 1? Like: "Install Sovrn on your Pi, leave it running 24/7, and your content is always available on the mesh." This directly solves the offline problem.
132|
133|**My take:** x86_64 only for v1. Pi support in v2. But design the architecture so Pi support is a smooth addition, not a rewrite.
134|
135|---
136|
137|### D10: The .sovrn TLD — Mesh-Only
138|
139|Just confirming: `alice.sovrn` resolves **only** on the mesh network. If someone types `alice.sovrn` in a regular browser on regular internet, it won't work. This is correct, yes?
140|
141|And on Sovrn, you have TWO addresses:
142|- `alice.sovrn` → your mesh homepage (served from your node)
143|- You can also browse regular internet sites (Chrome/Firefox work normally)
144|
145|Both networks coexist. The OS has a mesh-aware browser that auto-routes .sovrn to mesh and everything else to internet.
146|
147|**My take:** One browser, auto-routing. `.sovrn` goes to mesh DNS, everything else goes to regular internet DNS. Seamless user experience.
148|
149|---
150|
151|### D11: Video Content — Realistic Constraints
152|
153|You want YouTube-like functionality. Video takes MASSIVE storage and bandwidth:
154|- A 10-minute 1080p video = ~500MB
155|- Serving 100 views from your home internet = 50GB bandwidth
156|- Home upload speed in India (Jaipur): typically 10-50 Mbps
157|
158|**Reality check:** A home node in Jaipur cannot serve YouTube-scale video. Even with CDN, the first upload comes from your machine.
159|
160|**Options:**
161|- (a) Lower quality default — videos default to 720p, with options to upload higher. Users choose quality vs. bandwidth tradeoff.
162|- (b) CDN-priority for video — When you upload a video, Sovrn automatically pushes it to your CDN provider so your node doesn't serve it directly. Only metadata (title, thumbnail, description) stays on your node.
163|- (c) Chunked/streaming — Videos stream in chunks. CDN caches popular chunks. Like PeerTube's WebTorrent approach.
164|- (d) Time-limited local cache — Followers auto-cache content they've seen. Popular videos spread across follower nodes (swarm distribution).
165|
166|**My take:** (b) + (d). Video uploads go directly to CDN (user pays for this, or free tier with bandwidth limits). Swarm distribution among followers for popular content. Your node only serves text and thumbnails directly.
167|
168|---
169|
170|### D12: What's NOT in v1?
171|
172|To ship v1, we need to cut scope. Here's my proposed v1 vs. v2+ split:
173|
174|**v1 (Minimum Viable Product):**
175|- Hardened Debian base with GNOME desktop
176|- Yggdrasil mesh networking (auto-configured, zero-setup)
177|- Identity system (keypair + seed phrase + unique ID + QR code)
178|- Mesh DNS (.sovrn TLD, DHT-based)
179|- Mesh messaging (DMs, text only)
180|- Social feed (text + images, chronological, follow/unfollow, mute/block)
181|- Homepage hosting (static site on your node)
182|- CDN integration (connect to CDN nodes for paid storage)
183|- OOBE wizard (setup flow, domain claim, seed phrase backup)
184|- Regular internet access alongside mesh
185|
186|**v2:**
187|- Video support in social feed
188|- Group messaging
189|- Email (mesh-native messaging with SMTP gateway)
190|- Raspberry Pi support
191|- Community/channels feature
192|
193|**v3:**
194|- Full YouTube-like video with streaming
195|- Substack-like long-form articles
196|- Voice messages
197|- Advanced CDN marketplace
198|- Mobile companion app (viewer, not full node)
199|
200|Does this feel right? What would you add or remove from v1?
201|
202|---
203|
204|## ARCHITECTURE DIAGRAM (Updated v1)
205|
206|```
207|┌─────────────────────────────────────────────────────────────┐
208|│                    SOVRN v1                         │
209|├─────────────────────────────────────────────────────────────┤
210|│                                                             │
211|│  ┌─────────────┐                                            │
212|│  │  OOBE       │  Setup: Name → Key → Seed Phrase →        │
213|│  │  Wizard     │  Domain Claim → QR Code → Desktop         │
214|│  └─────────────┘                                            │
215|│                                                             │
216|│  ┌──────────────────────────────────────────────────────┐   │
217|│  │               GNOME DESKTOP (stripped, customized)  │   │
218|│  │                                                      │   │
219|│  │   ┌─────────┐  ┌─────────┐  ┌─────────┐             │   │
220|│  │   │  Feed   │  │  Chat   │  │ Browser │             │   │
221|│  │   │  (web)  │  │  (web)  │  │ (1 browser,            │   │
222|│  │   │         │  │         │  │  2 networks)            │   │
223|│  │   └────┬────┘  └────┬────┘  └────┬────┘             │   │
224|│  └───────┼─────────────┼────────────┼───────────────────┘   │
225|│          │             │            │                        │
226|│  ┌───────┴─────────────┴────────────┴───────────────────┐   │
227|│  │            LOCAL NODE SERVICES (systemd)             │   │
228|│  │                                                      │   │
229|│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐ │   │
230|│  │  │Mesh  │ │ DNS  │ │ Web  │ │Social │ │ Identity │ │   │
231|│  │  │Msg   │ │Server│ │Host  │ │Feed   │ │ Manager  │ │   │
232|│  │  └──────┘ └──────┘ └──────┘ └──────┘ └──────────┘ │   │
233|│  └──────────────────────┬─────────────────────────────┘   │
234|│                         │                                 │
235|│  ┌──────────────────────┴─────────────────────────────┐   │
236|│  │             YGGDRASIL MESH (ygg0)                  │   │
237|│  │  Encrypted E2E • Auto-configured • VPN overlay     │   │
238|│  └──────────────────────┬─────────────────────────────┘   │
239|│                         │                                 │
240|│  ┌──────────────────────┴─────────────────────────────┐   │
241|│  │          DEBIAN HARDENED BASE (kernel + systemd)    │   │
242|│  └──────────────────────┬─────────────────────────────┘   │
243|│                         │                                 │
244|│  ┌──────────────────────┴─────────────────────────────┐   │
245|│  │  HARDWARE                                            │   │
246|│  │  ┌──────────────┐  ┌──────────────┐                │   │
247|│  │  │ Regular      │  │ Mesh         │                │   │
248|│  │  │ Internet     │  │ Network      │                │   │
249|│  │  │ (eth0/wlan0) │  │ (ygg0)       │                │   │
250|│  │  └──────────────┘  └──────────────┘                │   │
251|│  └──────────────────────────────────────────────────────┘   │
252|│                                                             │
253|│  ┌──────────────────────────────────────────────────────┐   │
254|│  │  REGULAR LINUX APPS (snap/flatpak/deb) — sandboxed  │   │
255|│  └──────────────────────────────────────────────────────┘   │
256|│                                                             │
257|│  ┌──────────────────────────────────────────────────────┐   │
258|│  │  ENCRYPTION LAYERS                                  │   │
259|│  │  Disk: LUKS2  |  Network: Yggdrasil  |  Content: age│   │
260|│  │  Identity: Ed25519  |  Events: Signed + encrypted  │   │
261|│  └──────────────────────────────────────────────────────┘   │
262|│                                                             │
263|└─────────────────────────────────────────────────────────────┘
264|
265|CDN LAYER (external, decentralized):
266|┌──────────────────────────────────────────────────────┐
267|│  CDN Node A  │  CDN Node B  │  CDN Node C  │  ...    │
268|│  (run by us) │  (run by anyone)│ (run by anyone)    │
269|│                                                              │
270|│  Users pay CDN operators for storage + bandwidth.            │
271|│  Content is encrypted — CDN cannot read it.                  │
272|│  Content-addressed — tampering detected by hash mismatch.    │
273|└──────────────────────────────────────────────────────────────┘