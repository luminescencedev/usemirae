/**
 * The transport the running application uses to reach the engine.
 *
 * Canonical documentation: `docs/01-runtime/108-ipc-protocol.md`,
 * `docs/05-platform/501-desktop-shell.md` section 3.
 *
 * Inside the shell, this asks the bridge for engine status (`MIR-0116`). The
 * webview never reaches the engine socket: it asks the shell, which already
 * holds the authenticated connection, and reports what the shell observed.
 *
 * In a browser tab there is no shell, so there is no engine. The transport fails
 * with that reason rather than faking a connection, which drives the real
 * reconnect and failure states instead of showing a green badge that means
 * nothing.
 *
 * Tests use `@mirae/test-utils`, whose fake transport exercises the connected and
 * degraded paths through the same client code.
 */

import type { EngineTransport } from "@mirae/client";
import type { EngineReadiness } from "@mirae/contracts";

import { bridge, isHosted } from "./bridge";

/** Why no connection can be established outside the shell. */
const NOT_HOSTED = "the control UI is running outside the Mirae shell";

/** Build the transport for this build of the application. */
export function createEngineTransport(): EngineTransport {
  return {
    async connect(): Promise<EngineReadiness> {
      if (!isHosted()) {
        throw new Error(NOT_HOSTED);
      }

      const response = await bridge().request("engineStatus");

      if (!response.ok) {
        throw new Error(response.errorCode);
      }

      if (!response.engineConnected) {
        // The shell is there and the engine is not. Reported as a failure so
        // the connection's own reconnect policy handles it, rather than as a
        // readiness that claims something untrue.
        throw new Error("the shell holds no engine connection");
      }

      return {
        state: "ready",
        engineSessionId: response.engineSessionId,
        protocolMajor: response.protocolMajor,
        protocolMinor: response.protocolMinor,
      } satisfies EngineReadiness;
    },
    onDisconnect(): void {
      // The bridge is request-response today. The shell closes the window when
      // the engine fails, so a drop the page could observe separately does not
      // exist yet; when the bridge gains events, this registers for them.
    },
    close(): void {
      // Nothing to close: each request is independent.
    },
  };
}
