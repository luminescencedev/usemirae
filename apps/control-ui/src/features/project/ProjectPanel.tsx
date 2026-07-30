/**
 * Create, save, and see the state of the active project.
 *
 * Canonical documentation:
 * - `docs/01-runtime/109-ui-engine-synchronization.md`
 * - `docs/09-ui-ux/922-loading-empty-failure-and-recovery-states.md`
 * - `docs/09-ui-ux/923-accessibility-and-reduced-motion.md`
 *
 * The panel owns no project truth. Every render comes from the last answer the
 * shell gave, and every action asks the shell and then re-reads what it says —
 * `109` invariant 1 makes the engine authoritative, and a panel that predicted
 * the result of its own button would be a second source of truth that is right
 * most of the time.
 *
 * Two rules shape the markup. Save state is words as well as colour (`923`), so
 * "Unsaved changes" is written out rather than implied by a dot. And a rejected
 * request is shown against the control that caused it (`109` section 8), so the
 * error sits beside the buttons and is associated with them by
 * `aria-describedby` rather than announced from somewhere else on the page.
 */

import type { BridgeResponse } from "@mirae/contracts";
import { Button } from "@mirae/ui-kit";
import { useCallback, useEffect, useState } from "react";

import { bridge, isHosted } from "../../bridge";

/** What the panel knows, which is only what the shell last said. */
interface PanelState {
  readonly project: BridgeResponse | null;
  readonly error: string | null;
  readonly busy: boolean;
}

/** Turn a bridge error code into something a person can act on. */
function describeError(code: string): string {
  switch (code) {
    case "project_already_open":
      return "A project is already open. Close it before creating another.";
    case "no_project_open":
      return "There is no project to save yet.";
    case "invalid_argument":
      return "That project name cannot be used.";
    case "engine_unavailable":
      return "The engine is not available.";
    case "externally_modified":
      return "The file changed outside Mirae, so it was not overwritten.";
    case "filesystem_refused":
      return "The project file could not be written.";
    case "no_save_destination":
      return "There is nowhere to save this project yet.";
    default:
      return "The request was refused.";
  }
}

/** A panel for the active project. */
export function ProjectPanel() {
  const [state, setState] = useState<PanelState>({
    project: null,
    error: null,
    busy: false,
  });

  const hosted = isHosted();

  /** Send one request and adopt whatever the shell answers. */
  const send = useCallback(
    async (
      kind: "projectState" | "createProject" | "saveProject",
      name?: string,
    ) => {
      setState((previous) => ({ ...previous, busy: true }));

      try {
        const response = await bridge().request(kind, name);

        setState({
          project: response,
          error: response.ok ? null : describeError(response.errorCode),
          busy: false,
        });
      } catch {
        // A request that never came back says nothing about the project, so the
        // last known state is kept rather than replaced with a guess.
        setState((previous) => ({
          ...previous,
          error: "The shell did not answer.",
          busy: false,
        }));
      }
    },
    [],
  );

  // Read the project state once the panel is mounted. The request is made in
  // the effect and the state is set from its answer, never synchronously in the
  // effect body: an effect exists to synchronize with an external system, and
  // setting state on the way in is the cascading render React warns about.
  useEffect(() => {
    if (!hosted) {
      return;
    }

    let cancelled = false;

    void (async () => {
      try {
        const response = await bridge().request("projectState");

        if (!cancelled) {
          setState({
            project: response,
            error: response.ok ? null : describeError(response.errorCode),
            busy: false,
          });
        }
      } catch {
        if (!cancelled) {
          setState((previous) => ({
            ...previous,
            error: "The shell did not answer.",
            busy: false,
          }));
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [hosted]);

  if (!hosted) {
    return (
      <section aria-labelledby="project-heading" style={panelStyle}>
        <h2 id="project-heading" style={headingStyle}>
          Project
        </h2>
        <p style={{ margin: 0, color: "var(--mirae-fg-secondary)" }}>
          Project actions need the Mirae desktop window.
        </p>
      </section>
    );
  }

  const project = state.project;
  const open = project?.projectOpen ?? false;
  const dirty = project?.projectDirty ?? false;

  return (
    <section aria-labelledby="project-heading" style={panelStyle}>
      <h2 id="project-heading" style={headingStyle}>
        Project
      </h2>

      {/* Polite: a save finishing should not interrupt what is being read. */}
      <p role="status" aria-live="polite" style={{ margin: 0 }}>
        {open
          ? `${project?.projectName ?? ""} — ${dirty ? "Unsaved changes" : "Saved"}`
          : "No project open"}
      </p>

      {open && project?.projectPath ? (
        <p
          style={{
            margin: 0,
            color: "var(--mirae-fg-secondary)",
            font: "400 12px/1.4 inherit",
            wordBreak: "break-all",
          }}
        >
          {project.projectPath}
        </p>
      ) : null}

      <div
        style={{ display: "flex", gap: "8px" }}
        aria-describedby={state.error ? "project-error" : undefined}
      >
        <Button
          tone="primary"
          disabled={state.busy || open}
          onClick={() => void send("createProject", "Untitled project")}
        >
          New project
        </Button>
        <Button
          disabled={state.busy || !open}
          onClick={() => void send("saveProject")}
        >
          Save
        </Button>
      </div>

      {state.error ? (
        <p
          id="project-error"
          role="alert"
          style={{ margin: 0, color: "var(--mirae-critical, #f66)" }}
        >
          {state.error}
        </p>
      ) : null}
    </section>
  );
}

const panelStyle = {
  background: "var(--mirae-surface)",
  border: "1px solid var(--mirae-border)",
  borderRadius: "var(--mirae-radius-panel)",
  padding: "20px 24px",
  display: "grid",
  gap: "12px",
  minWidth: "36ch",
} as const;

const headingStyle = { font: "600 15px/1.4 inherit", margin: 0 } as const;
