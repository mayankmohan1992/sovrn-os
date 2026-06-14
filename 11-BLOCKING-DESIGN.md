# Sovrn — Blocking Design (Bidirectional, Network-Wide)

## THE GOTCHAS

Bidirectional blocking sounds simple ("A and B can't see each other"). In a decentralized system, there are edge cases:

### Gotcha 1: Block Propagation Delay
When A blocks B, the block event must propagate across the mesh. During propagation:
- A's node immediately stops serving content to B
- B's node hasn't received the block yet, so B still tries to fetch A's content
- B's requests to A get rejected (A already enforce the block locally)
- B sees "content unavailable" for a brief period until the block event reaches them

**Resolution:** This is acceptable. B sees a brief "unavailable" then A disappears entirely. The delay is seconds to minutes at most (Yggdrasil gossip speed).

### Gotcha 2: Mutual Block Race Condition
A blocks B at time T1. B blocks A at time T2 (later). Both have valid signed block events. No conflict — both events are kept. Order doesn't matter, result is the same.

### Gotcha 3: Caching — B Already Has A's Content
If B has previously saved or cached A's posts before the block:
- A's block removes A's content from B's feed UI
- B's local cache still has the data on disk
- This is the same as any social network: unfollow/block hides content from view, doesn't delete local browser cache
- If B is technically skilled enough to dig into the SQLite database, they can find old cached content
- This is UNAVOIDABLE in any system where content is delivered to a client

**Resolution:** Same as every social platform. Block hides content from the UI. It does not and cannot retroactively wipe the other person's hard drive.

### Gotcha 4: Third-Party Relays (The "See Through a Friend" Problem)
A blocks B. B asks C (who follows both A and B) to show them A's content. This is:
- **Technically possible:** C could manually copy A's posts and send them to B
- **Not a protocol problem:** This is the "you can't stop someone from taking a screenshot" problem
- **Same on every platform:** Twitter, Instagram, Facebook all have this issue
- **No cryptographic solution:** Short of DRM on content (which contradicts our values), this can't be prevented

**Resolution:** Accept this limitation. Block is a UI-level and protocol-level best effort, not a guarantee against determined bypass. The goal is to make casual browsing impossible, not to be cryptographically airtight against a determined adversary.

### Gotcha 5: Block on Some Devices, Not Others (Multi-Device)
If A has Sovrn on laptop and phone, both devices must enforce the block. Since both devices share the same identity keypair:
- Block event is signed by A's key
- Both devices receive and store the block event
- Both devices enforce it consistently

**Resolution:** No issue. Block events are signed and propagated like any other event. All devices see the same block list.

### Gotcha 6: Block List Privacy
Should B know that A blocked them?
- **Option (a): Explicit notification** — B gets a notification "A has blocked you." Clear, honest, but can cause confrontational behavior.
- **Option (b): Silent disappearance** — A just disappears from B's view. B notices A isn't around anymore. Less confrontational, more ambiguous.
- **Option (c): Soft block** — A stops seeing B's content, but B can still see A's. This is actually a "mute," not a block.

**Resolution for Sovrn:** Option (b) — silent disappearance. A block is not announced. B simply stops seeing A's content and A stops seeing B's. No notification. If B searches for A, A doesn't appear. B might figure it out, but no explicit "you've been blocked" message.

### Gotcha 7: Unblocking
A blocks B, then later unblocks B.
- Unblocking is a new signed event that removes the block
- B's content becomes visible to A again (if B is online)
- Does B get a notification that A unblocked them? **No.** Silent.

## FINAL DESIGN

### Block Event (Protocol Level)
```json
{
  "type": "block",
  "author": "<A's public key hash>",
  "target": "<B's public key hash>",
  "timestamp": 1717344000,
  "signature": "<Ed25519 signature>"
}
```

### Block Enforcement (Node Level)
When A's node sees a block event from A targeting B:
1. A's node stops serving any content to B's node
2. A's node stops requesting content from B's node
3. A's feed UI removes all of B's content
4. A's node rejects all incoming requests from B's node
5. Block event is propagated to the mesh (so B's node eventually receives it)

When B's node receives the block event:
1. B's node stops requesting content from A's node
2. B's feed UI removes all of A's content
3. B's node stops serving content to A's node (bidirectional)

### Block List (Storage)
- Each node maintains a local block list (list of blocked public key hashes)
- Block list is encrypted (stored locally, not broadcast to the mesh)
- Block events themselves are broadcast (only the target public key hash + signature, no reason given)

### Summary
| Feature | Behavior |
|---|---|
| Blocking | Bidirectional: A and B can't see each other |
| Notification | Silent: no "you've been blocked" message |
| Caching | Old cached content hidden from UI, not wiped from disk |
| Third-party bypass | Possible but not a protocol concern (like screenshots) |
| Multi-device | Consistent: block syncs across all devices |
| Unblocking | Signed event, silent, content reappears |
| Propagation delay | Seconds to minutes, content shows as "unavailable" briefly |