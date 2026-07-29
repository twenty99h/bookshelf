import { readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

describe("Tauri asset protocol", () => {
  it("allows imported books from the application data directory", () => {
    const config = JSON.parse(readFileSync(join(process.cwd(), "src-tauri/tauri.conf.json"), "utf8"));

    expect(config.app.security.assetProtocol).toMatchObject({
      enable: true,
      scope: ["$APPDATA/books/**/*"],
    });
  });

  it("allows PDF.js to compile the bundled image decoders", () => {
    const config = JSON.parse(readFileSync(join(process.cwd(), "src-tauri/tauri.conf.json"), "utf8"));

    expect(config.app.security.csp).toContain("script-src 'self' 'wasm-unsafe-eval'");
    expect(statSync(join(process.cwd(), "static/pdfjs/wasm/openjpeg.wasm")).size).toBeGreaterThan(0);
    expect(statSync(join(process.cwd(), "static/pdfjs/wasm/openjpeg_nowasm_fallback.js")).size).toBeGreaterThan(0);
  });
});
