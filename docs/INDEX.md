# Valet Manager — Project Plan Index
## Every planning document, what it contains, when to read it

---

## Quick orientation
> **Claude Design command** (paste into Claude to build the UI):
>
> ```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```


```
docs/
├── README.md                    Project overview + feature highlights
├── ROADMAP.md                   12-phase timeline (40 weeks)
├── FEATURES.md                  78-feature inventory with PHPMon comparison
├── COMPARISON.md                PHPMon vs Valet Manager full table
├── CONTRIBUTING.md              Dev setup, code standards, build commands
├── DEPENDENCIES.md              All Cargo crates with rationale + version pins
│
├── design/
│   ├── DESIGN-SYSTEM.md         Colour tokens, typography, egui component API
│   └── UI-PROMPTS.md             egui rendering prompts D1–D10 (run with Claude)
│
├── specs/
│   ├── SITE-CONFIG.md            Per-site TOML config: PHP, WP, Laravel, HTTP server
│   ├── PHPMYADMIN.md             phpMyAdmin per-site service spec
│   ├── HTTP-SERVERS.md           Nginx / FrankenPHP / Caddy / Apache abstraction
│   └── FRAMEWORKS.md             All 21 Valet frameworks: detection, badges, App Creator
│
└── prompts/
    ├── OVERVIEW.md               How to use Claude Code prompts
    ├── PHASES-01-07.md           Foundation → Polish (Claude Code prompts)
    ├── PHASES-08-10.md           PHPMon parity → DX Tier 2 (Claude Code prompts)
    ├── PHASE-11.md               Per-site config + HTTP servers (Claude Code prompts)
    ├── PHASE-12.md               phpMyAdmin (Claude Code prompts)
    └── DESIGN-PROMPTS.md         egui theme + all panels (Claude Design prompts)
```

---

## Document map by role

### "I want to understand the project"

### "I want to see all current verified versions"
→ [VERSIONS.md](VERSIONS.md)
→ [README.md](README.md) then [FEATURES.md](FEATURES.md)

### "I want to understand the timeline"
→ [ROADMAP.md](ROADMAP.md)

### "I want to compare with PHPMon"
→ [COMPARISON.md](COMPARISON.md)

### "I want to start implementing"
→ [prompts/OVERVIEW.md](prompts/OVERVIEW.md) → run Phase 1 prompt

### "I want to understand the visual design"
→ [design/DESIGN-SYSTEM.md](design/DESIGN-SYSTEM.md) → run DESIGN-PROMPTS.md

### "I want to understand per-site config"
→ [specs/SITE-CONFIG.md](specs/SITE-CONFIG.md)

### "I want to understand phpMyAdmin integration"
→ [specs/PHPMYADMIN.md](specs/PHPMYADMIN.md)

### "I want to understand HTTP server support"
→ [specs/HTTP-SERVERS.md](specs/HTTP-SERVERS.md)

### "I want to set up a dev environment"
→ [CONTRIBUTING.md](CONTRIBUTING.md)

### "I want to see all Cargo dependencies"
→ [DEPENDENCIES.md](DEPENDENCIES.md)

---

## Source files (full detail)

These files contain the original unabridged specifications produced during
the planning phase. The `docs/` files above are clean summaries; these are
the canonical reference for full implementation detail.

| File | Contents |
|---|---|
| `valet-manager-architecture-v2.md` | Complete architecture spec v2 (Valet command coverage, module structure, domain models) |
| `valet-manager-architecture-v3-additions.md` | All gap-closing and new DX features added in v3 (phpinfo, palette, DB manager, etc.) |
| `phpmon-vs-valet-manager-comparison.md` | Full PHPMon feature comparison (12 categories, 169 features) |
| `claude-code-prompts-unified.md` | **Primary prompt file** — Phases 1–10, architecture + design merged per phase |
| `claude-code-prompts-all-phases.md` | Original architecture-only prompts (reference only, superseded by unified) |
| `ui-design-prompts.md` | egui design prompts D1–D10 (theme engine, all panels, components) |
| `per-site-config-spec-and-prompts.md` | Phase 11 full spec + Claude Code prompt |
| `phpmyadmin-per-site-spec-and-prompt.md` | Phase 12 full spec + Claude Code prompt |

---

## Icon assets

| File | Use |
|---|---|
| `valet-manager-icon-512.svg` | App icon (XDG hicolor theme, 512px) |
| `valet-manager-wordmark.svg` | Horizontal lockup for splash / about |
| `valet-manager-favicon.svg` | 32px favicon for documentation site |
| `valet-manager-tray.svg` | 22px system tray icon (`currentColor`, GTK symbolic) |
| `valet-manager-icon-guide.md` | Icon installation + Rust embedding instructions |

---

## Implementation order

```
Week  1: Read README.md → ROADMAP.md → ARCHITECTURE notes
Week  1: Run design/DESIGN-SYSTEM.md to understand the visual language
Week  1: Run prompts/PHASES-01-07.md Phase 1 in Claude Code
Week  2: Verify Phase 1 acceptance criteria → run Phase 2
...
Week 35: Read specs/SITE-CONFIG.md → run prompts/PHASE-11.md
Week 38: Read specs/PHPMYADMIN.md → run prompts/PHASE-12.md
Week 40: Package .deb → publish GitHub release
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
*Project: Valet Manager · Linux desktop GUI for Laravel Valet*
