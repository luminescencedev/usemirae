import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "jsdom",
    include: [
      "src/**/*.{test,spec}.{ts,tsx}",
      "tests/**/*.{test,spec}.{ts,tsx}",
    ],
    // The UI kit is a scaffold with no components yet. Remove this once the first
    // component tests land with the UI implementation backlog (docs/09-ui-ux/928).
    passWithNoTests: true,
  },
});
