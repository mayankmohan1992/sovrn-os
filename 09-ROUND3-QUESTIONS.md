1|# Sovrn — Round 3 Deep Questions
2|
3|## Answers resolved: D1-D12 all accepted. See 01-ARCHITECTURE.md.
4|## Key decisions confirmed: No follow cap (architecture handles it), secure updates (6-layer system), .sovrn TLD mesh-only, both Chromium and Firefox browsers.
5|
6|---
7|
8|## E1: Browser — Are We Forking or Bundling?
9|
10|You said "Chromium as well as Firefox-based browser for wider compatibility." Two questions:
11|
12|**Option A: Bundle stock browsers + mesh extension**
13|- Ship stock Firefox and Chromium with a mesh extension/plugin
14|- The extension handles .sovrn DNS routing, mesh identity integration, and feed rendering
15|- Pros: Easier to maintain, automatic browser updates, familiar UX
16|- Cons: Extension could be disabled/removed, two separate codebases for mesh functionality
17|
18|**Option B: Fork Firefox (like Tor Browser forks Firefox)**
19|- Fork Firefox ESR, modify the source to integrate mesh DNS, identity, etc.
20|- Ship as "Mesh Browser" alongside a stock Chromium for regular internet
21|- Pros: Deep integration, can't accidentally disable mesh features, consistent UX
22|- Cons: Maintaining a Firefox fork is significant ongoing work
23|
24|**Option C: Stock browsers + local web apps (PWA)**
25|- Ship stock browsers unchanged
26|- All mesh services (Feed, Messages, Homepage editor, Settings) are LOCAL PWAs served from the node
27|- User opens their Mesh App (a PWA at `https://sovrn.local/` or dev: `localhost:54772`) which handles everything
28|- The browser is just a rendering engine — the mesh functionality is in the PWA, not the browser
29|
30|**My take:** Option C. This aligns perfectly with your self-contained HTML philosophy. The browser is just a viewer. All the mesh magic is in the local PWA. Stock Firefox + Chromium work fine. No fork needed. If a user removes the mesh extension or uses a different browser, the PWA still works at `https://sovrn.local/`.
31|
32|This also means your social feed, messages, homepage editor — all of it is built with HTML/CSS/JS. Your wheelhouse.
33|
34|---
35|
36|## E2: Service Architecture — How Do All the Background Services Run?
37|
38|Your OS needs these services running in the background:
39|1. **Yggdrasil** — mesh network
40|2. **Mesh DNS** — DHT resolver + registrar
41|3. **Web hosting** — serve your homepage
42|4. **Social feed** — store and serve your posts + fetch from followees
43|5. **Messaging** — receive and queue DMs
44|6. **Identity** — key management, alias delegation
45|7. **Presence** — heartbeat broadcasting
46|8. **CDN agent** — push/pull content to/from CDN nodes
47|
48|**How do we manage all these?**
49|
50|**Option A: Individual systemd services** — Each service is a separate systemd unit. Standard, debuggable, but 8+ services to manage.
51|
52|**Option B: Single monolithic daemon** — One "meshd" process that handles everything. Simpler to manage, but one crash takes down all services.
53|
54|**Option C: Core services (systemd) + PWA layer (web)** — Critical networking (Yggdrasil, DNS, identity, presence) run as systemd services. All user-facing functionality (social feed, messages, homepage) is in the local PWA served by a small web server (Caddy). The PWA talks to systemd services via a local API.
55|
56|**My take:** Option C. The networking/identity layer MUST be a daemon (it runs even when the user isn't looking at it — presence, message queuing). But the UI layer is a PWA. This is your quine architecture applied at Sovrn level — a self-contained local web app that IS the mesh interface.
57|
58|```
59|┌─────────────────────────────────────┐
60|│  MESHDAEMON (systemd services)      │
61|│  ┌──────────┐ ┌──────────┐        │
62|│  │ yggdrasil│ │ dns-dht  │        │
63|│  └──────────┘ └──────────┘        │
64|│  ┌──────────┐ ┌──────────┐        │
65|│  │ identity │ │ presence │        │
66|│  └──────────┘ └──────────┘        │
67|│  ┌──────────┐ ┌──────────┐        │
68|│  │ message-q│ │ cdn-agent│        │
69|│  └──────────┘ └──────────┘        │
70|└──────────────┬──────────────────────┘
71|               │ Local HTTP API
72|               │ (localhost:54771/api)
73|┌──────────────▼──────────────────────┐
74|│  LOCAL WEB SERVER (Caddy)           │
75|│  ┌──────────────────────────────────┤
76|│  │ PWA: Feed | Messages | Homepage  │
77|│  │ Settings | Identity | CDN Config │
78|│  │ (Vanilla HTML/CSS/JS)           │
79|│  └─────────────────────────────────┤
80|└──────────────────────────────────────┘
81|```
82|
83|The PWA makes API calls to sovrn.local/api/* which talk to the systemd services. All mesh logic is in the daemons. All UI is in the PWA. Clean separation.
84|
85|---
86|
87|## E3: Discovery — How Do People Find Each Other?
88|
89|You can follow someone by QR code or user ID. But how does someone NEW on the mesh discover content? If you don't know anyone's ID, your feed is empty.
90|
91|**Options:**
92|- (a) **Bootstrap directory** — Project-run nodes that host a "discover" page showing popular/public accounts
93|- (b) **Web of trust** — When you follow someone, you see who THEY follow (transitive discovery)
94|- (c) **Hashtags/topics** — Search by topic (like Twitter hashtags), results pulled from the mesh DHT
95|- (d) **Onion routing directory** — Like Tor's hidden service directory, but for mesh profiles
96|- (e) **All of the above** — Multiple discovery mechanisms
97|
98|**My take:** (e) All of the above. Discovery should be easy and multi-path:
99|1. QR code / user ID sharing (direct)
100|2. Web of trust ("People you follow also follow...") (social)
101|3. Hashtag/topic search (content-based)
102|4. Bootstrap directory (curated, project-run) (onboarding)
103|5. Local network discovery (see who's on your WiFi/LAN) (proximity)
104|
105|---
106|
107|## E4: What About Content You've Already Seen?
108|
109|When Alice follows Bob and sees his posts, those posts are cached on Alice's machine. What happens to that data?
110|
111|**Questions:**
112|- Should Alice be able to delete cached content from her local storage? (Yes, obviously)
113|- Should Alice be able to export/archive posts she's seen? (Like a personal archive)
114|- Should there be a "save" feature — bookmark posts that persist even if the original author deletes them?
115|- What about Bob deleting a post — does it disappear from Alice's cache too?
116|
117|**My take:**
118|- Users can always clear their cache (like browser cache clear)
119|- Users can "save" posts — saved posts persist locally even if author deletes original
120|- Author deletion sends a "delete event" to followers, but only clears the author's copy and the CDN copy. Followers who saved it keep their copy.
121|- This aligns with "your machine, your data" — if it's on YOUR machine, YOU decide.
122|
123|---
124|
125|## E5: Media Storage — Where Do Images Live?
126|
127|When Alice posts an image:
128|1. Image is stored on Alice's node (local)
129|2. Image reference (CID) is in the event
130|3. Followers fetch the image from Alice's node (if online) or CDN (if Alice has CDN)
131|
132|**Questions:**
133|- How much local storage for media? SSDs are big, but is there a limit?
134|- Should Sovrn auto-manage media cache (delete old, keep recent)?
135|- What's the maximum image/video size per post?
136|
137|**My take:**
138|- No hard limit per post (let the user decide how much of their own storage to use)
139|- Local cache auto-prunes after 90 days for viewed content (configurable)
140|- "Saved" items are never auto-pruned
141|- CDN has its own storage limits based on what the user pays for
142|- A 100 MB per post soft limit (with a warning), hard limit of 500 MB per post for non-video content
143|
144|---
145|
146|## E6: Search — Can You Search the Mesh?
147|
148|If Alice wants to find "photography" content across the mesh, how?
149|
150|**Approach:**
151|- Each node publishes a public profile with hashtags/interests
152|- Search queries are broadcast to the mesh DHT
153|- Nodes that match the query respond (opt-in — nodes can choose not to respond to searches)
154|- No centralized search index
155|- Search results are ranked by: proximity (closer nodes first), recency, social distance (follows-of-follows)
156|
157|**Privacy concern:** Search reveals that you exist and have content matching a query. Solution: search is OPT-IN. Users explicitly choose "make my content searchable" in settings. Default: OFF for posts, ON for public homepage.
158|
159|---
160|
161|## E7: Editing and Deleting Posts
162|
163|Can users edit posts after publishing? On the mesh, edits need to propagate.
164|
165|**Approach:**
166|- Edits are new events that reference the original event
167|- Edit events are signed with the same key as the original
168|- Followers see the latest version (edit events replace original in feed)
169|- Edit history is preserved (you can see previous versions if you want — like Wikipedia)
170|- Deletion is a "delete event" — signals to followers and CDN that the author wants this removed
171|- Deletion from YOUR node is immediate. From CDN: CDN respects delete events. From followers: depends on their cache settings.
172|
173|---
174|
175|## E8: What Happens When You Lose Your Machine?
176|
177|Your laptop is stolen. You have your seed phrase. You get a new laptop, install Sovrn, enter seed phrase. Now what?
178|
179|**This is the disaster recovery scenario:**
180|1. You enter seed phrase → identity keys are regenerated → same user ID
181|2. Your .sovrn domain — still registered to your ID → you reclaim it (seed phrase proves ownership)
182|3. Your homepage — GONE. It was on your old machine. Need to rebuild or restore from backup.
183|4. Your social posts — GONE from YOUR node (they were on the old machine). But they may exist on:
184|   - Followers' caches (whoever saved/hasn't-pruned your posts)
185|   - CDN (if you had CDN)
186|5. Your messages — Queued on other nodes (if they were trying to reach you while offline)
187|6. Your follow list — GONE. You need to re-follow everyone.
188|
189|**Questions:**
190|- Should Sovrn automatically back up your follow list, bookmarks, and settings to an encrypted store on the mesh? (Like a decentralized backup)
191|- Should we offer encrypted cloud backup (defeats "no cloud" but is pragmatic)?
192|- Should "your posts" be recoverable from followers' caches? (This is a privacy question — can followers reshare your content after you've lost it?)
193|
194|**My take:**
195|- Encrypted backup of metadata (follow list, settings, bookmarks) to mesh DHT — stored across multiple nodes, encrypted with your key, recoverable with seed phrase
196|- Homepage content: user's responsibility (suggest backup to USB or CDN)
197|- Posts: recoverable from followers' caches ONLY if the author's identity matches. Followers can re-broadcast your posts back to you when you come back online. This is like BitTorrent — other nodes have the pieces.
198|
199|---
200|
201|## E9: The Name of Sovrn
202|
203|We've been calling it "Sovrn" as a placeholder. Do you have a real name in mind?
204|
205|Some options inspired by the vision:
206|- **Meshter** (mesh + master) — you own the mesh
207|- **Sovrn** (sovereign) — you are sovereign over your data
208|- **Nocha** (no chains) — no corporate chains
209|- **Ather** (from Sanskrit — the arrow that strikes without being seen)
210|- **Vibhu** (Sanskrit — all-pervading, omnipresent)
211|- **Pratibha** (Sanskrit — intelligence, light)
212|- Or anything you want. This is your project, your name.
213|
214|---
215|
216|## E10: What's the Build Pipeline?
217|
218|Eventually we need to actually BUILD this OS. How?
219|
220|**The OS build pipeline (for later, but worth planning now):**
221|
222|1. **Base image:** Start with Debian testing (netinst)
223|2. **Hardening:** Apply KSPP kernel config, AppArmor profiles, remove unnecessary packages
224|3. **Mesh layer:** Install and configure Yggdrasil, our mesh services
225|4. **Desktop:** Install GNOME, strip unnecessary apps, apply custom theme
226|5. **PWA apps:** Install Caddy + our local PWA bundle
227|6. **OOBE:** Install our custom setup wizard
228|7. **Image output:** Generate a bootable ISO / USB image
229|
230|**Build tools:**
231|- `debos` — Debian OS image builder (used by many embedded Linux projects)
232|- `live-build` — Debian live system builder (used by Ubuntu, Kali, etc.)
233|- `ansible` or custom shell scripts — for post-install configuration
234|- GitHub Actions or local build server — for CI/CD
235|
236|**Release channels:**
237|- **Stable:** Thoroughly tested, recommended for all users
238|- **Testing:** New features, may have bugs
239|- **Unstable:** Bleeding edge, for developers
240|
241|This is for later, but knowing how we'll build it affects architecture decisions now. For example, we need to ensure all our custom services are packaged as .deb files so they integrate cleanly with APT.
242|
243|---
244|
## E11: Cloudflare ICANN Conflict for .sovrn

One practical concern: `.sovrn` is not currently an ICANN TLD, but it could be registered in the future. If ICANN sells `.sovrn` to someone (like they did with `.dev`, `.app`, etc.), regular internet users typing `alice.sovrn` in a regular browser would go to the ICANN `.sovrn` domain, not our mesh domain.

**Solutions:**
- (a) **Accept the risk** — `.sovrn` only works on our mesh. Regular internet users can't access it anyway. If ICANN creates `.sovrn`, it's a different namespace with different content.
- (b) **Use a different TLD** — Choose something less likely to be claimed by ICANN (like `.meshos`, `.p2p`, or a namespace like `sovrn://alice` instead of `alice.sovrn`)
- (c) **Register .sovrn with ICANN ourselves** — Expensive (millions of dollars), not practical for v1
253|
254|**My take:** (a) for now. Our `.sovrn` resolves via our DHT, not ICANN DNS. There's no conflict because regular internet DNS and our mesh DNS are completely separate systems. If ICANN allocates `.sovrn` in the future, the regular internet `.sovrn` and our mesh `.sovrn` simply coexist in different namespaces. But we should add a note about this risk.