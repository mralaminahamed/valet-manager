# Valet Manager — Project Plan Archive
## Complete planning documentation · May 2026

---

## What's in this archive

This archive contains the full planning documentation for **Valet Manager**,
a native Linux desktop application (Rust + egui) for managing Laravel Valet
PHP development environments.

---

## Quick start

**Start here:** `docs/INDEX.md` maps every file to a reader role.

### Claude Design command (paste into Claude to implement the UI)

```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```

Every phase in `source/prompts-unified-phases-1-10.md` includes this command
in its `DESIGN REFERENCE` block so Claude Code fetches the live design automatically.

| I want to… | Go to |
|---|---|
| Understand the project | `docs/README.md` |
| See the full timeline | `docs/ROADMAP.md` |
| See all 78 features | `docs/FEATURES.md` |
| See current verified versions | `docs/VERSIONS.md` |
| Start implementing | `docs/prompts/OVERVIEW.md` |
| Understand the UI design | `docs/design/DESIGN-SYSTEM.md` |
| Understand per-site config | `docs/specs/SITE-CONFIG.md` |
| Understand phpMyAdmin integration | `docs/specs/PHPMYADMIN.md` |
| Understand HTTP server support | `docs/specs/HTTP-SERVERS.md` |
| Understand framework detection | `docs/specs/FRAMEWORKS.md` |
| Understand version tracking | `docs/specs/VERSION-REGISTRY.md` |
| Set up a dev environment | `docs/CONTRIBUTING.md` |
| See all Cargo dependencies | `docs/DEPENDENCIES.md` |

---

## Directory structure

```
valet-manager-plan/
│
├── docs/                        Clean summary documents (start here)
│   ├── INDEX.md                 Master navigation — start here
│   ├── README.md                Project overview + feature highlights
│   ├── ROADMAP.md               14-phase development timeline (~42 weeks)
│   ├── FEATURES.md              78-feature inventory with PHPMon comparison
│   ├── VERSIONS.md              All verified software versions (May 2026)
│   ├── CONTRIBUTING.md          Dev setup, code standards, build commands
│   ├── DEPENDENCIES.md          All Cargo crates with rationale + version pins
│   │
│   ├── design/
│   │   └── DESIGN-SYSTEM.md     Colour tokens, typography, egui component API
│   │
│   ├── specs/                   Technical specifications
│   │   ├── SITE-CONFIG.md       Per-site TOML config spec
│   │   ├── PHPMYADMIN.md        phpMyAdmin per-site service spec
│   │   ├── HTTP-SERVERS.md      Nginx/FrankenPHP/Caddy/Apache abstraction
│   │   ├── FRAMEWORKS.md        All 21 Valet frameworks — detection to creator
│   │   └── VERSION-REGISTRY.md  In-app version tracking + refresh
│   │
│   └── prompts/                 Claude Code implementation prompts
│       ├── OVERVIEW.md          How to use the prompts
│       ├── PHASE-13.md          Framework support (all 21 frameworks)
│       └── PHASE-14.md          Version registry + crate upgrades
│
├── source/                      Full-detail originals (complete specs)
│   ├── architecture-v1.md       Initial architecture design
│   ├── architecture-v2.md       Complete architecture v2 (canonical)
│   ├── architecture-v3-additions.md  v3 additions (DX features, history, palette)
│   ├── phpmon-comparison.md     PHPMon vs Valet Manager full comparison
│   ├── prompts-architecture-only.md  Original Phases 1–10 (arch only)
│   ├── prompts-unified-phases-1-10.md  PRIMARY — arch + design merged
│   ├── prompts-ui-design-d1-d10.md    egui theme + all panels (D1–D10)
│   ├── spec-site-config-phase-11.md   Phase 11 full spec + Claude Code prompt
│   ├── spec-phpmyadmin-phase-12.md    Phase 12 full spec + Claude Code prompt
│   └── icon-guide.md            Brand asset installation + Rust embedding
│
└── assets/
    └── icons/
        ├── icon-512.svg          App icon 512px
        ├── wordmark.svg          Horizontal logo lockup
        ├── favicon.svg           32px favicon
        └── tray-icon.svg         22px GTK system tray icon
```

---

## Implementation order

Run Claude Code prompts in this sequence from the project root:

```
Phase  1–10 → source/prompts-unified-phases-1-10.md  (architecture + UI)
Phase 11    → source/spec-site-config-phase-11.md     (per-site config backend)
Phase 11b   → same file, Phase 11b section             (UI wiring)
Phase 12    → source/spec-phpmyadmin-phase-12.md      (phpMyAdmin service)
Phase 13    → docs/prompts/PHASE-13.md                 (all 21 frameworks)
Phase 14    → docs/prompts/PHASE-14.md                 (version registry + crate bumps)

Design  D1–D10 → source/prompts-ui-design-d1-d10.md  (run alongside Phase 1)
```

---

## Key technical decisions

| Decision | Rationale |
|---|---|
| Rust + egui 0.34.2 | Zero runtime deps, wgpu backend, immediate-mode suits live polling |
| tokio mpsc channels | GUI is single-threaded; background tasks send events via channels |
| Dual config files | Site root = portable/git; centralized = machine-local overrides |
| Sentinel comments in Nginx | Idempotent inject/remove without corrupting existing config |
| `PMA_CONFIG_DIR` for phpMyAdmin | Per-site DB isolation without multiple phpMyAdmin installs |
| `VersionRegistry::default()` with bundled data | App works offline; first refresh updates to live values |
| endoflife.date API for PHP EOL | Single authoritative source; JSON API; no scraping |

---

## Author

Al Amin Ahamed · [@mralaminahamed](https://github.com/mralaminahamed)
Senior Software Engineer & Team Lead of Product — Codexpert Inc., Dhaka, Bangladesh

---

*Generated: May 2026*
