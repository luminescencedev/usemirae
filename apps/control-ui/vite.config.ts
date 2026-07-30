import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    // The desktop shell hosts this UI locally; never expose it on the network.
    host: "127.0.0.1",
    strictPort: true,
    port: 5273,
  },
  build: {
    target: "es2023",
    sourcemap: true,
    // The shell serves this bundle under a content security policy with
    // `script-src 'self'` (501 section 4), and the module-preload polyfill is
    // emitted as an inline script. Keeping it off means a future code-split
    // build cannot silently produce a document the webview refuses to run.
    modulePreload: { polyfill: false },
  },
  test: {
    environment: "jsdom",
    // Playwright owns `tests/e2e`; vitest only runs unit and component tests.
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    setupFiles: ["./tests/setup.ts"],
    // Remove once the first screen tests land with MIR-0011.
    passWithNoTests: true,
  },
});
