# Claude Code Prompts — Overview
## How to use the implementation prompts

---

## Prompt files and their relationships

```
docs/prompts/DESIGN-PROMPTS.md       ← Run THIS with Claude to build the egui UI
                    │
                    │  is referenced by
                    ▼
source/prompts-unified-phases-1-10.md  ← Run THIS with Claude Code for architecture + UI
                    │
                    │  DESIGN REFERENCE sections point to
                    ▼
docs/design/UI-PROMPTS.md             ← Same file as DESIGN-PROMPTS.md (two paths, one file)
```

### The two prompt types

| Prompt file | Run with | What it produces |
|---|---|---|
| `source/prompts-unified-phases-1-10.md` | **Claude Code** | Rust backend + egui rendering, Phases 1–10 |
| `docs/prompts/DESIGN-PROMPTS.md` | **Claude** (chat/design) | egui theme, all UI panels, standalone |

### Claude Design command (paste this into Claude to implement the UI)

```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```

Design file URL: `https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html`
| `source/spec-site-config-phase-11.md` | **Claude Code** | Per-site config backend |
| `source/spec-phpmyadmin-phase-12.md` | **Claude Code** | phpMyAdmin service |
| `docs/prompts/PHASE-13.md` | **Claude Code** | All 21 framework detection + App Creator |
| `docs/prompts/PHASE-14.md` | **Claude Code** | Version registry + crate version bumps |

---

## How the two prompt types connect

The **unified phases prompt** (Claude Code) has a `DESIGN REFERENCE` line in each phase:

```
DESIGN REFERENCE: Read docs/design/UI-PROMPTS.md §D5 for the PHP panel spec.
```

This tells Claude Code to read `docs/design/UI-PROMPTS.md` (the egui design spec)
and implement the panel according to it. The design spec describes exact colours,
layout dimensions, component calls, and typography — Claude Code implements it in Rust.

The **DESIGN-PROMPTS.md** (run with Claude) can also be used standalone to:
- Generate the theme engine and all panels without writing any Rust logic
- Iterate on panel layouts before the backend exists
- Produce reference implementations that Claude Code phases then wire up

---

## Correct run order

```
Step 1 (optional):  In a Claude chat, run:
                    "Fetch this design file, read its readme, and implement the relevant aspects
                     of the design. https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
                     Implement: Valet Manager.html"
                    → review the UI panels interactively
                    → extract any design decisions you want to change

Step 2:             cd ~/projects/valet-manager && claude
                    Paste Phase 1 from source/prompts-unified-phases-1-10.md
                    (it reads docs/design/UI-PROMPTS.md for the design spec)

Step 3:             cargo build --workspace && cargo clippy -- -D warnings
                    Check Phase 1 acceptance criteria

Step 4:             Repeat for Phases 2–10 in order

Step 5:             Paste source/spec-site-config-phase-11.md
Step 6:             Paste source/spec-phpmyadmin-phase-12.md
Step 7:             Paste docs/prompts/PHASE-13.md
Step 8:             Paste docs/prompts/PHASE-14.md  (run this first — bumps crates)
```

---

## Design contract (applies to every Claude Code phase)

Every `src/ui/` file must use `Colors::*` from `src/ui/theme.rs`.
No `Color32::from_rgb(...)` outside `theme.rs`.

After each phase verify:
```bash
grep -rn "from_rgb\|from_rgba" src/ui/panels/ src/ui/sidebar.rs  # must be zero
grep -rn "Stroke::new(1\."     src/ui/                           # must be zero
cargo build --workspace 2>&1 | grep "^error"
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

---

## Phase dependency chain

```
1 → 2 → 3 → 4 → 5 (strict linear, each reads files from previous)
                ↓
           6 → 7 → 8 → 9 → 10
                              ↓
                         11 → 12 → 13 → 14
```

Phase 14 (version registry + crate bumps) should be run BEFORE Phase 13
if starting fresh, because the egui 0.34.2 API change affects all panels.

---

## Quick reference: AppCommand variants by phase

| Phase | Key new commands |
|---|---|
| 1 | RefreshAll, OpenPanel, OpenCommandPalette |
| 2 | SwitchGlobalPhp, IsolateSite, EnableExtension, SavePhpIni |
| 3 | RefreshSites, ParkDirectory, LinkSite, SecureSite |
| 4–5 | CreateApp, CancelCreation |
| 6 | AddProxy, SetTld, StartShare, RunDiagnose |
| 7 | CheckForUpdates, TrustValet |
| 8 | ViewPhpInfo, CheckAllCompatibility, ClearHistory |
| 9 | OpenCommandPalette (full), LoadDotEnv, RunArtisanCommand, CreateDatabase |
| 10 | EnableXdebug, StartMailCatcher, StartQueueWorker, RegenerateSslCert |
| 11 | LoadSiteConfig, SaveSiteConfig, EnableWpMultisite, SetSiteHttpServer |
| 12 | InstallPhpMyAdmin, ConfigurePhpMyAdminForSite, OpenPhpMyAdmin |
| 13 | (framework detection, App Creator types — no new commands) |
| 14 | RefreshVersionRegistry |

---

*Author: Al Amin Ahamed (@mralaminahamed)*
