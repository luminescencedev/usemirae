# Mirae — Exact Dependency Versions

**Status:** Canonical  
**Freeze date:** 2026-07-29  
**Audience:** Claude Code, contributors, CI and release automation  
**Location:** Repository root as `DEPENDENCY_VERSIONS.md`

---

## 1. Purpose

This file is the authoritative version lock for the toolchain and the approved frontend/UI dependency stack used to bootstrap Mirae.

Claude Code MUST read this file immediately after `CLAUDE.md` and before creating or modifying any JavaScript, TypeScript, React, UI-kit, test, lint, or package-manager configuration.

This file defines direct dependency versions. The committed lockfiles define exact transitive dependency versions.

---

## 2. Hard rules for Claude Code

Claude MUST NOT:

- use `latest`, `next`, `canary`, `beta`, `rc`, `*`, `^`, or `~`;
- run `pnpm update --latest`;
- install a package without an explicit version;
- replace an approved library with an alternative;
- add a router, state manager, query client, component kit, toast library, chart library, editor, or canvas framework without a dedicated ticket;
- change Node, pnpm, Rust, TypeScript, React, Vite, or ESLint versions inside an unrelated feature ticket;
- manually edit generated lockfiles or generated contracts;
- add npm, Yarn, Bun, or Deno as an alternative project package manager;
- use `framer-motion`; Mirae uses the `motion` package;
- use Radix or shadcn as the public component layer; Mirae uses Base UI behind Mirae-owned components;
- import UI primitives directly from feature code when a Mirae wrapper exists.

Claude MUST:

- preserve every version in this document exactly;
- use `pnpm install --frozen-lockfile` in CI;
- commit `pnpm-lock.yaml`;
- commit `Cargo.lock` for deployable applications;
- use `--save-exact` when adding approved npm packages;
- update this document in the same PR as an approved version change;
- create a dedicated dependency-upgrade ticket for every version change;
- verify peer dependencies before accepting an upgrade.

---

## 3. Toolchain lock

| Tool | Exact version | Canonical file |
|---|---:|---|
| Rust | `1.97.1` | `rust-toolchain.toml` |
| Node.js | `24.18.1` | `.node-version` and root `package.json` |
| pnpm | `11.17.0` | root `package.json#packageManager` |
| TypeScript | `6.0.3` | pnpm catalog |
| Cargo | bundled with Rust `1.97.1` | Rust toolchain |
| npm | not a project package manager | do not pin as project tooling |

### Why TypeScript is not `7.x`

Although a newer TypeScript major exists, the approved `typescript-eslint` release supports TypeScript versions below `6.1.0`.

Mirae therefore pins TypeScript `6.0.3` until a dedicated upgrade ticket proves that the complete lint, Vite, generated-contract, editor, and test toolchain supports TypeScript 7.

Claude MUST NOT “correct” this pin to the latest TypeScript version.

---

## 4. Core React and build stack

| Package | Exact version | Ownership |
|---|---:|---|
| `react` | `19.2.8` | `apps/control-ui` runtime |
| `react-dom` | `19.2.8` | `apps/control-ui` runtime |
| `typescript` | `6.0.3` | workspace development |
| `vite` | `8.1.5` | `apps/control-ui` development |
| `@vitejs/plugin-react` | `6.0.4` | `apps/control-ui` development |
| `@types/react` | `19.2.17` | workspace development |
| `@types/react-dom` | `19.2.3` | workspace development |
| `@types/node` | `24.13.3` | workspace development |

`@types/node` stays on major `24` because the runtime is Node `24.18.1`. Do not install the latest Node 26 definitions.

---

## 5. Approved UI runtime stack

