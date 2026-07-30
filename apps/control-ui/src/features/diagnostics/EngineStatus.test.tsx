/**
 * Behaviour, keyboard, and accessibility tests for the engine status panel.
 *
 * Canonical documentation:
 * `docs/08-development/809-testing-and-validation-workflow.md` section 3 (UI),
 * `docs/09-ui-ux/923-accessibility-and-reduced-motion.md`.
 */

import { EngineConnection } from "@mirae/client";
import {
  FakeEngineTransport,
  ManualScheduler,
  fakeReadiness,
} from "@mirae/test-utils";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import { EngineStatus } from "./EngineStatus";

afterEach(cleanup);

function build(transport: FakeEngineTransport) {
  const scheduler = new ManualScheduler();
  const connection = new EngineConnection(transport, { scheduler });

  return { connection, scheduler };
}

describe("EngineStatus", () => {
  it("says it is not connected before anything happens", () => {
    const { connection } = build(new FakeEngineTransport());

    render(<EngineStatus connection={connection} />);

    expect(screen.getByRole("status")).toHaveTextContent("Not connected");
    expect(
      screen.getByText("No engine details while disconnected."),
    ).toBeDefined();
  });

  it("shows the engine version, state, and session once connected", async () => {
    const { connection } = build(new FakeEngineTransport());

    render(<EngineStatus connection={connection} />);
    await connection.connect();

    expect(await screen.findByText("Connected")).toBeDefined();
    expect(screen.getByText("1.0")).toBeDefined();
    expect(screen.getByText("ready")).toBeDefined();
    expect(screen.getByText("0000000000000000000000000000002a")).toBeDefined();
  });

  it("reports a degraded engine and its reason rather than a bare ok", async () => {
    const transport = new FakeEngineTransport({
      readiness: fakeReadiness({ state: "degraded", detail: "no GPU adapter" }),
    });
    const { connection } = build(transport);

    render(<EngineStatus connection={connection} />);
    await connection.connect();

    expect(await screen.findByText("Connected, engine degraded")).toBeDefined();
    expect(screen.getByText("no GPU adapter")).toBeDefined();
  });

  it("explains the retry and how long it will take", async () => {
    const transport = new FakeEngineTransport({ failAttempts: 99 });
    const { connection } = build(transport);

    render(<EngineStatus connection={connection} />);
    await connection.connect();

    expect(await screen.findByText("Reconnecting to the engine")).toBeDefined();
    expect(
      screen.getByText(/Retrying in 0.5 seconds \(attempt 1\)/),
    ).toBeDefined();
    expect(screen.getByText(/the engine is not running/)).toBeDefined();
  });

  it("stops showing engine details when a live connection drops", async () => {
    // 803 invariant 2: never present stale engine state as current.
    const transport = new FakeEngineTransport();
    const { connection } = build(transport);

    render(<EngineStatus connection={connection} />);
    await connection.connect();
    expect(await screen.findByText("ready")).toBeDefined();

    transport.dropConnection("the engine exited");

    expect(
      await screen.findByText("No engine details while disconnected."),
    ).toBeDefined();
    expect(screen.queryByText("0000000000000000000000000000002a")).toBeNull();
  });

  it("offers a working retry once the engine is unavailable", async () => {
    const transport = new FakeEngineTransport({ failAttempts: 99 });
    const { connection, scheduler } = build(transport);

    render(<EngineStatus connection={connection} />);
    await connection.connect();

    for (let round = 0; round < 10 && scheduler.pending > 0; round += 1) {
      scheduler.runPending();
      await Promise.resolve();
      await Promise.resolve();
    }

    const button = await screen.findByRole("button", { name: "Try again" });
    transport.failNext(0);
    await userEvent.click(button);

    expect(await screen.findByText("Connected")).toBeDefined();
  });

  it("reaches the retry control by keyboard", async () => {
    // 923: every control is keyboard reachable and operable.
    const transport = new FakeEngineTransport({ failAttempts: 99 });
    const { connection, scheduler } = build(transport);

    render(<EngineStatus connection={connection} />);
    await connection.connect();
    for (let round = 0; round < 10 && scheduler.pending > 0; round += 1) {
      scheduler.runPending();
      await Promise.resolve();
      await Promise.resolve();
    }

    await screen.findByRole("button", { name: "Try again" });
    transport.failNext(0);

    await userEvent.tab();
    expect(document.activeElement).toBe(
      screen.getByRole("button", { name: "Try again" }),
    );

    await userEvent.keyboard("{Enter}");

    expect(await screen.findByText("Connected")).toBeDefined();
  });

  it("announces status changes politely and labels its region", () => {
    const { connection } = build(new FakeEngineTransport());

    render(<EngineStatus connection={connection} />);

    const status = screen.getByRole("status");

    expect(status.getAttribute("aria-live")).toBe("polite");
    expect(screen.getByRole("region", { name: "Engine" })).toBeDefined();
  });

  it("never conveys status by colour alone", () => {
    const { connection } = build(new FakeEngineTransport());

    render(<EngineStatus connection={connection} />);

    // The coloured dot is decorative; the words carry the meaning.
    const status = screen.getByRole("status");

    expect(status.textContent?.trim()).toBe("Not connected");
  });
});
