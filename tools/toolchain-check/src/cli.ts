/**
 * Toolchain verification entry point.
 *
 * Runs as the root `preinstall` hook and as `pnpm run check:toolchain`, so a
 * wrong Rust, Node, or pnpm version fails before any dependency is fetched
 * (`docs/08-development/808-local-development-environment.md` invariant 5).
 *
 * Uses only Node built-ins: `preinstall` executes before `node_modules` exists.
 *
 * This is the temporary home of the check. `MIR-0003` moves it behind
 * `cargo xtask bootstrap`, which owns toolchain verification per
 * `docs/08-development/806-build-system-and-toolchain.md` section 4. Removal
 * condition: `cargo xtask bootstrap` performs these checks and this package is
 * deleted.
 */

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

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
  type Finding,
  type InstalledVersions,
  type Pin,
} from "./pins.ts";

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

/** Read a repository file, or `null` when it does not exist. */
function readRepoFile(relativePath: string): string | null {
  const absolute = join(REPO_ROOT, relativePath);
  if (!existsSync(absolute)) {
    return null;
  }

  try {
    return readFileSync(absolute, "utf8");
  } catch {
    // An unreadable pin file is reported as a missing pin rather than a crash.
    return null;
  }
}

/**
 * Probe an installed tool's version. Returns `null` when it cannot be run.
 *
 * A shell is required because pnpm is a `.cmd` shim on Windows. The command is
 * passed as one string rather than as `(command, args)`: Node deprecates the
 * latter with `shell: true` (DEP0190). Every caller passes a literal, so no
 * external input reaches the shell.
 */
function probe(command: string): string | null {
  const result = spawnSync(command, {
    encoding: "utf8",
    shell: true,
    // A hung probe must not hang an install.
    timeout: 20_000,
  });

  if (result.error !== undefined || result.status !== 0) {
    return null;
  }

  return extractVersion(`${result.stdout ?? ""}${result.stderr ?? ""}`);
}

/** Resolve every declared pin from the canonical files. */
function resolvePins(): { pins: readonly Pin[]; findings: readonly Finding[] } {
  const pins: Pin[] = [];
  const findings: Finding[] = [];

  const nodeVersionFile = readRepoFile(".node-version");
  const nodePin =
    nodeVersionFile === null ? null : parseNodeVersionFile(nodeVersionFile);
  if (nodePin === null) {
    findings.push({
      tool: "node",
      detail: ".node-version is missing or empty",
      remediation:
        "Create .node-version containing the exact Node version from " +
        "DEPENDENCY_VERSIONS.md section 3.",
    });
  } else {
    pins.push({ tool: "node", version: nodePin, source: ".node-version" });
  }

  const rootPackage = readRepoFile("package.json");
  if (rootPackage === null) {
    findings.push({
      tool: "package.json",
      detail: "missing at the repository root",
      remediation: "Restore the root package.json.",
    });
  } else {
    let parsed: {
      engines?: { node?: string; pnpm?: string };
      packageManager?: string;
    } = {};

    try {
      parsed = JSON.parse(rootPackage) as typeof parsed;
    } catch {
      findings.push({
        tool: "package.json",
        detail: "is not valid JSON",
        remediation: "Fix the JSON syntax in the root package.json.",
      });
    }

    if (parsed.engines?.node !== undefined) {
      pins.push({
        tool: "node",
        version: normalizeVersion(parsed.engines.node),
        source: "package.json#engines.node",
      });
    }

    if (parsed.engines?.pnpm !== undefined) {
      pins.push({
        tool: "pnpm",
        version: normalizeVersion(parsed.engines.pnpm),
        source: "package.json#engines.pnpm",
      });
    }

    const packageManager = parsePackageManager(parsed.packageManager);
    if (packageManager === null) {
      findings.push({
        tool: "pnpm",
        detail: "package.json#packageManager is missing or unparsable",
        remediation:
          'Set "packageManager": "pnpm@<exact version>" so corepack activates ' +
          "the pinned pnpm.",
      });
    } else if (packageManager.name !== "pnpm") {
      findings.push({
        tool: "package manager",
        detail: `package.json#packageManager declares ${packageManager.name}`,
        remediation:
          "Mirae uses pnpm. npm, Yarn, Bun, and Deno are not project package " +
          "managers.",
      });
    } else {
      pins.push({
        tool: "pnpm",
        version: packageManager.version,
        source: "package.json#packageManager",
      });
    }
  }

  const rustToolchain = readRepoFile("rust-toolchain.toml");
  const rustPin = rustToolchain === null ? null : parseRustChannel(rustToolchain);
  if (rustPin === null) {
    findings.push({
      tool: "rust",
      detail: "rust-toolchain.toml is missing or declares no channel",
      remediation:
        "Create rust-toolchain.toml with the exact channel from " +
        "DEPENDENCY_VERSIONS.md section 11.",
    });
  } else {
    pins.push({
      tool: "rust",
      version: rustPin,
      source: "rust-toolchain.toml#channel",
    });
  }

  return { pins, findings };
}

function main(): number {
  const { pins, findings: pinFindings } = resolvePins();

  const installed: InstalledVersions = {
    node: normalizeVersion(process.versions.node),
    pnpm: probe("pnpm --version"),
    rustc: probe("rustc --version"),
    cargo: probe("cargo --version"),
  };

  const findings: readonly Finding[] = [
    ...checkPackageManagerAgent(process.env["npm_config_user_agent"]),
    ...pinFindings,
    ...checkPinConsistency(pins),
    ...compareInstalled(pins, installed),
    ...checkLockfiles({
      pnpm: existsSync(join(REPO_ROOT, "pnpm-lock.yaml")),
      cargo: existsSync(join(REPO_ROOT, "Cargo.lock")),
    }),
  ];

  const expectedFor = (tool: Pin["tool"]): string =>
    pins.find((pin) => pin.tool === tool)?.version ?? "unpinned";

  const rows: readonly (readonly [string, string, string])[] = [
    ["node", expectedFor("node"), installed.node ?? "not found"],
    ["pnpm", expectedFor("pnpm"), installed.pnpm ?? "not found"],
    ["rustc", expectedFor("rust"), installed.rustc ?? "not found"],
    ["cargo", expectedFor("rust"), installed.cargo ?? "not found"],
  ];

  const lines = [
    "Mirae toolchain check",
    ...rows.map(([tool, expected, actual]) => {
      const status = expected === actual ? "ok  " : "FAIL";
      return `  ${status} ${tool.padEnd(6)} expected ${expected.padEnd(10)} found ${actual}`;
    }),
  ];

  if (findings.length === 0) {
    lines.push("", "Toolchain matches DEPENDENCY_VERSIONS.md.");
    process.stdout.write(`${lines.join("\n")}\n`);
    return 0;
  }

  lines.push("", `${findings.length} problem(s) to fix:`);
  for (const [index, finding] of findings.entries()) {
    lines.push(
      "",
      `${index + 1}. ${finding.tool}: ${finding.detail}`,
      `   -> ${finding.remediation}`,
    );
  }
  lines.push(
    "",
    "Authoritative lock: DEPENDENCY_VERSIONS.md",
    "To install dependencies without this gate: pnpm install --ignore-scripts",
  );

  process.stderr.write(`${lines.join("\n")}\n`);
  return 1;
}

try {
  process.exitCode = main();
} catch (error) {
  // The gate must state why it could not decide instead of failing silently.
  process.stderr.write(
    `Mirae toolchain check could not complete: ${String(error)}\n` +
      "Report this as a repository tooling defect. To install without this " +
      "gate: pnpm install --ignore-scripts\n",
  );
  process.exitCode = 1;
}
