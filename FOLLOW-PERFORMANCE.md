1|# Sovrn — Follow Count Performance (No Cap Architecture)
2|
3|## WHY FOLLOWS CAN DEGRADE PERFORMANCE
4|
5|When Alice follows 500 people, her node must:
6|
7|1. **Presence tracking** — Know which of those 500 are online right now
8|2. **Content fetching** — Pull new posts from online nodes
9|3. **Storage** — Store fetched content locally for fast feed rendering
10|4. **Processing** — Sort, filter, and render potentially thousands of posts
11|
12|Each of these has a cost. Here's how we eliminate the cap:
13|
14|## SOLUTION: EFFICIENT FOLLOW ARCHITECTURE
15|
16|### 1. Presence Tracking — Gossip Protocol (Not Individual Polling)
17|
18|**Problem:** Checking 500 nodes individually every 2 minutes = 250 polls/second at worst. Not happening.
19|
20|**Solution:** Yggdrasil already uses a gossip/multicast protocol for routing. We overlay presence on top:
21|
22|- Each node broadcasts "I'm online" to the mesh via multicast (sent ONCE, received by ALL interested nodes)
23|- Nodes DON'T poll each followee individually
24|- Instead, they subscribe to a presence topic on the mesh
25|- Result: checking 500 people costs the same as checking 5 — one multicast subscription
26|
27|**Cost per follow:** ~0 (gossip-based, not polling-based)
28|
29|### 2. Content Fetching — Lazy + Push
30|
31|**Problem:** Fetching all new posts from 200 online users every time you open the feed.
32|
33|**Solution:** Two-phase approach:
34|
35|- **Push notifications:** When someone you follow publishes a post, THEIR node pushes a notification to YOUR node (just the event ID + timestamp, not the full content). This is a tiny message (~100 bytes).
36|- **Lazy content fetch:** When you open the feed, your node fetches only the content for posts you haven't seen yet, and only for the time range visible on screen (pagination).
37|
38|**Cost per follow:** ~100 bytes per post published (notification only), content fetched on demand
39|
40|### 3. Storage — Tiered Local Cache
41|
42|**Problem:** Storing all posts from all followees forever fills up your disk.
43|
44|**Solution:** Tiered storage:
45|
46|| Tier | Content | Retention | Storage |
47||---|---|---|---|
48|| Hot | Posts from last 7 days | Always cached locally | SQLite (fast, on disk) |
49|| Warm | Posts from last 90 days | Cached up to storage limit | SQLite (pruned when full) |
50|| Cold | Posts older than 90 days | Not stored, fetched from origin on demand | Origin node or CDN |
51|
52|When you scroll back past 7 days, the feed shows a "Loading older posts..." indicator and fetches from the origin node (if online) or CDN (if available).
53|
54|**Typical storage per followee:** ~50 KB for text posts, ~2-5 MB for image posts (thumbnails only), ~0 for video (CDN-hosted)
55|
56|**For 500 followees:** ~25-50 MB for text, ~1-2.5 GB with images. Manageable on any modern machine.
57|
58|### 4. Processing — Virtual Scroll + SQLite Indexes
59|
60|**Problem:** Rendering thousands of posts in a web UI.
61|
62|**Solution:**
63|
64|- **Virtual scrolling / pagination:** Only render the 20-50 posts visible on screen. "Load more" button (no infinite scroll per your requirement).
65|- **SQLite indexing:** All local posts indexed by timestamp, author, channel type. Feed query is a simple `SELECT * FROM events WHERE author_id IN (follow_list) ORDER BY timestamp DESC LIMIT 20 OFFSET ?`
66|- **Background indexing:** When new notifications arrive, posts are inserted into SQLite in the background. Opening the feed is a fast read from a local database, not a network request.
67|
68|**Cost per follow:** ~0 (SQLite handles millions of rows efficiently)
69|
70|## RESULT: NO CAP NEEDED
71|
72|| Concern | Solution | Scaling Concern |
73||---|---|---|
74|| Presence tracking | Multicast gossip | O(1), not O(N) |
75|| Content notification | Push (tiny headers only) | ~100 bytes/post |
76|| Content fetching | Lazy, on-demand, paginated | Only fetch what's on screen |
77|| Storage | Tiered cache, auto-pruning | ~50 MB for 500 text-only follows |
78|| Rendering | Virtual scroll + SQLite | Local DB query, sub-millisecond |
79|
80|A user following 10,000 people (extreme edge case) might use ~5 GB of local cache and see slightly slower feed load. A user following 500 people (realistic max) will see no measurable performance difference from following 5.
81|
82|**Bottom line: No follow cap. The architecture handles it.**