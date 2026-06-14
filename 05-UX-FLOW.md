1|# Sovrn — UX / First-Boot Flow (OOBE)
2|
3|## OOBE WIZARD (First Boot Experience)
4|
5|### Screen 1: Welcome
6|```
7|┌─────────────────────────────────────────────┐
8|│                                             │
9|│        Welcome to [OS Name]                 │
10|│                                             │
11|│   Your machine. Your network. Your data.    │
12|│                                             │
13|│   [Let's Get Started]    [Language ▼]       │
14|│                                             │
15|└─────────────────────────────────────────────┘
16|```
17|- Language selector (default: English)
18|- Brief 3-sentence pitch: private, decentralized, yours
19|
20|### Screen 2: Display Name
21|```
22|┌─────────────────────────────────────────────┐
23|│                                             │
24|│   What should people call you?              │
25|│                                             │
26|│   Display Name: [Alice Sharma          ]    │
27|│                                             │
28|│   This is NOT unique — others can have the  │
29|│   same name. Your unique ID makes you       │
30|│   identifiable.                             │
31|│                                             │
32|│   [Next]                                    │
33|│                                             │
34|└─────────────────────────────────────────────┘
35|```
36|- Display name is changeable later
37|- No uniqueness requirement — identification is by ID, not name
38|
39|### Screen 3: Identity Generated
40|```
41|┌─────────────────────────────────────────────┐
42|│                                             │
43|│   Your identity has been created!            │
44|│                                             │
45|│   ┌─────────┐                               │
46|│   │ QR CODE │     User ID: a7x3k9m2          │
47|│   │         │     Domain: (claim next step)  │
48|│   │         │                               │
49|│   └─────────┘                               │
50|│                                             │
51|│   Share your QR code or User ID so          │
52|│   others can find and follow you.            │
53|│                                             │
54|│   [Next]                                    │
55|│                                             │
56|└─────────────────────────────────────────────┘
57|```
58|- Auto-generated Ed25519 keypair
59|- QR code encodes: user ID + Yggdrasil address + public key
60|- User ID shown in a large, copyable format
61|
62|### Screen 4: Seed Phrase Backup ⚠️ CRITICAL
63|```
64|┌─────────────────────────────────────────────┐
65|│                                             │
66|│   ⚠️  BACK UP YOUR SEED PHASE              │
67|│                                             │
68|│   This is the ONLY way to recover your      │
69|│   identity if your device is lost or        │
70|│   damaged. There is NO password reset.      │
71|│   NO ONE can help you if you lose this.     │
72|│                                             │
73|│   ┌─────────────────────────────────────┐   │
74|│   │ river  forest  sunset  purple  bridge│   │
75|│   │ glacier  amber  thunder  oak  river │   │
76|│   │ pebble  crystal                     │   │
77|│   └─────────────────────────────────────┘   │
78|│                                             │
79|│   [I've written this down]                  │
80|│                                             │
81|│   ⚠️  Verification required:               │
82|│   Enter word #3: [______]                    │
83|│   Enter word #7: [______]                    │
84|│   Enter word #11: [______]                   │
85|│                                             │
86|│   [Verify & Continue]                        │
87|│                                             │
88|└─────────────────────────────────────────────┘
89|```
90|- Must write down seed phrase
91|- Must verify by re-entering 3 random words
92|- Cannot skip or proceed without verification
93|- Option: export to USB drive (encrypted) or print QR code
94|
95|### Screen 5: Claim Your Domain
96|```
97|┌─────────────────────────────────────────────┐
98|│                                             │
99|│   Claim your home on the mesh               │
100|│                                             │
101|│   [    alice    ].sovrn                        │
102|│                                             │
103|│   ✓ alice.sovrn is available!                  │
104|│                                             │
105|│   Your domain is your homepage, your        │
106|│   address, your identity on the mesh.       │
107|│                                             │
108|│   You can register up to 5 domains this     │
109|│   month. (5 remaining)                     │
110|│                                             │
111|│   [Claim]  [Skip — I'll do this later]     │
112|│                                             │
113|└─────────────────────────────────────────────┘
114|```
115|- Real-time availability check against DHT
116|- Shows rate limit remaining
117|- Skip option — can claim domain later from settings
118|
119|### Screen 6: Connect to Mesh
120|```
121|┌─────────────────────────────────────────────┐
122|│                                             │
123|│   Connecting to the mesh...                 │
124|│                                             │
125|│   ┌──────────────────────────────────┐      │
126|│   │  ✓  Mesh network connected       │      │
127|│   │  ✓  Your address: [2001:db8::]   │      │
128|│   │  ⏳ Discovering peers...          │      │
129|│   │  ⏳ Registering alice.sovrn...       │      │
130|│   │  ✓  Domain registered!           │      │
131|│   │  ✓  Services starting...         │      │
132|│   └──────────────────────────────────┘      │
133|│                                             │
134|│   [Enter Your Desktop]                       │
135|│                                             │
136|└─────────────────────────────────────────────┘
137|```
138|- Auto-configures Yggdrasil
139|- Registers domain in DHT
140|- Starts all background services
141|- Shows connection status
142|
143|### Screen 7: Desktop
144|- GNOME desktop loads
145|- Dock/sidebar contains: Feed, Messages, Browser, Files, Settings, Terminal
146|- Welcome notification with tips
147|- All services running in background, zero config needed
148|
149|## Settings App (Post-Setup)
150|
151|### Identity Section
152|- View/change display name
153|- View user ID + QR code
154|- Generate new alias IDs
155|- Manage domain registrations (claim new, view existing, see expiry dates)
156|- Export identity (seed phrase backup, QR code)
157|
158|### Mesh Section
159|- View connection status
160|- View Yggdrasil address
161|- View connected peers
162|- Mesh network statistics
163|
164|### CDN Section
165|- Browse CDN providers (like an app store for storage)
166|- View storage/bandwidth pricing
167|- Subscribe to a CDN provider
168|- View current CDN usage
169|- Push content to CDN
170|
171|### Privacy Section
172|- What data your node is serving
173|- Connected peers log
174|- Encryption status
175|- Alias management
176|- Block/mute lists
177|
178|### Apps Section (Feed, Messages, etc.)
179|- Each is a PWA accessible from the dock
180|- Settings for each app inside the app itself
181|- No separate account creation needed
182|
183|## DAILY USAGE FLOWS
184|
185|### Following Someone
186|1. Scan their QR code OR
187|2. Search their user ID OR
188|3. Search their .sovrn domain
189|4. Click "Follow"
190|5. Their content appears in your feed when they're online
191|
192|### Posting Content
193|1. Open Feed app
194|2. Type text / attach image
195|3. Choose channel: Micro (text), Media (images), Article (long-form)
196|4. Post → signed and stored on your node
197|5. Followers who are online receive it in real-time
198|
199|### Sending a Message
200|1. Open Messages app
201|2. Search or scan for recipient
202|3. Type message → encrypted with recipient's X25519 key
203|4. Sent directly to recipient's node (if online) or queued for delivery
204|
205|### Creating a Homepage
206|1. Open Homepage editor (built-in)
207|2. Visual editor for simple pages (about me, links, portfolio)
208|3. Or upload raw HTML/CSS/JS (your quine philosophy fits here!)
209|4. Save → immediately available at yourdomain.sovrn