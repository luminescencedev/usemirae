/**
 * Application root.
 *
 * Placeholder created by `MIR-0001 — Initialize monorepo`. Engine connection,
 * version, process state, and reconnect behavior arrive with `MIR-0011`.
 * The desktop shell layout is specified in `docs/09-ui-ux/904-desktop-shell-layout.md`.
 */
export function App() {
  return (
    <main
      style={{
        display: "grid",
        placeItems: "center",
        height: "100%",
        padding: "24px",
      }}
    >
      <section
        style={{
          background: "var(--mirae-surface)",
          border: "1px solid var(--mirae-border)",
          borderRadius: "var(--mirae-radius-panel)",
          padding: "24px 28px",
          maxWidth: "44ch",
        }}
      >
        <h1 style={{ font: "600 15px/1.4 inherit", margin: 0 }}>Mirae</h1>
        <p style={{ color: "var(--mirae-fg-muted)", margin: "8px 0 0" }}>
          Control UI scaffold. No engine connection yet — that arrives with
          MIR-0011.
        </p>
      </section>
    </main>
  );
}
