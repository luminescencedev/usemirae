import { describe, expect, it } from "vitest";

import {
  checkLockfiles,
  checkPackageManagerAgent,
  checkPinConsistency,
  compareInstalled,
  extractVersion,
  normalizeVersion,
  parseNodeVersionFile,
  parsePackageManager,
  parseRustChannel,
  parseRustComponents,
  type Pin,
} from "../src/pins.ts";

const NODE_PIN: Pin = {
  tool: "node",
  version: "24.18.1",
  source: ".node-version",
};
const PNPM_PIN: Pin = {
  tool: "pnpm",
  version: "11.17.0",
  source: "package.json#packageManager",
};
const RUST_PIN: Pin = {
  tool: "rust",
  version: "1.97.1",
  source: "rust-toolchain.toml#channel",
};
const ALL_PINS = [NODE_PIN, PNPM_PIN, RUST_PIN];

describe("parseNodeVersionFile", () => {
  it("reads the version and strips a leading v", () => {
    expect(parseNodeVersionFile("v24.18.1\n")).toBe("24.18.1");
  });

  it("skips comments and blank lines", () => {
    expect(parseNodeVersionFile("\n# pinned by MIR-0002\n24.18.1\n")).toBe(
      "24.18.1",
    );
  });

  it("returns null for an empty file so the caller reports a missing pin", () => {
    expect(parseNodeVersionFile("   \n\n")).toBeNull();
  });
});

describe("parseRustChannel", () => {
  it("reads the channel", () => {
    const toml = '[toolchain]\nchannel = "1.97.1"\nprofile = "minimal"\n';
    expect(parseRustChannel(toml)).toBe("1.97.1");
  });

  it("returns null when no channel is declared", () => {
    expect(parseRustChannel('[toolchain]\nprofile = "minimal"\n')).toBeNull();
  });

  it("reads components", () => {
    const toml = '[toolchain]\ncomponents = ["clippy", "rustfmt", "rust-src"]\n';
    expect(parseRustComponents(toml)).toEqual(["clippy", "rustfmt", "rust-src"]);
  });
});

describe("parsePackageManager", () => {
  it("reads name and version", () => {
    expect(parsePackageManager("pnpm@11.17.0")).toEqual({
      name: "pnpm",
      version: "11.17.0",
    });
  });

  it("ignores a corepack integrity suffix", () => {
    expect(parsePackageManager("pnpm@11.17.0+sha224.abcdef")?.version).toBe(
      "11.17.0",
    );
  });

  it("reports a foreign package manager as its own name", () => {
    expect(parsePackageManager("yarn@4.0.0")?.name).toBe("yarn");
  });

  it("returns null when the field is absent", () => {
    expect(parsePackageManager(undefined)).toBeNull();
  });
});

describe("extractVersion", () => {
  it("reads a version out of rustc output", () => {
    expect(extractVersion("rustc 1.97.1 (8bab26f4f 2026-07-14)")).toBe("1.97.1");
  });

  it("reads a bare version", () => {
    expect(extractVersion("11.17.0\n")).toBe("11.17.0");
  });

  it("returns null when there is no version", () => {
    expect(extractVersion("command not found")).toBeNull();
  });
});

describe("normalizeVersion", () => {
  it("makes v-prefixed and bare versions comparable", () => {
    expect(normalizeVersion("v24.18.1")).toBe(normalizeVersion("24.18.1"));
  });
});

describe("checkPinConsistency", () => {
  it("accepts pins that agree", () => {
    expect(checkPinConsistency(ALL_PINS)).toEqual([]);
  });

  it("rejects pin files that disagree about the same tool", () => {
    const findings = checkPinConsistency([
      NODE_PIN,
      { tool: "node", version: "24.18.0", source: "package.json#engines.node" },
    ]);

    expect(findings).toHaveLength(1);
    expect(findings[0]?.detail).toContain("disagree");
    expect(findings[0]?.detail).toContain("24.18.0");
  });

  it.each(["^24.18.1", "~24.18.1", ">=24.18.1", "*", "latest", "24.x-beta"])(
    "rejects the forbidden pin %s",
    (version) => {
      const findings = checkPinConsistency([
        { tool: "node", version, source: ".node-version" },
      ]);

      expect(findings).toHaveLength(1);
      expect(findings[0]?.remediation).toContain("exact");
    },
  );

  it("rejects a non-exact version that is not a range", () => {
    const findings = checkPinConsistency([
      { tool: "node", version: "24.18", source: ".node-version" },
    ]);

    expect(findings[0]?.detail).toContain("not an exact");
  });
});

describe("compareInstalled", () => {
  const matching = {
    node: "24.18.1",
    pnpm: "11.17.0",
    rustc: "1.97.1",
    cargo: "1.97.1",
  };

  it("passes when everything matches", () => {
    expect(compareInstalled(ALL_PINS, matching)).toEqual([]);
  });

  it("reports a mismatch with both versions and a fix", () => {
    const findings = compareInstalled(ALL_PINS, {
      ...matching,
      node: "24.15.0",
    });

    expect(findings).toHaveLength(1);
    expect(findings[0]?.detail).toBe("found 24.15.0, expected 24.18.1");
    expect(findings[0]?.remediation).toContain("nvm install 24.18.1");
  });

  it("reports a missing tool as actionable instead of crashing", () => {
    const findings = compareInstalled(ALL_PINS, { ...matching, rustc: null });

    expect(findings[0]?.detail).toContain("not found on PATH");
    expect(findings[0]?.remediation).toContain("rustup toolchain install 1.97.1");
  });

  it("reports an undeclared pin", () => {
    const findings = compareInstalled([NODE_PIN, PNPM_PIN], matching);

    expect(findings).toHaveLength(2); // rustc and cargo share the missing rust pin
    expect(findings[0]?.detail).toContain("no rust pin");
  });

  it("checks cargo separately so a partial toolchain is caught", () => {
    const findings = compareInstalled(ALL_PINS, { ...matching, cargo: "1.96.0" });

    expect(findings).toHaveLength(1);
    expect(findings[0]?.tool).toBe("cargo");
  });
});

describe("checkPackageManagerAgent", () => {
  it("accepts pnpm", () => {
    expect(checkPackageManagerAgent("pnpm/11.17.0 npm/? node/v24.18.1")).toEqual(
      [],
    );
  });

  it("accepts an absent user agent", () => {
    expect(checkPackageManagerAgent(undefined)).toEqual([]);
  });

  it.each(["npm/10.9.0 node/v24.18.1", "yarn/4.0.0", "bun/1.1.0"])(
    "rejects %s",
    (agent) => {
      const findings = checkPackageManagerAgent(agent);

      expect(findings).toHaveLength(1);
      expect(findings[0]?.remediation).toContain("Use pnpm");
    },
  );
});

describe("checkLockfiles", () => {
  it("accepts both lockfiles present", () => {
    expect(checkLockfiles({ pnpm: true, cargo: true })).toEqual([]);
  });

  it("reports each missing lockfile", () => {
    const findings = checkLockfiles({ pnpm: false, cargo: false });

    expect(findings.map((finding) => finding.tool)).toEqual([
      "pnpm-lock.yaml",
      "Cargo.lock",
    ]);
  });
});