| Purpose | Package | Exact version | Public usage rule |
|---|---|---:|---|
| Accessible primitives | `@base-ui/react` | `1.6.0` | wrap in `@mirae/ui-kit` |
| Motion and layout animation | `motion` | `12.42.2` | import through Mirae motion helpers where available |
| Drag and drop | `@dnd-kit/react` | `0.5.0` | use Mirae patterns and keyboard equivalents |
| Drag-and-drop helpers | `@dnd-kit/helpers` | `0.5.0` | internal to UI kit/patterns |
| List virtualization | `@tanstack/react-virtual` | `3.14.8` | long lists only |
| Resizable panels | `react-resizable-panels` | `4.12.2` | wrap behind Mirae workspace API |
| Forms | `react-hook-form` | `7.83.0` | forms and temporary drafts only |
| Icon React adapter | `@hugeicons/react` | `1.1.9` | import through Mirae `Icon` |
| Free icon set | `@hugeicons/core-free-icons` | `4.2.3` | no direct feature imports |

### UI stack decisions

- Base UI is the primitive layer.
- `@mirae/ui-kit` is the public component layer.
- Tailwind and CSS variables provide styling.
- Motion handles springs, shared layout, enter/exit, and gesture animation.
- CSS transitions handle simple hover, focus, color, and opacity changes.
- dnd-kit workflows require an equivalent keyboard workflow.
- No feature may expose the API of `react-resizable-panels` directly.
- Hugeicons are rendered through one Mirae icon component to control stroke, size, alignment, and accessibility.

---

## 6. Styling stack

| Package | Exact version | Ownership |
|---|---:|---|
| `tailwindcss` | `4.3.3` | `apps/control-ui` development |
| `@tailwindcss/vite` | `4.3.3` | `apps/control-ui` development |

The canonical design values remain:

```text
packages/ui-kit/tokens/design-tokens.v1.json
```

The generated/runtime CSS representation is:

```text
packages/ui-kit/src/styles/tokens.css
```

Feature code must not create a competing palette.

---

## 7. Test and accessibility stack

| Package | Exact version | Purpose |
|---|---:|---|
| `vitest` | `4.1.10` | unit and component test runner |
| `jsdom` | `29.1.1` | DOM test environment |
| `@testing-library/react` | `16.3.2` | React component tests |
| `@testing-library/dom` | `10.4.1` | explicit Testing Library peer |
| `@testing-library/user-event` | `14.6.1` | realistic interaction tests |
| `@playwright/test` | `1.62.0` | end-to-end and visual tests |
| `@axe-core/playwright` | `4.12.1` | accessibility checks in Playwright |

The Playwright runtime resolved in `pnpm-lock.yaml` must stay aligned with `@playwright/test` `1.62.0`.

Install the browser binaries with:

```bash
pnpm exec playwright install
```

---

## 8. Linting and formatting stack

| Package | Exact version | Purpose |
|---|---:|---|
| `eslint` | `10.8.0` | JavaScript and TypeScript lint engine |
| `@eslint/js` | `10.0.1` | official JavaScript recommended config |
| `typescript-eslint` | `8.65.0` | TypeScript parser, plugin, and flat config |
| `eslint-plugin-react-hooks` | `7.1.1` | React Hooks rules |
| `eslint-plugin-react-refresh` | `0.5.3` | Vite Fast Refresh export rules |
| `eslint-plugin-jsx-a11y` | `6.10.2` | JSX accessibility rules |
| `globals` | `17.7.0` | explicit runtime globals |
| `prettier` | `3.9.6` | source formatting |

Do not split `typescript-eslint` into separately versioned parser/plugin packages unless a dedicated tooling ticket requires it.

---

## 9. Canonical pnpm catalog

Create or update the root `pnpm-workspace.yaml` with one exact default catalog:

