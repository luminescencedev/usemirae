/**
 * Pure pin parsing and comparison.
 *
 * This module performs no I/O so that every rule is testable. `cli.ts` owns
 * reading files and probing installed tools.
 *
 * Canonical documentation:
 * - `DEPENDENCY_VERSIONS.md` (authoritative version lock)
 * - `docs/08-development/806-build-system-and-toolchain.md`
 * - `docs/08-development/808-local-development-environment.md`
 */

/** An exact semantic version, such as `24.18.1`. */
const EXACT_VERSION = /^\d+\.\d+\.\d+$/;

/** Range operators and release tags the version lock forbids. */
const FORBIDDEN_PIN = /^[\^~>=<]|\*|\b(latest|next|canary|beta|rc)\b/;

/** Where a pin came from, used verbatim in operator-facing output. */
export type PinSource =
  | ".node-version"
  | "package.json#engines.node"
  | "package.json#engines.pnpm"
  | "package.json#packageManager"
  | "rust-toolchain.toml#channel";

/** One toolchain requirement resolved from a canonical file. */
export interface Pin {
  readonly tool: "node" | "pnpm" | "rust";
  readonly version: string;
  readonly source: PinSource;
}

/** A problem an operator has to act on. */
export interface Finding {
  readonly tool: string;
  readonly detail: string;
  readonly remediation: string;
}

/**
 * Read the single version in a `.node-version` file.
 *
 * Returns `null` when the file has no usable content, so the caller can report a
 * missing pin instead of comparing against a blank string.
 */
export function parseNodeVersionFile(text: string): string | null {
  const line = text
    .split("\n")
    .map((candidate) => candidate.trim())
    .find((candidate) => candidate.length > 0 && !candidate.startsWith("#"));

  if (line === undefined) {
    return null;
  }

  return normalizeVersion(line);
}

/** Read `channel` out of a `rust-toolchain.toml` file. */
export function parseRustChannel(text: string): string | null {
  const match = /^\s*channel\s*=\s*["']([^"']+)["']/m.exec(text);
  return match?.[1] ?? null;
}

