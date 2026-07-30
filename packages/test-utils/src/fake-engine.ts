/**
 * A fake engine transport and a controllable scheduler.
 *
 * Canonical documentation:
 * `docs/08-development/803-frontend-workspace-and-packages.md` sections 3 and 8.
 *
 * The real transport arrives with `MIR-0012`. Until then this is what the control
 * UI runs against, in tests and in development, so the connection code exercised
 * by a test is the code that ships.
 */

import type { EngineTransport, Scheduler } from "@mirae/client";
import type { EngineReadiness } from "@mirae/contracts";

/** A readiness report with sensible defaults, overridable per field. */
export function fakeReadiness(
  overrides: Partial<EngineReadiness> = {},
): EngineReadiness {
  return {
    state: "ready",
    protocolMajor: 1,
    protocolMinor: 0,
    engineSessionId: "0000000000000000000000000000002a",
    ...overrides,
  };
}

/** How the fake transport should behave on the next connection attempt. */
export interface FakeEngineOptions {
  /** Readiness to resolve with. Defaults to a ready engine. */
  readonly readiness?: EngineReadiness;
  /** Fail this many attempts before succeeding. */
  readonly failAttempts?: number;
  /** The message failed attempts reject with. */
  readonly failureMessage?: string;
}

/** A transport that never touches a socket. */
export class FakeEngineTransport implements EngineTransport {
  #readiness: EngineReadiness;
  #remainingFailures: number;
  #failureMessage: string;
  #disconnectListeners: ((reason: string) => void)[] = [];

  /** How many times a connection was attempted. */
  attempts = 0;
  /** Whether `close` was called. */
  closed = false;

  constructor(options: FakeEngineOptions = {}) {
    this.#readiness = options.readiness ?? fakeReadiness();
    this.#remainingFailures = options.failAttempts ?? 0;
    this.#failureMessage =
      options.failureMessage ?? "the engine is not running";
  }

  connect(): Promise<EngineReadiness> {
    this.attempts += 1;

    if (this.#remainingFailures > 0) {
      this.#remainingFailures -= 1;
      return Promise.reject(new Error(this.#failureMessage));
    }

    return Promise.resolve(this.#readiness);
  }

  onDisconnect(listener: (reason: string) => void): void {
    this.#disconnectListeners.push(listener);
  }

  close(): void {
    this.closed = true;
  }

  /** Simulate an established connection dropping. */
  dropConnection(reason = "the engine exited"): void {
    for (const listener of this.#disconnectListeners) {
      listener(reason);
    }
  }

  /** Change what the next successful attempt reports. */
  setReadiness(readiness: EngineReadiness): void {
    this.#readiness = readiness;
  }

  /** Make the next `count` attempts fail. */
  failNext(count: number, message?: string): void {
    this.#remainingFailures = count;
    if (message !== undefined) {
      this.#failureMessage = message;
    }
  }
}

/** A scheduler whose timers only fire when a test says so. */
export class ManualScheduler implements Scheduler {
  #tasks: { task: () => void; delayMs: number }[] = [];

  schedule(task: () => void, delayMs: number): () => void {
    const entry = { task, delayMs };
    this.#tasks.push(entry);

    return () => {
      this.#tasks = this.#tasks.filter((candidate) => candidate !== entry);
    };
  }

  /** How many timers are waiting. */
  get pending(): number {
    return this.#tasks.length;
  }

  /** The delay of the next waiting timer, if there is one. */
  get nextDelayMs(): number | null {
    return this.#tasks[0]?.delayMs ?? null;
  }

  /** Fire every waiting timer once, oldest first. */
  runPending(): void {
    const tasks = this.#tasks;
    this.#tasks = [];

    for (const { task } of tasks) {
      task();
    }
  }
}