```yaml
packages:
  - "apps/*"
  - "packages/*"
  - "tools/*"

catalog:
  react: 19.2.8
  react-dom: 19.2.8

  typescript: 6.0.3
  vite: 8.1.5
  "@vitejs/plugin-react": 6.0.4
  "@types/react": 19.2.17
  "@types/react-dom": 19.2.3
  "@types/node": 24.13.3

  "@base-ui/react": 1.6.0
  motion: 12.42.2
  "@dnd-kit/react": 0.5.0
  "@dnd-kit/helpers": 0.5.0
  "@tanstack/react-virtual": 3.14.8
  react-resizable-panels: 4.12.2
  react-hook-form: 7.83.0
  "@hugeicons/react": 1.1.9
  "@hugeicons/core-free-icons": 4.2.3

  tailwindcss: 4.3.3
  "@tailwindcss/vite": 4.3.3

  vitest: 4.1.10
  jsdom: 29.1.1
  "@testing-library/react": 16.3.2
  "@testing-library/dom": 10.4.1
  "@testing-library/user-event": 14.6.1
  "@playwright/test": 1.62.0
  "@axe-core/playwright": 4.12.1

  eslint: 10.8.0
  "@eslint/js": 10.0.1
  typescript-eslint: 8.65.0
  eslint-plugin-react-hooks: 7.1.1
  eslint-plugin-react-refresh: 0.5.3
  eslint-plugin-jsx-a11y: 6.10.2
  globals: 17.7.0
  prettier: 3.9.6
```

Package manifests should reference catalog versions:

```json
{
  "dependencies": {
    "react": "catalog:",
    "react-dom": "catalog:"
  }
}
```

Do not duplicate literal external versions across multiple workspace package manifests.

---

## 10. Root package configuration

The root `package.json` must contain:

```json
{
  "name": "mirae",
  "private": true,
  "packageManager": "pnpm@11.17.0",
  "engines": {
    "node": "24.18.1",
    "pnpm": "11.17.0"
  }
}
```

Create `.node-version`:

```text
24.18.1
```

Create or update `.npmrc`:

```ini
save-exact=true
strict-peer-dependencies=true
shared-workspace-lockfile=true
prefer-workspace-packages=true
link-workspace-packages=true
```

---

