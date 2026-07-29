/**
 * The generated handshake contracts are usable from TypeScript.
 *
 * Hand-written and outside the generated file, so regeneration never overwrites
 * them. They assert the same facts as the Rust suite in
 * `crates/foundation/contracts/tests/handshake_contracts.rs`, which is how the two
 * languages are kept in agreement.
 */

import { describe, expect, it } from "vitest";

import {
  CONTRACT_IDS,
  ENGINE_READINESS_DETAIL_MAX_LENGTH,
  ENGINE_READINESS_ENGINE_SESSION_ID_MAX_LENGTH,
  PROTOCOL_VERSION_MAJOR,
  PROTOCOL_VERSION_MINOR,
  type EngineReadiness,
  type EngineReadinessState,
  type ProtocolVersion,
} from "../src/index";

describe("protocol version", () => {
  it("exposes the constants from the schema", () => {
    expect(PROTOCOL_VERSION_MAJOR).toBe(1);
    expect(PROTOCOL_VERSION_MINOR).toBe(0);
  });

  it("can be built from its constants", () => {
    const version: ProtocolVersion = {
      major: PROTOCOL_VERSION_MAJOR,
      minor: PROTOCOL_VERSION_MINOR,
    };

    expect(version).toEqual({ major: 1, minor: 0 });
  });
});

describe("engine readiness", () => {
  it("accepts every readiness state", () => {
    const states: readonly EngineReadinessState[] = [
      "starting",
      "ready",
      "degraded",
      "stopping",
      "stopped",
    ];

    expect(states).toHaveLength(5);
    expect(new Set(states).size).toBe(states.length);
  });

  it("treats detail as optional", () => {
    const ready: EngineReadiness = {
      state: "ready",
      protocolMajor: PROTOCOL_VERSION_MAJOR,
      protocolMinor: PROTOCOL_VERSION_MINOR,
      engineSessionId: "session-1",
    };

    const degraded: EngineReadiness = {
      ...ready,
      state: "degraded",
      detail: "no GPU adapter",
    };

    expect(ready.detail).toBeUndefined();
    expect(degraded.detail).toBe("no GPU adapter");
  });

  it("exposes string bounds for bounded decoding", () => {
    // 108 section 9: reject oversized input before allocating for it.
    expect(ENGINE_READINESS_ENGINE_SESSION_ID_MAX_LENGTH).toBe(64);
    expect(ENGINE_READINESS_DETAIL_MAX_LENGTH).toBe(256);
  });
});

describe("contract ids", () => {
  it("are sorted and unique", () => {
    const ids = [...CONTRACT_IDS];

    expect(ids).toEqual([...ids].sort());
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toContain("mirae://ipc/v1/protocol-version");
    expect(ids).toContain("mirae://ipc/v1/engine-readiness");
  });
});
