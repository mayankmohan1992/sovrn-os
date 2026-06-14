1|# Sovrn — Secure Update System
2|
3|## THREAT MODEL
4|
5|Supply chain attacks (like the CCleaner incident) happen when:
6|1. Build servers are compromised → modified binary pushed to users
7|2. Package repositories are compromised → malicious packages distributed
8|3. Man-in-the-middle on downloads → user receives tampered package
9|4. Developer keys are stolen → attacker signs malicious packages as "official"
10|
11|Our update system must prevent ALL of these.
12|
13|## ARCHITECTURE
14|
15|### Layer 1: GPG-Signed Repository Metadata (Trust Foundation)
16|
17|Debian's APT already has a strong security model. We harden it:
18|
19|- **Release files** signed by TWO independent GPG keys (requires compromise of BOTH keys to forge a package)
20|  - Key A: Project release signing key (stored in hardware security module)
21|  - Key B: Community witness key (held by a trusted community member, rotated annually)
22|- **Package hashes** (SHA-512) listed in signed metadata
23|- **Metadata includes a timestamp** — prevents replay attacks (old, valid-but-superseded metadata can't be served instead of current)
24|
25|How this prevents CCleaner-style attacks:
26|- Even if our build server is compromised, the attacker CANNOT sign packages without both GPG keys
27|- The build server NEVER has access to the signing keys
28|- Signing happens on an AIR-GAPPED machine: build → transfer hash manifest to air-gapped machine → sign → publish signature
29|
30|### Layer 2: Reproducible Builds (Verification)
31|
32|Anyone can verify that the published binary matches the source code:
33|
34|- Every package is built deterministically (same source + same build environment = same binary hash)
35|- Build instructions published alongside source code
36|- Community members can rebuild packages independently and compare hashes
37|- If ANYONE gets a different hash, it's flagged as a potential compromise
38|
39|How this prevents build-server attacks:
40|- Build server cannot inject malicious code because the hash would differ from community-rebuilt versions
41|- Two or more independent build environments must produce identical binaries
42|- Like what Tor Project and Debian already do for their critical packages
43|
44|### Layer 3: HTTPS + Certificate Pinning (Transport Security)
45|
46|- ALL package downloads happen over HTTPS with certificate pinning
47|- The OS ships with pinned certificates for our repository servers
48|- Prevents MITM attacks on the download path
49|- Even if a CDN node is compromised, it cannot serve tampered packages because:
50|  - The package hash is in the SIGNED metadata
51|  - APT verifies the hash AFTER download
52|  - A tampered package = hash mismatch = rejected
53|
54|### Layer 4: P2P Seeding (Bandwidth, NOT Trust)
55|
56|- We run an official apt-p2p or torrent-based seeding system
57|- Peers can download packages from OTHER users who already have the update
58|- **P2P is for bandwidth only — trust comes from GPG signatures, not from the peer**
59|- Downloaded package hash is ALWAYS verified against the signed metadata from Layer 1
60|- A malicious peer can't serve a tampered package — the hash check will fail
61|
62|How this works:
63|```
64|1. OS checks for updates → downloads signed metadata from our repository (over HTTPS, cert-pinned)
65|2. OS verifies metadata signature (both GPG keys required)
66|3. For each package to update:
67|   a. Check if any mesh peers have this package (P2P) — faster
68|   b. If no peers, download from our repository (HTTPS) — reliable fallback
69|   c. VERIFY package hash against signed metadata (MANDATORY, cannot skip)
70|   d. If hash matches → install. If hash doesn't match → REJECT and report.
71|```
72|
73|### Layer 5: Automated Security Audits
74|
75|- All package updates are automatically scanned with:
76|  - `lintian` (Debian package checker)
77|  - ClamAV (malware scanner)
78|  - Custom scripts checking for unexpected binary changes
79|- Delta updates (only download changes, not full packages) — reduces attack surface AND bandwidth
80|- Update metadata includes expected package SIZE — prevents truncation attacks
81|
82|### Layer 6: Boot Integrity
83|
84|- UEFI Secure Boot enabled by default
85|- Kernel and initramfs signed with our key
86|- `dm-verity` on read-only system partitions (kernel, system libraries)
87|- Boot process verifies signatures before executing
88|- If ANY system file is tampered, boot fails with a clear warning
89|
90|## COMPLETE UPDATE FLOW
91|
92|```
93|User clicks "Update Available" or scheduled auto-update triggers:
94|
95|1. ┌─────────────────────────────────────────────────┐
96|   │  FETCH METADATA                                 │
97|   │  Download Release/InRelease from repo (HTTPS)   │
98|   │  Verify GPG signature (TWO keys required)        │
99|   │  Verify timestamp (not a replay attack)          │
100|   └─────────────────────┬───────────────────────────┘
101|                         │
102|2. ┌─────────────────────▼───────────────────────────┐
103|   │  RESOLVE PACKAGES                               │
104|   │  Compare installed versions vs available          │
105|   │  Generate list of packages to update             │
106|   └─────────────────────┬───────────────────────────┘
107|                         │
108|3. ┌─────────────────────▼───────────────────────────┐
109|   │  DOWNLOAD PACKAGES                              │
110|   │  Try P2P (mesh peers) first                     │
111|   │  Fallback to HTTPS repository                   │
112|   │  Delta updates where possible                   │
113|   └─────────────────────┬───────────────────────────┘
114|                         │
115|4. ┌─────────────────────▼───────────────────────────┐
116|   │  VERIFY PACKAGES                                │
117|   │  Calculate SHA-512 hash of each downloaded .deb  │
118|   │  Compare against hash in signed metadata         │
119|   │  ANY mismatch → REJECT entire update + alert      │
120|   │  Also check: package size, GPG signature on .deb  │
121|   └─────────────────────┬───────────────────────────┘
122|                         │
123|5. ┌─────────────────────▼───────────────────────────┐
124|   │  INSTALL                                        │
125|   │  APT installs verified packages                  │
126|   │  dm-verity verifies system partition changes     │
127|   │  If installation fails → automatic rollback      │
128|   └─────────────────────┬───────────────────────────┘
129|                         │
130|6. ┌─────────────────────▼───────────────────────────┐
131|   │  QUARANTINE CHECK (optional, paranoid mode)     │
132|   │   Run new binaries in sandbox for 30 seconds     │
133|   │   Check for unexpected network connections        │
134|   │   Check for unexpected file system writes         │
135|   │   If suspicious → block + report                  │
136|   └─────────────────────────────────────────────────┘
137|```
138|
139|## WHY THIS PREVENTS CCLEANER-STYLE ATTACKS
140|
141|| Attack Vector | CCleaner Attack | Our Defense |
142||---|---|---|
143|| Compromised build server | Injected malicious code into binary | Dual GPG signing on air-gapped machine; build server has no signing keys |
144|| No source verification | Users can't verify binary matches source | Reproducible builds — anyone can rebuild and compare hashes |
145|| MITM on download | Attacker intercepts download, serves modified binary | HTTPS + cert pinning + hash verification against SIGNED metadata |
146|| Stolen signing key | Attacker signs malicious package with one key | TWO keys required — attacker needs BOTH keys (one on air-gapped machine, one held by community member) |
147|| Malicious P2P peer | Malicious peer serves tampered package | P2P is for bandwidth only; all packages verified against signed metadata hash |
148|| Supply chain via dependency | Upstream dependency compromised | Reproducible builds catch unexpected changes; delta updates reduce scope |
149|
150|## ADDITIONAL SECURITY MEASURES
151|
152|- **Rate limiting on repository:** Prevents DDoS on our update servers
153|- **Mirror network:** Multiple geographic mirrors for redundancy
154|- **Community mirrors:** Anyone can run a mirror (signed, verified by users)
155|- **Automatic rollback:** If update breaks the system, APT automatically reverts to previous version
156|- **Staged rollouts:** Updates pushed to 1% of users first, then 10%, then 100% (like Chrome). Any issues caught early before reaching everyone.
157|- **Immutable base system:** System partition is read-only (dm-verity). Only `/var`, `/home`, and `/tmp` are writable. Makes it much harder for malware to persist.