1|# Sovrn — Vision Document
2|
3|## One-Liner
4|A privacy-first, decentralized operating system where every machine is a node, every user is a host, and no corporation sits between you and your data.
5|
6|## Core Philosophy
7|- **User sovereignty** — Your machine, your data, your rules. No ads, no tracking, no surveillance.
8|- **Zero-config** — Install, pick a name, start posting. No terminals, no server config, no DNS jargon.
9|- **Decentralized by default** — Mesh network, distributed DNS, content on your own hardware.
10|- **Revenue via infrastructure** — CDN storage/pinning for popular content. Never user data.
11|
12|## What The OS Does (User Perspective)
13|1. Install Sovrn, pick a display name → get a unique ID + QR code
14|2. Your machine becomes a node on a private mesh network
15|3. You get: email, DNS, web hosting, social media — all built-in, all self-hosted on YOUR hardware
16|4. You can browse and use the regular internet alongside the mesh network
17|5. You can create a homepage, register a domain on the mesh TLD, post content
18|6. Others follow you, see your posts — content served FROM your machine (CDN for scale)
19|7. Single identity across all services — no separate logins
20|
21|## Key Differentiators vs Existing Solutions
22|| Feature | This OS | Fediverse | Clearnet |
23||---|---|---|---|
24|| Setup | Zero-config | Host a server | Use someone's server |
25|| Data location | Your machine | A server | Someone's cloud |
26|| Identity | Unified DID + aliases | Per-instance account | Per-service account |
27|| DNS | Distributed mesh TLD | Domain name + hosting | ICANN |
28|| Revenue model | CDN storage fees | Donations / none | Ads / data sales |
29|| Encryption | Local military-grade | Varies | Varies |
30|
31|## Status
32|- [x] Vision defined
33|- [ ] Architecture decisions (see 01-ARCHITECTURE.md)
34|- [ ] Component selection (see 02-COMPONENTS.md)
35|- [ ] Network design (see 03-NETWORK.md)
36|- [ ] Identity system (see 04-IDENTITY.md)
37|- [ ] UX / First-boot flow (see 05-UX-FLOW.md)
38|- [ ] Revenue model (see 06-REVENUE.md)
39|- [ ] Open questions (see 07-OPEN-QUESTIONS.md)