## 11. Rust toolchain file

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.97.1"
profile = "minimal"
components = ["clippy", "rustfmt", "rust-src"]
```

Future Rust crate dependencies are not approved by this frontend lock.

When a ticket introduces a Rust crate:

1. justify the crate in the ticket;
2. add it to `[workspace.dependencies]`;
3. use an exact requirement such as `version = "=x.y.z"`;
4. commit `Cargo.lock`;
5. add the crate and version to a dedicated Rust dependency section in this file;
6. run license, security, build, and compatibility checks.

Claude must not invent the future renderer, IPC, async-runtime, serialization, or FFmpeg binding versions before their implementation tickets.

### Approved Rust dependencies

| Crate | Exact version | Introduced by | Justification |
|---|---:|---|---|
| `serde` | `1.0.229` | MIR-0012 | Derive-based serialization for generated contracts (ADR-0067). Features: `derive` only. |
| `serde_json` | `1.0.151` | MIR-0012 | The control-plane payload encoding chosen in ADR-0067. |

Transitive graph accepted with the above, all pinned by `Cargo.lock`:
`serde_derive 1.0.229`, `proc-macro2 1.0.107`, `quote 1.0.47`, `syn 3.0.3`,
`unicode-ident 1.0.24`, `itoa 1.0.18`, `memchr 2.8.3`, `ryu`.

Licence review: every crate above is dual-licensed MIT or Apache-2.0, which is
compatible with a proprietary desktop application and requires only attribution in
the distributed notices file.

Security review: `serde` and `serde_json` are the de facto standard in the Rust
ecosystem, are widely audited, and contain no `unsafe` in the paths this project
uses. `serde_json` parses untrusted input, so callers must bound the input before
parsing; the frame header in `01-runtime/108-ipc-protocol.md` section 4 does that,
and `crates/runtime/runtime/src/ipc.rs` rejects an oversized frame before
allocating. Deserialization executes no schema-supplied code, which is one of the
criteria ADR-0067 was chosen against.

Not yet approved, and each needs its own ticket: `wry` and `tao` for the desktop
window (ADR-0068), and any renderer, async-runtime, or FFmpeg binding.

---

## 12. Suggested package ownership

### `apps/control-ui/package.json`

```json
{
  "dependencies": {
    "@mirae/ui-kit": "workspace:*",
    "@tanstack/react-virtual": "catalog:",
    "react": "catalog:",
    "react-dom": "catalog:",
    "react-hook-form": "catalog:"
  },
  "devDependencies": {
    "@axe-core/playwright": "catalog:",
    "@playwright/test": "catalog:",
    "@tailwindcss/vite": "catalog:",
    "@testing-library/dom": "catalog:",
    "@testing-library/react": "catalog:",
    "@testing-library/user-event": "catalog:",
    "@types/react": "catalog:",
    "@types/react-dom": "catalog:",
    "@vitejs/plugin-react": "catalog:",
    "jsdom": "catalog:",
    "tailwindcss": "catalog:",
    "typescript": "catalog:",
    "vite": "catalog:",
    "vitest": "catalog:"
  }
}
```

### `packages/ui-kit/package.json`

```json
{
  "dependencies": {
    "@base-ui/react": "catalog:",
    "@dnd-kit/helpers": "catalog:",
    "@dnd-kit/react": "catalog:",
    "@hugeicons/core-free-icons": "catalog:",
    "@hugeicons/react": "catalog:",
    "motion": "catalog:",
    "react-resizable-panels": "catalog:"
  },
  "peerDependencies": {
    "react": "19.2.8",
    "react-dom": "19.2.8"
  },
  "devDependencies": {
    "@testing-library/dom": "catalog:",
    "@testing-library/react": "catalog:",
    "@testing-library/user-event": "catalog:",
    "@types/react": "catalog:",
    "@types/react-dom": "catalog:",
    "jsdom": "catalog:",
    "typescript": "catalog:",
    "vitest": "catalog:"
  }
}
```

### Root tooling dependencies

```json
{
  "devDependencies": {
    "@eslint/js": "catalog:",
    "@types/node": "catalog:",
    "eslint": "catalog:",
    "eslint-plugin-jsx-a11y": "catalog:",
    "eslint-plugin-react-hooks": "catalog:",
    "eslint-plugin-react-refresh": "catalog:",
    "globals": "catalog:",
    "prettier": "catalog:",
    "typescript": "catalog:",
    "typescript-eslint": "catalog:"
  }
}
```

---

## 13. Exact bootstrap commands

After the workspace packages exist, Claude may use these exact commands.

### Root tooling

```bash
pnpm add -Dw --save-exact \
  typescript@6.0.3 \
  @types/node@24.13.3 \
  eslint@10.8.0 \
  @eslint/js@10.0.1 \
  typescript-eslint@8.65.0 \
  eslint-plugin-react-hooks@7.1.1 \
  eslint-plugin-react-refresh@0.5.3 \
  eslint-plugin-jsx-a11y@6.10.2 \
  globals@17.7.0 \
  prettier@3.9.6
```

### Control UI runtime

```bash
pnpm --filter @mirae/control-ui add --save-exact \
  react@19.2.8 \
  react-dom@19.2.8 \
  @tanstack/react-virtual@3.14.8 \
  react-hook-form@7.83.0
```

### Control UI development

```bash
pnpm --filter @mirae/control-ui add -D --save-exact \
  typescript@6.0.3 \
  vite@8.1.5 \
  @vitejs/plugin-react@6.0.4 \
  @types/react@19.2.17 \
  @types/react-dom@19.2.3 \
  tailwindcss@4.3.3 \
  @tailwindcss/vite@4.3.3 \
  vitest@4.1.10 \
  jsdom@29.1.1 \
  @testing-library/react@16.3.2 \
  @testing-library/dom@10.4.1 \
  @testing-library/user-event@14.6.1 \
  @playwright/test@1.62.0 \
  @axe-core/playwright@4.12.1
