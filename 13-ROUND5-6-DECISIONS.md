# Sovrn — Round 5-6 Decisions

## G1: Installation → Ubuntu-style installer
- Bootable USB with live mode (try without installing) AND install option
- Install options: Erase disk, Install alongside Windows, Install alongside other Linux
- Calamares installer (standard, well-tested, used by many distros)

## G4: Guest mode → Yes, live USB
- Full Sovrn experience from USB without installing
- Slower (runs from USB) but fully functional
- OOBE wizard runs every time on live USB (no persistence by default)
- Option to "Install Sovrn" from live desktop

## H1: Privacy / Ghost Mode → Fine-grained visibility controls
- Users can see public content without following, with NO activity record
- Browsing is anonymous by default — no "viewed" events sent to content authors
- Activity is recorded ONLY when user takes explicit action: like, comment, share, follow
- Settings per profile:
  - Who can see my online status: everyone / followers only / nobody (ghost mode)
  - Who can see my content: everyone / followers only
  - Who can message me: everyone / followers only / nobody
  - Who can find me in search: everyone / followers only / nobody
- Default: content = everyone, messaging = everyone, online status = followers only, search = everyone

## H2: CSAM and Content Moderation → Automated scan + report system
- Flow: User reports CSAM → post disappears from all feeds immediately → sent to open-source scanner → if flagged, post stays blocked, author gets 24-hour notice to remove or get blocked from network
- Scanner: PhotoDNA (Microsoft, free for eligible organizations), or open-source alternatives like NSFWJS (content classification), TensorFlow models for CSAM detection
- This is an area that needs more research — documenting options below
- Key principle: automated moderation, no human moderator team needed
- Report system: any user can report, multiple independent reports escalate priority
- False positives: scanner not 100%, so 24-hour window gives author chance to appeal
- Appeal: author can dispute, post goes to community review (trusted node operators) — but this is v2

## H3: Account Deletion → Both options with double warning
- Option 1: Deactivate (hide from mesh, keep data, can return)
  - Domain reserved for 12 months
  - Content hidden from all feeds
  - Node stops broadcasting presence
  - Can reactivate anytime with seed phrase
- Option 2: Nuclear delete (irreversible)
  - Two confirmation screens with warnings
  - "This is irreversible. Your identity, domain, and all content will be permanently deleted."
  - Broadcasts deletion event to entire mesh
  - Domain released immediately
  - All cached content on other nodes receives deletion event
  - Seed phrase is invalidated
  - Cannot be undone

## H4: Accessibility → Not in v1
- Skip screen reader support and ARIA for v1
- Focus on core functionality first, add a11y in v2

## H5: Language → English only for v1
- Keeps installation size small
- Localization planned for future versions