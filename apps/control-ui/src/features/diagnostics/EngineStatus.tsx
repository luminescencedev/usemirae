/**
 * Shows engine connection, version, process state, and reconnect behavior.
 *
 * Canonical documentation:
 * - `docs/09-ui-ux/922-loading-empty-failure-and-recovery-states.md`
 * - `docs/09-ui-ux/923-accessibility-and-reduced-motion.md`
 * - `docs/08-development/803-frontend-workspace-and-packages.md` section 6
 *
 * The component renders the client's snapshot and nothing else. It holds no
 * engine truth of its own: when the connection reports no readiness, it says so
 * rather than showing the last value it saw.
 */

import type { EngineConnection, ConnectionSnapshot } from "@mirae/client";
import { StatusBadge, type StatusTone } from "@mirae/ui-kit";
import { useSyncExternalStore } from "react";

/** Props for {@link EngineStatus}. */
export interface EngineStatusProps {
  /** The connection to render. */
  readonly connection: EngineConnection;
}

/**
 * Subscribe to a connection and re-render on each snapshot.
 *
 * `useSyncExternalStore` is the right primitive here: the connection is an
 * external store, and mirroring it into local state would both duplicate engine
 * truth and cause a cascading render.
 */
function useConnectionSnapshot(
  connection: EngineConnection,
): ConnectionSnapshot {
  return useSyncExternalStore(
    (onChange) => connection.subscribe(onChange),
    () => connection.snapshot(),
  );
}

/** The tone and words for a phase. Words never depend on colour alone. */
function describePhase(snapshot: ConnectionSnapshot): {
  readonly tone: StatusTone;
  readonly label: string;
} {
  switch (snapshot.phase) {
    case "idle":
      return { tone: "neutral", label: "Not connected" };
    case "connecting":
      return { tone: "neutral", label: "Connecting to the engine" };
    case "reconnecting":
      return { tone: "caution", label: "Reconnecting to the engine" };
    case "unavailable":
      return { tone: "critical", label: "Engine unavailable" };
    case "connected":
      break;
  }

  // Connected: the engine's own state is the more precise answer.
  switch (snapshot.readiness?.state) {
    case "degraded":
      return { tone: "caution", label: "Connected, engine degraded" };
    case "starting":
      return { tone: "neutral", label: "Connected, engine starting" };
    case "stopping":
    case "stopped":
      return { tone: "caution", label: "Connected, engine stopping" };
    default:
      return { tone: "positive", label: "Connected" };
  }
}

/** A panel describing the engine connection and what happens next. */
export function EngineStatus({ connection }: EngineStatusProps) {
  const snapshot = useConnectionSnapshot(connection);
  const { tone, label } = describePhase(snapshot);
  const readiness = snapshot.readiness;

  return (
    <section
      aria-labelledby="engine-status-heading"
      style={{
        background: "var(--mirae-surface)",
        border: "1px solid var(--mirae-border)",
        borderRadius: "var(--mirae-radius-panel)",
        padding: "20px 24px",
        display: "grid",
        gap: "12px",
        minWidth: "36ch",
      }}
    >
      <h2
        id="engine-status-heading"
        style={{ font: "600 15px/1.4 inherit", margin: 0 }}
      >
        Engine
      </h2>

      {/* Polite, so a screen reader announces a change without interrupting. */}
      <p role="status" aria-live="polite" style={{ margin: 0 }}>
        <StatusBadge tone={tone}>{label}</StatusBadge>
      </p>

      {readiness ? (
        <dl
          style={{
            display: "grid",
            gridTemplateColumns: "auto 1fr",
            gap: "4px 16px",
            margin: 0,
            color: "var(--mirae-fg-secondary)",
            font: "400 13px/1.4 inherit",
          }}
        >
          <dt>Protocol</dt>
          <dd style={{ margin: 0 }}>
            {readiness.protocolMajor}.{readiness.protocolMinor}
          </dd>
          <dt>Engine state</dt>
          <dd style={{ margin: 0 }}>{readiness.state}</dd>
          <dt>Session</dt>
          <dd
            style={{
              margin: 0,
              fontFamily: "var(--mirae-font-mono, monospace)",
            }}
          >
            {readiness.engineSessionId}
          </dd>
        </dl>
      ) : (
        <p
          style={{
            margin: 0,
            color: "var(--mirae-fg-muted)",
            font: "400 13px/1.4 inherit",
          }}
        >
          No engine details while disconnected.
        </p>
      )}

      {readiness?.detail ? (
        <p
          style={{
            margin: 0,
            color: "var(--mirae-fg-secondary)",
            font: "400 13px/1.4 inherit",
          }}
        >
          {readiness.detail}
        </p>
      ) : null}

      {snapshot.lastError ? (
        <p
          style={{
            margin: 0,
            color: "var(--mirae-fg-muted)",
            font: "400 13px/1.4 inherit",
          }}
        >
          Last error: {snapshot.lastError}
        </p>
      ) : null}

      {snapshot.nextRetryDelayMs !== null ? (
        <p
          style={{
            margin: 0,
            color: "var(--mirae-fg-muted)",
            font: "400 13px/1.4 inherit",
          }}
        >
          Retrying in {Math.round(snapshot.nextRetryDelayMs / 100) / 10} seconds
          (attempt {snapshot.attempt}).
        </p>
      ) : null}

      {snapshot.phase === "unavailable" || snapshot.phase === "idle" ? (
        <button
          type="button"
          onClick={() => {
            void connection.retryNow();
          }}
          style={{
            justifySelf: "start",
            background: "var(--mirae-accent)",
            color: "var(--mirae-accent-fg, #fff)",
            border: "none",
            borderRadius: "var(--mirae-radius-control)",
            minHeight: "var(--mirae-control)",
            padding: "0 14px",
            font: "500 13px/1 inherit",
            cursor: "pointer",
          }}
        >
          Try again
        </button>
      ) : null}
    </section>
  );
}