```

### Mirae UI kit

```bash
pnpm --filter @mirae/ui-kit add --save-exact \
  @base-ui/react@1.6.0 \
  motion@12.42.2 \
  @dnd-kit/react@0.5.0 \
  @dnd-kit/helpers@0.5.0 \
  react-resizable-panels@4.12.2 \
  @hugeicons/react@1.1.9 \
  @hugeicons/core-free-icons@4.2.3
```

After installation, migrate literal versions to the pnpm catalog and verify that `pnpm-lock.yaml` does not change on a second frozen install.

---

## 14. Libraries that are not yet approved

Claude MUST NOT add any of these categories without a ticket and an architecture decision where necessary:

- React Router or TanStack Router;
- Zustand, Redux, Jotai, Recoil, MobX, or XState;
- TanStack Query or SWR;
- Zod, Valibot, ArkType, or Yup;
- Radix UI;
- shadcn/ui;
- Lucide;
- Sonner or another toast package;
- cmdk or another command-palette package;
- Floating UI as a direct dependency;
- charting libraries;
- rich-text editors;
- canvas frameworks;
- WebGL/Three.js wrappers;
- Electron;
- Tauri;
- Next.js;
- Storybook;
- Chromatic;
- CSS-in-JS libraries;
- a second animation library;
- a second drag-and-drop library;
- a second component primitive library.

Absence from the approved list means “not approved,” not “choose the latest.”

---

## 15. Dependency update procedure

Every version update requires a dedicated ticket, for example:

```text
MIR-DEPS-0001 — Evaluate React 19.2.9
MIR-DEPS-0002 — Evaluate TypeScript 7 migration
```

The ticket must include:

1. old and proposed versions;
2. reason for the update;
3. upstream changelog and security review;
4. peer-dependency compatibility;
5. generated-contract compatibility;
6. UI visual-regression result;
7. unit, integration, E2E, and accessibility result;
8. before/after bundle or performance data when relevant;
9. rollback plan;
10. update to this file.

Version updates must occur by coherent groups:

- React and React DOM together;
- Tailwind and `@tailwindcss/vite` together;
- Playwright packages together;
- TypeScript and typescript-eslint compatibility reviewed together;
- React runtime and React type packages reviewed together.

---

## 16. Validation commands

Claude must run:

```bash
node --version
pnpm --version
rustc --version
cargo --version
pnpm install --frozen-lockfile
pnpm list --depth 0 -r
pnpm exec tsc --version
pnpm exec vite --version
pnpm exec eslint --version
pnpm exec vitest --version
pnpm exec playwright --version
```

Expected core outputs:

```text
Node.js:    v24.18.1
pnpm:       11.17.0
Rust:       rustc 1.97.1
TypeScript: 6.0.3
Vite:       8.1.5
ESLint:     v10.8.0
Vitest:     4.1.10
Playwright: 1.62.0
```

Then run the repository checks:

```bash
cargo xtask generate --check
cargo xtask fmt --check
cargo xtask lint
cargo xtask test-affected
cargo xtask docs --check
```

Before `xtask` exists:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
pnpm -r typecheck
pnpm -r lint
pnpm -r test
pnpm -r build
```

---

## 17. Claude completion checklist

Before Claude marks a dependency/bootstrap ticket complete:

- [ ] `DEPENDENCY_VERSIONS.md` was read.
- [ ] No direct dependency uses `^`, `~`, `*`, or a release tag.
- [ ] Node is exactly `24.18.1`.
- [ ] pnpm is exactly `11.17.0`.
- [ ] Rust is exactly `1.97.1`.
- [ ] TypeScript is exactly `6.0.3`, not 7.x.
- [ ] React and React DOM are both exactly `19.2.8`.
- [ ] The pnpm catalog contains the exact approved versions.
- [ ] `pnpm-lock.yaml` is committed.
- [ ] A frozen reinstall succeeds without changing the lockfile.
- [ ] No unapproved library was added.
- [ ] Peer-dependency warnings are zero.
- [ ] The relevant tests and builds pass.
- [ ] Any approved change to versions updated this document.
