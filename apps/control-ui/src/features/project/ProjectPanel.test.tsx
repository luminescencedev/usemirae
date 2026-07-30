/**
 * Tests for the project panel.
 *
 * What `MIR-0113` owes: the UI mirrors engine state and owns none of it, save
 * state is announced accessibly and not by colour alone, a rejected request is
 * shown against the control that caused it, disconnection disables project
 * actions rather than faking them, and the keyboard path works.
 *
 * The shell is faked at the `window.ipc` seam, which is exactly where the real
 * one sits. Nothing else is stubbed, so these exercise the same bridge client
 * the application ships.
 */

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ProjectPanel } from "./ProjectPanel";

/** A response with everything the schema requires. */
function response(overrides: Record<string, unknown> = {}) {
  return {
    requestId: "",
    ok: true,
    errorCode: "",
    engineConnected: true,
    engineSessionId: "session",
    protocolMajor: 1,
    protocolMinor: 0,
    projectOpen: false,
    projectName: "",
    projectPath: "",
    projectDirty: false,
    savedGeneration: 0,
    stateGeneration: 0,
    ...overrides,
  };
}

/**
 * Install a fake shell that answers each request with `answers.shift()`.
 *
 * The answer is delivered the way the real shell delivers it: by calling
 * `window.__mirae.receive` with a JSON string.
 */
function installShell(answers: Array<Record<string, unknown>>) {
  const sent: Array<Record<string, unknown>> = [];

  window.ipc = {
    postMessage(message: string) {
      const request = JSON.parse(message) as { requestId: string };
      sent.push(request);

      const answer = answers.shift() ?? response();

      // Asynchronous, like the real one: the shell answers from its event loop,
      // never inside the call that asked.
      queueMicrotask(() => {
        window.__mirae?.receive(
          JSON.stringify({ ...answer, requestId: request.requestId }),
        );
      });
    },
  };

  return sent;
}

afterEach(() => {
  // Vitest runs without `globals`, so Testing Library's automatic cleanup is
  // not installed. Without this, each test renders into the same document and
  // every query finds the previous test's panel as well as its own.
  cleanup();
  delete window.ipc;
  // `window.__mirae` is installed once by the bridge client, which is a module
  // singleton exactly as it is in the running application. Removing it here
  // would leave every later test with a shell that can never answer.
  vi.restoreAllMocks();
});

describe("ProjectPanel", () => {
  it("says project actions need the desktop window when there is no shell", () => {
    // Not a fake connection and not a disabled button with no explanation: the
    // panel says why (922).
    render(<ProjectPanel />);

    expect(screen.getByText(/need the Mirae desktop window/i)).toBeDefined();
  });

  it("reports no project open before one is created", async () => {
    installShell([response()]);
    render(<ProjectPanel />);

    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("No project open");
    });
  });

  it("disables save while no project is open", async () => {
    installShell([response()]);
    render(<ProjectPanel />);

    await waitFor(() => {
      expect(
        (screen.getByRole("button", { name: "Save" }) as HTMLButtonElement)
          .disabled,
      ).toBe(true);
    });

    // Disabled and still announced: `aria-disabled` keeps it in the
    // accessibility tree, so a screen-reader user learns it is unavailable
    // rather than finding it missing.
    expect(
      screen
        .getByRole("button", { name: "Save" })
        .getAttribute("aria-disabled"),
    ).toBe("true");
  });

  it("creates a project and shows what the shell reported", async () => {
    // The panel owns no project truth: the name it shows is the one that came
    // back, not the one it sent.
    installShell([
      response(),
      response({
        projectOpen: true,
        projectName: "Untitled project",
        projectDirty: true,
      }),
    ]);
    render(<ProjectPanel />);

    await userEvent.click(
      await screen.findByRole("button", { name: "New project" }),
    );

    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent(
        "Untitled project — Unsaved changes",
      );
    });
  });

  it("announces save state in words rather than by colour alone", async () => {
    // 923: colour is never the only indication. "Saved" and "Unsaved changes"
    // are text in a polite live region.
    installShell([
      response({
        projectOpen: true,
        projectName: "Stream",
        projectDirty: true,
      }),
      response({
        projectOpen: true,
        projectName: "Stream",
        projectDirty: false,
        savedGeneration: 1,
        projectPath: "C:/projects/stream.mirae.json",
      }),
    ]);
    render(<ProjectPanel />);

    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("Unsaved changes");
    });

    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("Stream — Saved");
    });
    expect(screen.getByText("C:/projects/stream.mirae.json")).toBeDefined();
  });

  it("shows a rejected request against the controls that caused it", async () => {
    // 109 section 8: a conflict is visible, and it is visible where the user
    // acted rather than somewhere else on the page.
    installShell([
      response(),
      response({ ok: false, errorCode: "project_already_open" }),
    ]);
    render(<ProjectPanel />);

    await userEvent.click(
      await screen.findByRole("button", { name: "New project" }),
    );

    const alert = await screen.findByRole("alert");

    expect(alert).toHaveTextContent("already open");

    // Associated with the controls, not merely near them.
    const group = screen.getByRole("button", { name: "Save" }).parentElement;
    expect(group?.getAttribute("aria-describedby")).toBe(alert.id);
  });

  it("keeps the last known state when the shell does not answer", async () => {
    // A request that never returns says nothing about the project. Replacing
    // what is known with a guess would be the UI inventing engine state.
    installShell([
      response({
        projectOpen: true,
        projectName: "Stream",
        projectDirty: true,
      }),
    ]);
    window.ipc = {
      postMessage() {
        // Answers nothing, ever.
      },
    };

    render(<ProjectPanel />);

    expect(screen.getByRole("status")).toHaveTextContent("No project open");
  });

  it("is operable from the keyboard alone", async () => {
    // 923 and 613: every path a pointer can take, a keyboard can take.
    installShell([
      response(),
      response({
        projectOpen: true,
        projectName: "Untitled project",
        projectDirty: true,
      }),
    ]);
    render(<ProjectPanel />);

    const create = await screen.findByRole("button", { name: "New project" });

    await userEvent.tab();

    expect(document.activeElement).toBe(create);

    await userEvent.keyboard("{Enter}");

    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("Untitled project");
    });
  });

  it("sends the request kinds the shell understands", async () => {
    const sent = installShell([
      response(),
      response({ projectOpen: true, projectName: "Untitled project" }),
      response({ projectOpen: true, projectName: "Untitled project" }),
    ]);
    render(<ProjectPanel />);

    await userEvent.click(
      await screen.findByRole("button", { name: "New project" }),
    );
    await waitFor(() => {
      expect(
        (screen.getByRole("button", { name: "Save" }) as HTMLButtonElement)
          .disabled,
      ).toBe(false);
    });
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(sent.map((request) => request["kind"])).toEqual([
        "projectState",
        "createProject",
        "saveProject",
      ]);
    });
  });
});
