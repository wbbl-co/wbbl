// vite.config.js
import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  base: "/app",
  worker: {
    // Not needed with vite-plugin-top-level-await >= 1.3.0
    format: "es",
    plugins: () => [wasm()],
  },
});