/** Read `components` out of a `rust-toolchain.toml` file. */
export function parseRustComponents(text: string): readonly string[] {
  const match = /^\s*components\s*=\s*\[([^\]]*)\]/m.exec(text);
  if (match?.[1] === undefined) {
    return [];
  }

  return match[1]
    .split(",")
    .map((entry) => entry.trim().replace(/^["']|["']$/g, ""))
    .filter((entry) => entry.length > 0);
}

/** Read the pnpm version out of a `packageManager` field such as `pnpm@11.17.0`. */
export function parsePackageManager(
  field: string | undefined,
): { readonly name: string; readonly version: string } | null {
  if (field === undefined) {
    return null;
  }

  // Corepack allows a `+sha224.<hash>` suffix; the hash is not a version.
  const match = /^([a-z]+)@([^+\s]+)/.exec(field.trim());
  if (match?.[1] === undefined || match[2] === undefined) {
    return null;
  }

  return { name: match[1], version: match[2] };
}

/** Strip a leading `v` so `v24.18.1` and `24.18.1` compare equal. */
export function normalizeVersion(raw: string): string {
  return raw.trim().replace(/^v/, "");
}

/**
 * Extract a version from `--version` output.
 *
 * Handles `rustc 1.97.1 (8bab26f4f 2026-07-14)` and bare `11.17.0` alike.
 */
export function extractVersion(output: string): string | null {
  const match = /(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)/.exec(output);
  return match?.[1] ?? null;
}

/**
 * Verify the canonical files agree with each other.
 *
 * A lock that contradicts itself is worse than a mismatch against the machine,
 * because every later check inherits the ambiguity.
 */
export function checkPinConsistency(pins: readonly Pin[]): readonly Finding[] {
  const findings: Finding[] = [];

  for (const tool of ["node", "pnpm", "rust"] as const) {
    const forTool = pins.filter((pin) => pin.tool === tool);
    const distinct = [...new Set(forTool.map((pin) => pin.version))];

    if (distinct.length > 1) {
      const detail = forTool
        .map((pin) => `${pin.source} = ${pin.version}`)
        .join(", ");
      findings.push({
        tool,
        detail: `pin files disagree (${detail})`,
        remediation:
          `Make every ${tool} pin identical and record the change in ` +
          `DEPENDENCY_VERSIONS.md in the same commit.`,
      });
    }
  }

  for (const pin of pins) {
    if (FORBIDDEN_PIN.test(pin.version)) {
      findings.push({
        tool: pin.tool,
        detail: `${pin.source} uses a range or release tag (${pin.version})`,
        remediation:
          "DEPENDENCY_VERSIONS.md section 2 forbids ^, ~, *, >=, latest, next, " +
          "canary, beta, and rc. Use one exact version.",
      });
      continue;
    }

    if (!EXACT_VERSION.test(pin.version)) {
      findings.push({
        tool: pin.tool,
        detail: `${pin.source} is not an exact x.y.z version (${pin.version})`,
        remediation: `Pin ${pin.tool} to an exact version such as 1.2.3.`,
      });
    }
  }

  return findings;
}

/** What is actually installed on this machine. `null` means "not found". */
export interface InstalledVersions {
  readonly node: string | null;
  readonly pnpm: string | null;
  readonly rustc: string | null;
  readonly cargo: string | null;
}

/** How to install each tool, kept next to the comparison that needs it. */
const REMEDIATION: Record<string, (expected: string) => string> = {
  node: (expected) =>
    `Install Node ${expected} and activate it (nvm install ${expected} && ` +
    `nvm use ${expected}). The pin lives in .node-version.`,
  pnpm: (expected) =>
    `Activate pnpm ${expected} (corepack enable && corepack use pnpm@${expected}). ` +
    "npm, Yarn, Bun, and Deno are not project package managers.",
  rustc: (expected) =>
    `Install the pinned Rust toolchain (rustup toolchain install ${expected}). ` +
    "rustup reads rust-toolchain.toml automatically inside the repository.",
  cargo: (expected) =>
    `Cargo ships with Rust ${expected}; installing that toolchain fixes both ` +
    "(rustup toolchain install " +
    expected +
    ").",
};

/** Compare the resolved pins against what is installed. */
export function compareInstalled(
  pins: readonly Pin[],
  installed: InstalledVersions,
): readonly Finding[] {
  const findings: Finding[] = [];
  const expectedFor = (tool: Pin["tool"]): string | null =>
    pins.find((pin) => pin.tool === tool)?.version ?? null;

  const checks: readonly {
    readonly key: keyof InstalledVersions;
    readonly tool: Pin["tool"];
  }[] = [
    { key: "node", tool: "node" },
    { key: "pnpm", tool: "pnpm" },
    { key: "rustc", tool: "rust" },
    { key: "cargo", tool: "rust" },
  ];

  for (const { key, tool } of checks) {
    const expected = expectedFor(tool);
    if (expected === null) {
      findings.push({
        tool: key,
        detail: `no ${tool} pin found in the canonical files`,
        remediation: `Declare the ${tool} version, then re-run the check.`,
      });
      continue;
    }

    const actual = installed[key];
    if (actual === null) {
      findings.push({
        tool: key,
        detail: `not found on PATH (expected ${expected})`,
        remediation: REMEDIATION[key]?.(expected) ?? `Install ${key} ${expected}.`,
      });
      continue;
    }

    if (actual !== expected) {
      findings.push({
        tool: key,
        detail: `found ${actual}, expected ${expected}`,
        remediation: REMEDIATION[key]?.(expected) ?? `Install ${key} ${expected}.`,
      });
    }
  }

  return findings;
}

/**
 * Reject package managers the repository does not use.
 *
 * npm and Yarn silently produce a second lockfile, which breaks the reproducible
 * install the version lock depends on.
 */
export function checkPackageManagerAgent(
  userAgent: string | undefined,
): readonly Finding[] {
  if (userAgent === undefined || userAgent.length === 0) {
    return [];
  }

  const name = /^([a-z]+)\//.exec(userAgent)?.[1];
  if (name === undefined || name === "pnpm") {
    return [];
  }

  return [
    {
      tool: "package manager",
      detail: `invoked through ${name}`,
      remediation:
        "Use pnpm. DEPENDENCY_VERSIONS.md section 2 forbids npm, Yarn, Bun, and " +
        "Deno as project package managers; another manager would write a second " +
        "lockfile and break reproducible installs.",
    },
  ];
}

/** Report a missing committed lockfile, which breaks frozen installs. */
export function checkLockfiles(
  present: { readonly pnpm: boolean; readonly cargo: boolean },
): readonly Finding[] {
  const findings: Finding[] = [];

  if (!present.pnpm) {
    findings.push({
      tool: "pnpm-lock.yaml",
      detail: "missing",
      remediation:
        "Run pnpm install and commit pnpm-lock.yaml; " +
        "pnpm install --frozen-lockfile cannot work without it.",
    });
  }

  if (!present.cargo) {
    findings.push({
      tool: "Cargo.lock",
      detail: "missing",
      remediation:
        "Run cargo check --workspace and commit Cargo.lock; deployable " +
        "applications require a committed lockfile.",
    });
  }

  return findings;
}
