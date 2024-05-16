// vite.config.js
import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { TanStackRouterVite } from '@tanstack/router-vite-plugin'

export default defineConfig({
  plugins: [
    TanStackRouterVite({
      routesDirectory: './web/routes',
      generatedRouteTree: './web/routeTree.gen.ts',
      base: '/app'
    }),
    wasm(),
    topLevelAwait()
  ],
  base: "/app",
  worker: {
    // Not needed with vite-plugin-top-level-await >= 1.3.0
    format: "es",
    plugins: () => [wasm()],
  },
});
