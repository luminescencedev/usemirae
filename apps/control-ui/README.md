# @mirae/control-ui

The operator interface. Canonical documentation:

- `docs/08-development/803-frontend-workspace-and-packages.md`
- `docs/09-ui-ux/903-information-architecture.md`
- `docs/09-ui-ux/904-desktop-shell-layout.md`
- `docs/09-ui-ux/927-screen-inventory.md`

## Layout

```text
src/
├── app/          application root and providers
├── features/     project, scenes, sources, audio, outputs, settings, diagnostics
├── components/   app-local composition of @mirae/ui-kit
├── stores/       UI draft and high-frequency metric stores
├── routes/       route definitions
├── hooks/
└── styles/       index.css (Tailwind + ui-kit tokens)
```

## Boundaries

This app must not read project files, store credentials, invoke native APIs,
create media pipelines, duplicate project validation, infer capability from OS
name, or treat local React state as authoritative project state.

## Commands

```bash
pnpm --filter @mirae/control-ui dev
pnpm --filter @mirae/control-ui typecheck
pnpm --filter @mirae/control-ui test
pnpm --filter @mirae/control-ui build
```
