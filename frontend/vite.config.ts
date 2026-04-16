import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import UnoCSS from "unocss/vite";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react(), UnoCSS()],
  resolve: {
    alias: {
      "#src": resolve(__dirname, "src"),
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
        timeout: 30_000,
      },
      "/authorize": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/token": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/userinfo": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/jwks.json": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/.well-known": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/logout": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/health": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
    },
  },
});
