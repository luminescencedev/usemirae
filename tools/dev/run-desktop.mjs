/**
 * Build and launch the Mirae desktop application in debug.
 *
 * Canonical documentation: `docs/05-platform/501-desktop-shell.md`,
 * `docs/08-development/806-build-system-and-toolchain.md`.
 *
 * The shell serves locally packaged resources (`501` invariant 2), so running it
 * means building the control UI first and pointing `MIRAE_UI_PATH` at the result.
 * That is three commands and an environment variable, which is three chances to
 * forget one and then debug a window showing a stale bundle. This script is the
 * one command.
 *
 * Plain Node, no dependency: `DEPENDENCY_VERSIONS.md` section 2 requires a ticket
 * for any package, and a process launcher does not need one.
 */

import { spawn } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const skipBuild = process.argv.includes("--no-build");

/** Run one command, resolving with its exit code. */
function run(command, args, options = {}) {
  return new Promise((resolveRun) => {
    const child = spawn(command, args, {
      cwd: root,
      stdio: "inherit",
      // Windows resolves `pnpm` and `cargo` through shell path lookup.
      shell: process.platform === "win32",
      ...options,
    });

    child.on("close", (code) => resolveRun(code ?? 1));
    child.on("error", () => resolveRun(1));
  });
}

/** Build, then launch. Stops at the first failure rather than launching a stale build. */
async function main() {
  if (!skipBuild) {
    const ui = await run("pnpm", ["--filter", "@mirae/control-ui", "build"]);

    if (ui !== 0) {
      console.error(
        "\nThe control UI did not build. The window would have shown a stale bundle.",
      );
      process.exit(ui);
    }

    const shell = await run("cargo", ["build", "--package", "mirae-shell"]);

    if (shell !== 0) {
      console.error("\nThe shell did not build.");
      process.exit(shell);
    }
  }

  const uiPath = join(root, "apps", "control-ui", "dist");

  console.log(`\nLaunching the desktop shell.\n  control UI: ${uiPath}\n`);

  const code = await run("cargo", ["run", "--package", "mirae-shell"], {
    env: {
      ...process.env,
      // The shell finds the engine beside its own executable, so only the UI
      // needs pointing at.
      MIRAE_UI_PATH: uiPath,
    },
  });

  process.exit(code);
}

await main();
