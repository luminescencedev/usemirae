/**
 * Typed engine client: connection state, reconnect logic, and the transport
 * boundary.
 *
 * Canonical documentation:
 * `docs/08-development/803-frontend-workspace-and-packages.md`.
 *
 * This package does not become engine authority (`803` invariant 2). It reports
 * what the engine said and when it last said it; it never invents state, and it
 * clears what it knows the moment the connection drops.
 *
 * The real transport arrives with `MIR-0012`. Until then, `@mirae/test-utils`
 * provides a fake one so the UI and its tests exercise the same code path.
 */

export {
  DEFAULT_RECONNECT_POLICY,
  EngineConnection,
  retryDelayMs,
  timerScheduler,
  type ConnectionPhase,
  type ConnectionSnapshot,
  type EngineTransport,
  type ReconnectPolicy,
  type Scheduler,
} from "./connection";
