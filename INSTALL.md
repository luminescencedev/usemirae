# Mirae — Repository Installation Guide

> Place this file at the root of the new Mirae repository as `INSTALL.md`.
>
> This guide installs the architecture documentation, development blueprint, UI/UX visual system, brand assets, design tokens, and frontend foundations into one clean repository.

---

## 1. Installation order

Install the documentation packs in numeric order:

1. Pack 01 — Foundations
2. Pack 02 — Runtime
3. Pack 03 — Rendering
4. Pack 04 — Media
5. Pack 05 — Project and Persistence
6. Pack 06 — Platform
7. Pack 07 — Quality
8. Pack 08 — SDK and Extensions
9. Pack 09 — Development Blueprint
10. Pack 10 — UI, UX and Visual System

Later packs may replace `docs/SUMMARY.md` with an expanded version. The newest `SUMMARY.md` is authoritative.

---

## 2. Required tools

The exact versions must eventually be pinned by `MIR-0002 — Pin toolchains`.

Required locally:

- Git
- Rust and Cargo
- Node.js
- pnpm
- platform-native compiler/build tools
- PowerShell on Windows, or a POSIX-compatible shell on macOS/Linux

Do not install production credentials, signing keys, or streaming secrets into the repository.

---

## 3. Create the repository

```bash
mkdir mirae
cd mirae
git init
```

The target repository root will eventually contain:

```text
mirae/
├── apps/
├── crates/
├── packages/
├── schemas/
├── tools/
├── tests/
├── fixtures/
├── assets/
├── docs/
├── CLAUDE.md
├── BOOTSTRAP_TICKETS.md
├── INSTALL.md
├── Cargo.toml
├── package.json
├── pnpm-workspace.yaml
└── rust-toolchain.toml
```

Most implementation directories are created by the bootstrap tickets. The documentation can be installed before the code workspace exists.

---

## 4. Install Packs 01–09

Extract every pack into a temporary directory, then merge its contents into the repository root in numeric order.

### macOS/Linux

```bash
unzip mirae-docs-pack-01-foundations.zip -d /tmp/mirae-pack-01
cp -R /tmp/mirae-pack-01/. .
```

Repeat for Packs 02 through 09.

### Windows PowerShell

```powershell
Expand-Archive .\mirae-docs-pack-01-foundations.zip -DestinationPath $env:TEMP\mirae-pack-01 -Force
Copy-Item "$env:TEMP\mirae-pack-01\*" . -Recurse -Force
```

Repeat for Packs 02 through 09.

Pack 09 provides the root files:

```text
CLAUDE.md
BOOTSTRAP_TICKETS.md
```

Keep these files at the repository root.

---

## 5. Install Pack 10 — UI, UX and Visual System

Extract `mirae-visual-pack-v1.zip` into a temporary directory.

Copy the canonical UI documentation:

```text
mirae-visual-pack-v1/docs/09-ui-ux/
    → docs/09-ui-ux/

mirae-visual-pack-v1/docs/adr/ADR-0061-*.md
mirae-visual-pack-v1/docs/adr/ADR-0062-*.md
mirae-visual-pack-v1/docs/adr/ADR-0063-*.md
mirae-visual-pack-v1/docs/adr/ADR-0064-*.md
mirae-visual-pack-v1/docs/adr/ADR-0065-*.md
mirae-visual-pack-v1/docs/adr/ADR-0066-*.md
    → docs/adr/
```

Append the section from:

```text
mirae-visual-pack-v1/docs/SUMMARY-APPEND.md
```

to the repository `docs/SUMMARY.md` if the Pack 10 section is not already present.

Do not replace the complete summary with `SUMMARY-APPEND.md`; it is only a fragment.

---

## 6. Install the visual references

The visual renders are documentation references, not production UI assets.

Copy:

```text
mirae-visual-pack-v1/previews/
    → docs/assets/visual-direction-v1/previews/

mirae-visual-pack-v1/presentation/
    → docs/assets/visual-direction-v1/presentation/
```

Recommended final structure:

```text
docs/assets/visual-direction-v1/
├── previews/
│   ├── edit-workspace.png
│   ├── studio-mode.png
│   ├── audio-mixer.png
│   ├── command-palette.png
│   ├── color-system.png
│   └── final-direction.png
└── presentation/
    ├── mirae-visual-direction-v1.pptx
    └── mirae-visual-direction-v1-montage.png
```

Do not copy the conceptual mockup layout directly into application code. Rebuild every screen from Mirae tokens and components.

---

## 7. Install the brand assets

Create the canonical brand source directory:

```text
assets/brand/
```

Copy:

```text
mirae-visual-pack-v1/brand/mirae-mark.svg
mirae-visual-pack-v1/brand/mirae-logo-horizontal-dark.svg
mirae-visual-pack-v1/brand/mirae-app-icon-dark.svg
mirae-visual-pack-v1/brand/mirae-app-icon-dark.png
mirae-visual-pack-v1/brand/mirae-mark-transparent.png
```

into:

```text
assets/brand/
```

Rules:

- `mirae-mark.svg` is the canonical vector mark.
- Platform icons are generated from the canonical vector source.
- Do not edit exported PNG files as the source of truth.
- Do not stretch, outline, recolor, or redraw the mark inside feature code.
- Product surfaces should consume brand assets through a shared asset module.

---

## 8. Install the design tokens

After `packages/ui-kit/` exists, create:

```text
packages/ui-kit/
├── tokens/
│   └── design-tokens.v1.json
└── src/
    └── styles/
        └── tokens.css
```

