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
  },
  test: {
    environment: "jsdom",
    // Playwright owns `tests/e2e`; vitest only runs unit and component tests.
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    // Remove once the first screen tests land with MIR-0011.
    passWithNoTests: true,
  },
});
