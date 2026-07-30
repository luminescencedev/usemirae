/**
 * Engine connection state and reconnect policy.
 *
 * Canonical documentation:
 * - `docs/08-development/803-frontend-workspace-and-packages.md` sections 3 and 5
 * - `docs/01-runtime/109-ui-engine-synchronization.md`
 * - `docs/09-ui-ux/922-loading-empty-failure-and-recovery-states.md`
 *
 * Two rules shape this module:
 *
 * - the UI never owns engine truth, so a disconnect clears the last readiness
 *   instead of remembering it. A stale "ready" badge during an outage is worse
 *   than an honest "reconnecting";
 * - reconnection is bounded, so a dead engine produces a final, actionable state
 *   rather than an endless spinner.
 *
 * Time and transport are injected, so every delay and failure path is unit tested
 * without waiting.
 */

import type { EngineReadiness } from "@mirae/contracts";

/** Where the connection is right now. */
export type ConnectionPhase =
  "idle" | "connecting" | "connected" | "reconnecting" | "unavailable";

/** What the UI renders. */
export interface ConnectionSnapshot {
  readonly phase: ConnectionPhase;
  /**
   * The engine's last report, or `null` whenever it is not connected.
   *
   * Cleared on disconnect on purpose: the UI shows what is true now.
   */
  readonly readiness: EngineReadiness | null;
  /** How many connection attempts have been made in this run. */
  readonly attempt: number;
  /** Milliseconds until the next automatic attempt, when one is scheduled. */
  readonly nextRetryDelayMs: number | null;
  /** A safe reason for the current phase, when there is one. */
  readonly lastError: string | null;
}

/** How reconnection backs off before giving up. */
export interface ReconnectPolicy {
  /** Delay before the first retry. */
  readonly baseDelayMs: number;
  /** Multiplier applied to each successive delay. */
  readonly factor: number;
  /** Upper bound on any single delay. */
  readonly maxDelayMs: number;
  /** Attempts after the first failure before the connection is unavailable. */
  readonly maxAttempts: number;
}

/** A bounded default: five attempts, backing off from 250 ms to 8 s. */
export const DEFAULT_RECONNECT_POLICY: ReconnectPolicy = {
  baseDelayMs: 250,
  factor: 2,
  maxDelayMs: 8_000,
  maxAttempts: 5,
};

/** Whatever actually talks to the engine. */
export interface EngineTransport {
  /** Attempt one connection, resolving with the engine's readiness. */
  connect(): Promise<EngineReadiness>;
  /** Register the callback invoked when an established connection drops. */
  onDisconnect(listener: (reason: string) => void): void;
  /** Close the connection, if one is open. */
  close(): void;
}

/** Schedules deferred work. Injected so tests control time. */
export interface Scheduler {
  /** Run `task` after `delayMs`, returning a cancel function. */
  schedule(task: () => void, delayMs: number): () => void;
}

/** A scheduler backed by the platform timer. */
export const timerScheduler: Scheduler = {
  schedule(task, delayMs) {
    const handle = setTimeout(task, delayMs);
    return () => clearTimeout(handle);
  },
};

/**
 * Compute the delay before attempt number `attempt`, counting from 1.
 *
 * Exported because the delay is user-visible: `922` asks a failure state to say
 * what happens next, which means the UI needs the number.
 */
export function retryDelayMs(
  attempt: number,
  policy: ReconnectPolicy = DEFAULT_RECONNECT_POLICY,
): number {
  const exponent = Math.max(0, attempt - 1);
  const delay = policy.baseDelayMs * policy.factor ** exponent;

  return Math.min(delay, policy.maxDelayMs);
}