Copy:

```text
mirae-visual-pack-v1/tokens/design-tokens-v1.json
    → packages/ui-kit/tokens/design-tokens.v1.json

mirae-visual-pack-v1/tokens/design-tokens-v1.css
    → packages/ui-kit/src/styles/tokens.css
```

The JSON file is the canonical token source.

The CSS file is a generated/runtime representation and must eventually be produced by `cargo xtask generate` or the dedicated token generator.

Do not manually create independent color values inside feature components.

---

## 9. Install the frontend foundations

Run these commands after the pnpm workspace and `apps/control-ui` package exist.

### Runtime UI dependencies

```bash
pnpm --filter @mirae/control-ui add \
  motion \
  @dnd-kit/react \
  @dnd-kit/helpers \
  @tanstack/react-virtual \
  react-resizable-panels \
  react-hook-form \
  @hugeicons/react \
  @hugeicons/core-free-icons
```

The accessible interaction foundation is not installed here. MIR-0118 made
`react-aria-components` private to `@mirae/ui-kit`, so it belongs to that
package's manifest and never to a feature application
(`docs/09-ui-ux/911-ui-library-decisions.md`).

### Tailwind CSS v4 for Vite

```bash
pnpm --filter @mirae/control-ui add -D \
  tailwindcss \
  @tailwindcss/vite
```

The Vite configuration must register the Tailwind plugin, and the root UI stylesheet must import Tailwind:

```css
@import "tailwindcss";
@import "@mirae/ui-kit/styles/tokens.css";
```

Adapt the token import to the package export map once `@mirae/ui-kit` is created.

### UI validation dependencies

```bash
pnpm --filter @mirae/control-ui add -D \
  vitest \
  @testing-library/react \
  @testing-library/user-event \
  @playwright/test \
  @axe-core/playwright
```

Then install Playwright browser binaries:

```bash
pnpm exec playwright install
```

Versions must be pinned by the workspace lockfile. Do not use unpinned CDN imports in the desktop application.

---

## 10. Create the UI kit package

The target package is:

```text
packages/ui-kit/
├── package.json
├── src/
│   ├── components/
│   ├── primitives/
│   ├── patterns/
│   ├── icons/
│   ├── motion/
│   ├── styles/
│   │   ├── index.css
│   │   └── tokens.css
│   └── index.ts
├── tokens/
│   └── design-tokens.v1.json
└── tests/
```

Responsibilities:

- wrap Base UI primitives;
- expose Mirae-owned component APIs;
- apply the Obsidian Precision tokens;
- centralize keyboard and focus behavior;
- centralize Motion configuration;
- centralize Hugeicons rendering through one `Icon` wrapper;
- expose no application-domain state.

Do not import Base UI, Motion, Hugeicons, or resizable-panel primitives directly across feature code when a Mirae wrapper exists.

---

## 11. Required root files

The repository root must contain:

```text
CLAUDE.md
BOOTSTRAP_TICKETS.md
INSTALL.md
```

`CLAUDE.md` defines how coding agents work.

`BOOTSTRAP_TICKETS.md` defines the first implementation queue.

`INSTALL.md` is this installation and integration guide.

---

## 12. Start implementation

The first ticket remains:

```text
MIR-0001 — Initialize monorepo
```

Then continue in order from `BOOTSTRAP_TICKETS.md`.

Do not begin complex screen implementation until the following foundations exist:

- pnpm workspace;
- `apps/control-ui`;
- `packages/ui-kit`;
- canonical tokens;
- Base UI wrappers;
- motion configuration;
- icon wrapper;
- accessibility test harness;
- visual-regression test harness.

---

## 13. Validation

When `xtask` exists, run:

```bash
cargo xtask generate --check
cargo xtask fmt --check
cargo xtask lint
cargo xtask test-affected
cargo xtask docs --check
```

Before `xtask` is implemented, use the temporary fallback checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
pnpm -r typecheck
pnpm -r test
pnpm -r build
```

Also verify:

- all `docs/SUMMARY.md` links resolve;
- ADR-0001 through ADR-0066 are present;
- tokens contain no duplicate semantic names;
- the UI contains no hard-coded replacement palette;
- the logo renders from the canonical SVG;
- no secrets or local paths are committed.

---

## 14. Expected documentation structure

After every pack is installed:

```text
docs/
├── 00-foundations/
├── 01-runtime/
├── 02-rendering/
├── 03-media/
├── 04-project/
├── 05-platform/
├── 06-quality/
├── 07-sdk/
├── 08-development/
├── 09-ui-ux/
├── adr/
├── assets/
├── README.md
└── SUMMARY.md
```

---

## 15. Important rules

- The old web Mirae repository is a source for the brand mark and selected polish ideas only.
- The new desktop application is dark-first and desktop-first.
- Obsidian Precision is the canonical visual direction.
- The design-token JSON is the canonical visual value source.
- UI mockups are references, not source code.
- Engine state and UI draft state remain separate.
- Every drag-and-drop workflow requires a keyboard equivalent.
- Motion must respect reduced-motion preferences.
- Glass effects are limited to floating surfaces.
- Production, Live, Record, warning, and failure colors remain semantically reserved.
- No library primitive should define Mirae's public component API directly.

---

## 16. Clean installation checkpoint

Before starting `MIR-0001`, commit the documentation and visual foundation separately:

```bash
git add docs assets CLAUDE.md BOOTSTRAP_TICKETS.md INSTALL.md
git commit -m "docs: install Mirae architecture and visual system"
```

Then start implementation from a clean branch.
