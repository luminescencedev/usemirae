import { describe, expect, it } from "vitest";

import {
  FakeEngineTransport,
  ManualScheduler,
  fakeReadiness,
} from "@mirae/test-utils";

import {
  DEFAULT_RECONNECT_POLICY,
  EngineConnection,
  retryDelayMs,
  type ConnectionSnapshot,
} from "../src/index";

/** Build a connection with a manual clock, so nothing waits. */
function build(transport: FakeEngineTransport) {
  const scheduler = new ManualScheduler();
  const connection = new EngineConnection(transport, { scheduler });
  const seen: ConnectionSnapshot[] = [];
  connection.subscribe((snapshot) => seen.push(snapshot));

  return { connection, scheduler, seen };
}

describe("connecting", () => {
  it("starts idle and knows nothing about the engine", () => {
    const { connection } = build(new FakeEngineTransport());

    expect(connection.snapshot().phase).toBe("idle");
    expect(connection.snapshot().readiness).toBeNull();
  });

  it("reports the engine's readiness once connected", async () => {
    const transport = new FakeEngineTransport({
      readiness: fakeReadiness({ protocolMajor: 1, protocolMinor: 0 }),
    });
    const { connection } = build(transport);

    await connection.connect();

    const snapshot = connection.snapshot();

    expect(snapshot.phase).toBe("connected");
    expect(snapshot.readiness?.protocolMajor).toBe(1);
    expect(snapshot.readiness?.engineSessionId).toBe(
      "0000000000000000000000000000002a",
    );
    expect(snapshot.lastError).toBeNull();
  });

  it("passes through a degraded engine rather than hiding it", async () => {
    const transport = new FakeEngineTransport({
      readiness: fakeReadiness({
        state: "degraded",
        detail: "no GPU adapter",
      }),
    });
    const { connection } = build(transport);

    await connection.connect();

    expect(connection.snapshot().readiness?.state).toBe("degraded");
    expect(connection.snapshot().readiness?.detail).toBe("no GPU adapter");
  });
});

describe("reconnecting", () => {
  it("retries a failed attempt after a backoff and reports the delay", async () => {
    const transport = new FakeEngineTransport({ failAttempts: 1 });
    const { connection, scheduler } = build(transport);

    await connection.connect();

    expect(connection.snapshot().phase).toBe("reconnecting");
    expect(connection.snapshot().nextRetryDelayMs).toBe(
      retryDelayMs(2, DEFAULT_RECONNECT_POLICY),
    );
    expect(connection.snapshot().lastError).toBe("the engine is not running");
    expect(scheduler.pending).toBe(1);

    scheduler.runPending();
    await Promise.resolve();
    await Promise.resolve();

    expect(connection.snapshot().phase).toBe("connected");
    expect(transport.attempts).toBe(2);
  });

  it("backs off exponentially and stops at the ceiling", () => {
    expect(retryDelayMs(1)).toBe(250);
    expect(retryDelayMs(2)).toBe(500);
    expect(retryDelayMs(3)).toBe(1_000);
    expect(retryDelayMs(10)).toBe(DEFAULT_RECONNECT_POLICY.maxDelayMs);
  });

  it("gives up after the bounded number of attempts", async () => {
    const transport = new FakeEngineTransport({ failAttempts: 99 });
    const { connection, scheduler } = build(transport);

    await connection.connect();

    for (let round = 0; round < 10 && scheduler.pending > 0; round += 1) {
      scheduler.runPending();
      await Promise.resolve();
      await Promise.resolve();
    }

    expect(connection.snapshot().phase).toBe("unavailable");
    expect(transport.attempts).toBe(DEFAULT_RECONNECT_POLICY.maxAttempts);
    expect(scheduler.pending).toBe(0);
    expect(connection.snapshot().lastError).toBe("the engine is not running");
  });

  it("clears the readiness when an established connection drops", async () => {
    // 803 invariant 2: the UI must not present stale engine state as current.
    const transport = new FakeEngineTransport();
    const { connection } = build(transport);

    await connection.connect();
    expect(connection.snapshot().readiness).not.toBeNull();

    transport.dropConnection("the engine exited");

    expect(connection.snapshot().readiness).toBeNull();
    expect(connection.snapshot().phase).toBe("reconnecting");
    expect(connection.snapshot().lastError).toBe("the engine exited");
  });

  it("does not keep a stale readiness while retrying", async () => {
    const transport = new FakeEngineTransport({ failAttempts: 1 });
    const { connection, seen } = build(transport);

    await connection.connect();

    expect(
      seen.every(
        (snapshot) =>
          snapshot.phase === "connected" || snapshot.readiness === null,
      ),
    ).toBe(true);
  });

  it("retryNow starts a fresh budget after giving up", async () => {
    const transport = new FakeEngineTransport({ failAttempts: 99 });
    const { connection, scheduler } = build(transport);

    await connection.connect();
    for (let round = 0; round < 10 && scheduler.pending > 0; round += 1) {
      scheduler.runPending();
      await Promise.resolve();
      await Promise.resolve();
    }
    expect(connection.snapshot().phase).toBe("unavailable");

    transport.failNext(0);
    await connection.retryNow();

    expect(connection.snapshot().phase).toBe("connected");
    expect(connection.snapshot().attempt).toBe(1);
  });
});

describe("disconnecting", () => {
  it("closes the transport and forgets everything", async () => {
    const transport = new FakeEngineTransport();
    const { connection } = build(transport);

    await connection.connect();
    connection.disconnect();

    expect(transport.closed).toBe(true);
    expect(connection.snapshot().phase).toBe("idle");
    expect(connection.snapshot().readiness).toBeNull();
  });

  it("cancels a pending retry so a stopped connection stays stopped", async () => {
    const transport = new FakeEngineTransport({ failAttempts: 99 });
    const { connection, scheduler } = build(transport);

    await connection.connect();
    expect(scheduler.pending).toBe(1);

    connection.disconnect();

    expect(scheduler.pending).toBe(0);

    scheduler.runPending();
    await Promise.resolve();

    expect(connection.snapshot().phase).toBe("idle");
  });

  it("ignores a drop that arrives after disconnecting", async () => {
    const transport = new FakeEngineTransport();
    const { connection } = build(transport);

    await connection.connect();
    connection.disconnect();
    transport.dropConnection();

    expect(connection.snapshot().phase).toBe("idle");
  });
});

describe("subscribers", () => {
  it("receive every phase change and can unsubscribe", async () => {
    const transport = new FakeEngineTransport();
    const { connection, seen } = build(transport);

    await connection.connect();

    expect(seen.map((snapshot) => snapshot.phase)).toEqual([
      "connecting",
      "connected",
    ]);

    const unsubscribe = connection.subscribe(() => {
      throw new Error("this listener was removed and must not run");
    });
    unsubscribe();

    connection.disconnect();

    expect(seen.at(-1)?.phase).toBe("idle");
  });
});
