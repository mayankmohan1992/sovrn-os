1|# Sovrn — Identity System
2|
3|## OVERVIEW
4|Every user has ONE identity across ALL mesh services. No separate logins. No accounts per service. Your identity is your keypair.
5|
6|## IDENTITY COMPONENTS
7|
8|### 1. Master Keypair
9|- **Signing key:** Ed25519 (for signing events, posts, messages)
10|- **Encryption key:** X25519 (derived from same seed, for encrypted messaging)
11|- **Public key hash:** SHA-256 of Ed25519 public key, truncated to 16 characters = unique user ID
12|- **Example ID:** `a7x3k9m2p8f1q4b5`
13|
14|### 2. Display Name
15|- Chosen by user at setup (e.g., "Alice Sharma")
16|- NOT unique — multiple users can have the same display name
17|- Unique identification is by the user ID, not display name
18|- Changeable at any time
19|
20|### 3. Mesh Domain
21|- Registered during setup (e.g., `alice.sovrn`)
22|- UNIQUE — first come, first served within rate limits
23|- Maps to user's Yggdrasil IPv6 address
24|- This is the user's homepage, email address, and identity on the mesh
25|
26|### 4. Seed Phrase (Recovery)
27|- BIP-39 standard: 12 or 24 English words
28|- Generated from the same entropy as the master keypair
29|- LOSE THIS = LOSE YOUR IDENTITY
30|- Must be written down during setup, enforced by the OOBE wizard
31|
32|### 5. QR Code
33|- Encodes: user ID + Yggdrasil IPv6 address + mesh domain
34|- Scannable by another user to instantly follow/add you
35|- Also encodes the Ed25519 public key for identity verification
36|
37|## ALIAS SYSTEM (Firefox Relay Style)
38|
39|### How It Works
40|- User can generate alias keypairs that delegate to the master keypair
41|- Each alias gets its own short public key hash
42|- Example: User ID `a7x3k9m2p8f1q4b5` creates alias `b3n7m2k1` for a forum, alias `p9q4w8e5` for shopping
43|- Aliases forward all events to the master identity
44|- Aliases are disposable — revoke without affecting master identity
45|
46|### Use Cases
47|- Share an alias at a public event instead of your real ID
48|- Use different aliases for different communities
49|- One compromised alias doesn't expose your master identity
50|- Others following an alias see the same content as following your master ID
51|
52|### Technical Implementation
53|```
54|Master keypair:    Ed25519(sk_m, pk_m) → User ID: a7x3k9m2p8f1q4b5
55|Alias keypair 1:   Ed25519(sk_1, pk_1) → Alias ID: b3n7m2k1
56|Alias keypair 2:   Ed25519(sk_2, pk_2) → Alias ID: p9q4w8e5
57|
58|Delegation record: Signed by master key
59|  "I, a7x3k9m2p8f1q4b5, authorize b3n7m2k1 and p9q4w8e5 as my aliases"
60|
61|When someone follows alias b3n7m2k1:
62|  1. Node looks up delegation record
63|  2. Confirms b3n7m2k1 is authorized by a7x3k9m2p8f1q4b5
64|  3. All content from a7x3k9m2p8f1q4b5 is visible via b3n7m2k1
65|  4. Messages to b3n7m2k1 are delivered to a7x3k9m2p8f1q4b5
66|```
67|
68|## IDENTITY ACROSS SERVICES
69|
70|| Service | Identity Used | Address Format |
71||---|---|---|
72|| Social Feed | Master keypair / aliases | User ID follows: `a7x3k9m2...` |
73|| Messaging | Master keypair | DMs to: `a7x3k9m2...` or `alice.sovrn` |
74|| Homepage | Mesh domain | `alice.sovrn` |
75|| DNS | Mesh domain registration | `alice.sovrn` → Yggdrasil IPv6 |
76|| Email (v2) | Mesh domain | `alice@alice.sovrn` |
77|
78|**No separate logins. No separate accounts. One identity everywhere.**
79|
80|## KEY CEREMONY (Setup Time)
81|
82|### Step 1: Generate
83|- OS generates 256 bits of entropy
84|- Entropy → BIP-39 seed phrase (12 words)
85|- Entropy → Ed25519 master keypair
86|- Entropy → X25519 encryption keypair
87|
88|### Step 2: Display and Verify
89|- Show seed phrase on screen
90|- User MUST write it down
91|- Confirmation: ask user to re-enter 3 random words from the phrase
92|- Cannot proceed without confirmation
93|
94|### Step 3: Export (Optional)
95|- QR code of seed phrase for paper backup
96|- USB drive encrypted backup
97|- Print option (if printer available)
98|
99|### Step 4: Lock
100|- Private keys stored in OS keyring (encrypted with user's login password)
101|- Public keys and user ID written to mesh identity record
102|- Seed phrase is NEVER stored digitally on the machine after this step
103|
104|## KEY ROTATION (Future)
105|- v1: No key rotation. Lose your key = new identity.
106|- v2 (planned): Key rotation with signed transition record. "I, old-key, authorize new-key as my successor."