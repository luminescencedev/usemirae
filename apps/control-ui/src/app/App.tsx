/**
 * Application root.
 *
 * Canonical documentation:
 * - `docs/09-ui-ux/904-desktop-shell-layout.md`
 * - `docs/08-development/803-frontend-workspace-and-packages.md`
 *
 * `MIR-0011` shows engine connection, version, process state, and reconnect
 * behavior. The full workspace layout arrives with the UI implementation backlog.
 */

import { EngineConnection } from "@mirae/client";
import { useState } from "react";

import { EngineStatus } from "../features/diagnostics/EngineStatus";
import { createEngineTransport } from "../engine-transport";

export function App() {
  // One connection for the lifetime of the app. `useState` with an initializer
  // keeps it stable across renders without a module-level singleton, which would
  // outlive a hot reload and leak listeners.
  const [connection] = useState(
    () => new EngineConnection(createEngineTransport()),
  );

  return (
    <main
      style={{
        display: "grid",
        placeItems: "center",
        height: "100%",
        padding: "24px",
      }}
    >
      <div style={{ display: "grid", gap: "16px", justifyItems: "center" }}>
        <h1 style={{ font: "600 24px/1.3 inherit", margin: 0 }}>Mirae</h1>
        <EngineStatus connection={connection} />
      </div>
    </main>
  );
}