/** Drives connection and reconnection, and publishes snapshots. */
export class EngineConnection {
  #transport: EngineTransport;
  #policy: ReconnectPolicy;
  #scheduler: Scheduler;
  #listeners = new Set<(snapshot: ConnectionSnapshot) => void>();
  #cancelPending: (() => void) | null = null;
  #stopped = false;
  #snapshot: ConnectionSnapshot = {
    phase: "idle",
    readiness: null,
    attempt: 0,
    nextRetryDelayMs: null,
    lastError: null,
  };

  constructor(
    transport: EngineTransport,
    options: {
      readonly policy?: ReconnectPolicy;
      readonly scheduler?: Scheduler;
    } = {},
  ) {
    this.#transport = transport;
    this.#policy = options.policy ?? DEFAULT_RECONNECT_POLICY;
    this.#scheduler = options.scheduler ?? timerScheduler;

    this.#transport.onDisconnect((reason) => {
      this.#handleDisconnect(reason);
    });
  }

  /** The current snapshot. */
  snapshot(): ConnectionSnapshot {
    return this.#snapshot;
  }

  /** Subscribe to snapshots. Returns an unsubscribe function. */
  subscribe(listener: (snapshot: ConnectionSnapshot) => void): () => void {
    this.#listeners.add(listener);
    return () => this.#listeners.delete(listener);
  }

  /** Start connecting. Safe to call once; later calls are ignored while active. */
  async connect(): Promise<void> {
    if (
      this.#snapshot.phase === "connecting" ||
      this.#snapshot.phase === "connected"
    ) {
      return;
    }

    this.#stopped = false;
    await this.#attempt(1);
  }

  /**
   * Retry immediately, resetting the attempt budget.
   *
   * This is the "try again" the failure state offers a user (`922`), so it starts
   * a fresh budget rather than continuing an exhausted one.
   */
  async retryNow(): Promise<void> {
    this.#cancelPending?.();
    this.#cancelPending = null;
    this.#stopped = false;
    await this.#attempt(1);
  }

  /** Stop connecting and close the transport. */
  disconnect(): void {
    this.#stopped = true;
    this.#cancelPending?.();
    this.#cancelPending = null;
    this.#transport.close();
    this.#publish({
      phase: "idle",
      readiness: null,
      attempt: 0,
      nextRetryDelayMs: null,
      lastError: null,
    });
  }

  async #attempt(attempt: number): Promise<void> {
    if (this.#stopped) {
      return;
    }

    this.#publish({
      phase: attempt === 1 ? "connecting" : "reconnecting",
      // Whatever the engine last said is not true while reconnecting.
      readiness: null,
      attempt,
      nextRetryDelayMs: null,
      lastError: this.#snapshot.lastError,
    });

    try {
      const readiness = await this.#transport.connect();

      if (this.#stopped) {
        return;
      }

      this.#publish({
        phase: "connected",
        readiness,
        attempt,
        nextRetryDelayMs: null,
        lastError: null,
      });
    } catch (error) {
      this.#scheduleRetry(attempt, describe(error));
    }
  }

  #scheduleRetry(attempt: number, reason: string): void {
    if (this.#stopped) {
      return;
    }

    if (attempt >= this.#policy.maxAttempts) {
      this.#publish({
        phase: "unavailable",
        readiness: null,
        attempt,
        nextRetryDelayMs: null,
        lastError: reason,
      });
      return;
    }

    const next = attempt + 1;
    const delay = retryDelayMs(next, this.#policy);

    this.#publish({
      phase: "reconnecting",
      readiness: null,
      attempt,
      nextRetryDelayMs: delay,
      lastError: reason,
    });

    this.#cancelPending = this.#scheduler.schedule(() => {
      void this.#attempt(next);
    }, delay);
  }

  #handleDisconnect(reason: string): void {
    if (this.#stopped) {
      return;
    }

    // An established connection dropped: clear the readiness and start a fresh
    // budget, because this is a new outage rather than a continuing one.
    this.#scheduleRetry(1, reason);
  }

  #publish(snapshot: ConnectionSnapshot): void {
    this.#snapshot = snapshot;
    for (const listener of this.#listeners) {
      listener(snapshot);
    }
  }
}

/** Turn an unknown thrown value into a safe message. */
function describe(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return "the engine could not be reached";
}
