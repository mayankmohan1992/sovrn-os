# Sovrn — Design System

## COLOR PALETTE

Dark mode first (default). Light mode supported but secondary.

| Role | Name | Hex | Usage |
|---|---|---|---|
| Primary | Sovereign Teal | `#0D7377` | Buttons, active states, headers, links |
| Primary Dark | Deep Vault | `#0A4F52` | Hover/pressed states, nav bars |
| Accent | Crown Gold | `#E8A838` | CTAs, highlights, badges, toggles, notifications |
| Accent Muted | Dust Amber | `#B8862D` | Secondary icons, borders, progress bars |
| Success | Vernal Green | `#2ECC71` | Secure connection, verified identity, confirmed |
| Danger | Ember Red | `#E74C3C` | Warnings, insecure, errors, delete actions |
| Surface Dark | Obsidian | `#1A1D23` | Main background (dark mode) |
| Surface Light | Cloud Slate | `#F0F2F5` | Main background (light mode) |
| Text on Dark | `#E8EAED` | High-contrast text on dark surfaces |
| Text on Light | `#1A1D23` | High-contrast text on light surfaces |
| Text Muted | `#8B929A` | Labels, descriptions, timestamps |
| Border Dark | `#2D3139` | Cards, dividers (dark mode) |
| Border Light | `#C4C9D1` | Cards, dividers (light mode) |

Why this palette: Sovereign Teal conveys trust and security (different from generic blue). Crown Gold signals sovereignty and warmth. Obsidian is near-black with blue undertone (not void-black). No purple, no default blue.

## TYPOGRAPHY

| Role | Font | License | Weights |
|---|---|---|---|
| Body / UI | Inter | SIL OFL 1.1 | Regular 400, Medium 500, Semibold 600, Bold 700 |
| Display / Headings | Outfit | SIL OFL 1.1 | Semibold 600, Bold 700 |
| Monospace | JetBrains Mono | SIL OFL 1.1 | Regular 400, Bold 700 |

Type scale (1.25 major third):
- Display: 36px — Hero text, onboarding
- H1: 30px — Window titles
- H2: 24px — Section headers
- H3: 20px — Sub-sections
- Body: 16px — Default
- Small: 14px — Captions, labels
- Tiny: 12px — Timestamps, badges

Line height: 1.5 body, 1.2 headings. Letter-spacing: -0.01em headings, 0 body.

## COMPONENTS

Buttons:
- Primary: Sovereign Teal bg, white text, 8px border-radius, flat (no shadows)
- Secondary: Transparent bg, teal border, teal text
- Danger: Ember Red bg, white text
- Ghost: Text only, no border

Cards:
- 1px solid border, 12px border-radius, 16px padding, no drop shadows
- Privacy card variant: left border 3px in status color (teal/gold/red)

Toggles:
- Pill toggle, Off = slate track, On = Sovereign Teal + white thumb
- Show status text below: "Mesh: Connected"

Notifications:
- Bottom-right, auto-dismiss 5s, max 2 stacked
- Left stripe in status color

Input fields:
- 4px border-radius, 40px height, 2px teal focus ring
- Password fields: strength meter + toggle visibility

Spacing: 4px base unit. Scale: 4, 8, 12, 16, 24, 32, 48, 64.
Max content: 960px settings, 1280px data views.

Icons: Lucide (MIT, tree-shakeable, matches GNOME symbolic style)
Motion: Reduced by default (privacy users prefer minimal animation). Durations: 100ms micro, 150ms small, 250ms medium, 400ms large.

## CSS CUSTOM PROPERTIES

Shared across PWA and GNOME theme:

```css
:root {
  --sovrn-primary: #0D7377;
  --sovrn-primary-dark: #0A4F52;
  --sovrn-accent: #E8A838;
  --sovrn-accent-muted: #B8862D;
  --sovrn-success: #2ECC71;
  --sovrn-danger: #E74C3C;
  --sovrn-surface-dark: #1A1D23;
  --sovrn-surface-light: #F0F2F5;
  --sovrn-text-on-dark: #E8EAED;
  --sovrn-text-on-light: #1A1D23;
  --sovrn-text-muted: #8B929A;
  --sovrn-border-dark: #2D3139;
  --sovrn-border-light: #C4C9D1;
  --sovrn-radius-sm: 4px;
  --sovrn-radius-md: 8px;
  --sovrn-radius-lg: 12px;
  --sovrn-spacing-unit: 4px;
}
```

## GNOME THEME MAPPING

```css
/* gtk.css */
@define-color sovrn_primary #0D7377;
@define-color sovrn_accent #E8A838;
@define-color theme_bg_color #1A1D23;
@define-color theme_fg_color #E8EAED;
@define-color theme_selected_bg_color #0D7377;
@define-color theme_selected_fg_color #FFFFFF;
```

Customize GNOME theme using libadwaita recoloring + CSS overrides (not a full theme fork). Ship Sovrn colors as a GNOME theme overlay that applies on top of Adwaita-dark.