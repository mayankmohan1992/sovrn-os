# Sovrn — Content Moderation Design (CSAM & Illegal Content)

## AUTOMATED MODERATION PIPELINE

```
User A reports content as CSAM/illegal
        │
        ▼
Content immediately hidden from ALL feeds (reporter's view, then network-wide)
        │
        ▼
Content sent to automated scanner
        │
        ├── SCANNER: Not flagged → Content restored to feeds, reporter notified
        │
        └── SCANNER: Flagged as CSAM/illegal
                │
                ▼
        Content stays blocked network-wide
        Author receives 24-hour notice:
        "Your post was flagged and verified as violating Sovrn content policy.
         Please remove it within 24 hours or your node will be blocked from
         the Sovrn network."
                │
                ├── Author removes within 24h → Node stays active
                │
                └── Author does NOT remove within 24h → Node blocked from network
                    (other nodes refuse to connect, serve content, or communicate)
```

## OPEN-SOURCE SCANNING OPTIONS

### Option A: PhotoDNA (Recommended for CSAM)
- Microsoft's PhotoDNA: creates a unique signature of an image, compared against database of known CSAM hashes
- Free for eligible organizations (NCMEC partners, child safety orgs)
- Does NOT scan image content with AI — compares hashes against known CSAM database
- Zero false positives for known CSAM (exact hash match)
- Does NOT detect NEW CSAM (never-before-seen images) — only known images
- This is the industry standard used by Facebook, Google, Twitter

### Option B: Apple CSAM Detection (NeuralHash-style)
- On-device neural network that generates a perceptual hash
- Compare against known CSAM hash database
- Can detect slightly modified versions of known CSAM
- Open-source implementations exist (e.g., in the research community)

### Option C: NSFWJS / TensorFlow Content Classification
- Open-source (NSFWJS on GitHub)
- Classifies images: "Neutral, Drawing, Hentai, Porn, Sexy"
- Can be run locally on the reporting user's node or on a scanner node
- Higher false positive rate than hash-based systems
- Good for general content moderation (not just CSAM)

### Option D: Custom Hash + Classification Pipeline
- Combine PhotoDNA-style perceptual hashing (for known CSAM) with ML classification (for new content)
- PhotoDNA detects known CSAM with near-zero false positives
- ML classifier catches content that looks like CSAM but isn't in the hash database
- Both run as part of the report pipeline

## RECOMMENDED APPROACH FOR SOVRN

### v1: Report + PhotoDNA
- Any user can report content
- Reported content immediately hidden from feeds (pending review)
- PhotoDNA hash comparison against NCMEC/known CSAM database
- If match → content stays blocked, 24h notice to author
- If no match → content is flagged for community review (v2 feature)

### v2: Report + PhotoDNA + ML Classification
- Add NSFWJS or similar classifier for general content policy enforcement
- Community review system for disputed reports
- Trusted node operators as reviewers

### v3: Proactive Scanning (Optional, Privacy Debate)
- Automatically scan ALL uploaded media against PhotoDNA hash before it's served
- This would catch known CSAM before anyone reports it
- PRIVACY CONCERN: this means every image uploaded to Sovrn is hashed and compared against a database
- Could be implemented as an optional "safe mode" that users can enable
- Decision: NOT in v1, requires community discussion

## REPORT TYPES (v1)

| Report Category | Action |
|---|---|
| CSAM | Immediate hide → PhotoDNA scan → 24h notice if flagged |
| Spam | Hide from reporter only → user can mute/block |
| Harassment | Hide from reporter only → user can block |
| Illegal content (non-CSAM) | Immediate hide → manual review (v2) |
| Other | Logged for future review |

## BLOCKING A NODE FROM THE NETWORK

When a node is blocked (24h expired, content not removed):
1. Block event signed by Sovrn project key (NOT by individual users)
2. Other nodes receive block event and refuse connections from the blocked node
3. Blocked node cannot: serve content, register domains, send messages, appear in search
4. Block can be appealed by removing the offending content and requesting review
5. This is the ONLY case where the Sovrn project exerts centralized authority — to remove CSAM and illegal content
6. All block events are publicly auditable (transparency log)

## IMPORTANT NOTE

CSAM detection is a LEGAL REQUIREMENT in most jurisdictions. Running any network or service without CSAM detection exposes the project to legal liability. This is not optional — it must be in v1.

The automated approach means no human moderators are needed in v1. Reports trigger automated scanning. Only confirmed CSAM matches result in network-wide blocking. Everything else is user-controlled (mute, block, hide).