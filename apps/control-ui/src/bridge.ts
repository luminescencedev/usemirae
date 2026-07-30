/**
 * The page half of the shell bridge.
 *
 * Canonical documentation: `docs/05-platform/501-desktop-shell.md` sections 3
 * and 13, ADR-0068.
 *
 * The shell posts answers by evaluating a script that calls
 * `window.__mirae.receive`. This module installs that function, correlates each
 * answer with the request that asked for it, and times out the ones that never
 * arrive — a request with no answer and no timeout is a promise that never
 * settles, and a UI waiting on one looks identical to a UI that has hung.
 *
 * Nothing here decides anything. The shell validates every request against a
 * closed set, so this side sends what it is asked to and reads what comes back.
 */

import type { BridgeRequest, BridgeResponse } from "@mirae/contracts";

/** How long to wait for an answer before giving up on it. */
const REQUEST_TIMEOUT_MS = 5_000;

/** What the shell exposes to the page. */
interface ShellIpc {
  /** Post a message to the shell. */
  postMessage(message: string): void;
}

declare global {
  interface Window {
    /** Present only when running inside the Mirae shell. */
    ipc?: ShellIpc;
    /** Installed by this module for the shell to call back into. */
    __mirae?: {
      receive(payload: string): void;
    };
  }
}

/** Whether this page is running inside the shell rather than a browser tab. */
export function isHosted(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof window.ipc?.postMessage === "function"
  );
}

/** Correlates requests with the answers the shell sends back. */
class BridgeClient {
  readonly #pending = new Map<
    string,
    {
      resolve: (response: BridgeResponse) => void;
      reject: (error: Error) => void;
    }
  >();

  #nextId = 0;

  constructor() {
    if (typeof window === "undefined") {
      return;
    }

    window.__mirae = {
      receive: (payload: string) => {
        this.#receive(payload);
      },
    };
  }

  /** Send a request and resolve with its answer. */
  request(kind: BridgeRequest["kind"]): Promise<BridgeResponse> {
    const ipc = window.ipc;

    if (!ipc) {
      return Promise.reject(
        new Error("the control UI is not hosted by the Mirae shell"),
      );
    }

    this.#nextId += 1;
    const requestId = `r-${this.#nextId}`;

    return new Promise<BridgeResponse>((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.#pending.delete(requestId);
        reject(new Error("the shell did not answer"));
      }, REQUEST_TIMEOUT_MS);

      this.#pending.set(requestId, {
        resolve: (response) => {
          clearTimeout(timeout);
          resolve(response);
        },
        reject: (error) => {
          clearTimeout(timeout);
          reject(error);
        },
      });

      ipc.postMessage(
        JSON.stringify({ requestId, kind } satisfies BridgeRequest),
      );
    });
  }

  /** Handle one answer from the shell. */
  #receive(payload: string): void {
    let response: BridgeResponse;

    try {
      response = JSON.parse(payload) as BridgeResponse;
    } catch {
      // A payload that will not parse names no request, so there is nothing to
      // settle. The waiting requests time out, which is the honest outcome.
      return;
    }

    const pending = this.#pending.get(response.requestId);

    if (!pending) {
      // An answer to a request that already timed out, or one the page never
      // sent. Dropped rather than acted on.
      return;
    }

    this.#pending.delete(response.requestId);
    pending.resolve(response);
  }
}

let client: BridgeClient | undefined;

/** The bridge client for this page. */
export function bridge(): BridgeClient {
  client ??= new BridgeClient();

  return client;
}
