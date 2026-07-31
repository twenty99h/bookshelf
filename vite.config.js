import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { readFileSync } from "node:fs";

const host = process.env.TAURI_DEV_HOST;
const browserAdapter = process.env.BOOKSHELF_BROWSER_ADAPTER === "browser";

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    tailwindcss(),
    sveltekit(),
    browserAdapter && {
      name: "bookshelf-browser-pdf-fixture",
      generateBundle() {
        this.emitFile({
          type: "asset",
          fileName: "bookshelf-test.pdf",
          source: readFileSync(new URL("./e2e/fixtures/bookshelf-test.pdf", import.meta.url)),
        });
      },
    },
  ],
  resolve: {
    alias: {
      "@/app": new URL("./src/app", import.meta.url).pathname,
      "@/pages": new URL("./src/pages", import.meta.url).pathname,
      "@/shared": new URL("./src/shared", import.meta.url).pathname,
    },
  },
  define: {
    __BOOKSHELF_BROWSER_ADAPTER__: JSON.stringify(browserAdapter),
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
