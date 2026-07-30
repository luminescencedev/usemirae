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
import { useEffect, useState } from "react";

import { EngineStatus } from "../features/diagnostics/EngineStatus";
import { ProjectPanel } from "../features/project/ProjectPanel";
import { createEngineTransport } from "../engine-transport";

export function App() {
  // One connection for the lifetime of the app. `useState` with an initializer
  // keeps it stable across renders without a module-level singleton, which would
  // outlive a hot reload and leak listeners.
  const [connection] = useState(
    () => new EngineConnection(createEngineTransport()),
  );

  // Connect on mount. Until MIR-0116 there was nothing to connect to, so the
  // panel sat in its idle state waiting for a button nobody had a reason to
  // press; now the shell answers, and a control UI that shows the engine as
  // disconnected until asked would be reporting its own inaction as engine
  // state.
  useEffect(() => {
    void connection.connect();

    return () => {
      connection.disconnect();
    };
  }, [connection]);

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
        <ProjectPanel />
      </div>
    </main>
  );
}
