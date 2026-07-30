/**
 * The transport the running application uses to reach the engine.
 *
 * Canonical documentation: `docs/01-runtime/108-ipc-protocol.md`,
 * `docs/05-platform/501-desktop-shell.md` section 3.
 *
 * There is no endpoint yet. The engine speaks no IPC until `MIR-0012`, and the
 * shell hosts no webview to bridge through, so a browser cannot reach it. Rather
 * than fake a connection and show a green badge that means nothing, this
 * transport fails with the reason, which drives the real reconnect and failure
 * states in the UI.
 *
 * Tests use `@mirae/test-utils`, whose fake transport exercises the connected and
 * degraded paths through the same client code.
 */

import type { EngineTransport } from "@mirae/client";
import type { EngineReadiness } from "@mirae/contracts";

/** Why no connection can be established yet. */
const REASON = "the engine IPC endpoint arrives with MIR-0012";

/** Build the transport for this build of the application. */
export function createEngineTransport(): EngineTransport {
  return {
    connect(): Promise<EngineReadiness> {
      return Promise.reject(new Error(REASON));
    },
    onDisconnect(): void {
      // Nothing can drop, because nothing connects.
    },
    close(): void {
      // Nothing to close.
    },
  };
}
