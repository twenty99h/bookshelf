import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e/specs",
  outputDir: "./test-results/playwright",
  snapshotPathTemplate: "{testDir}/../snapshots/{arg}{ext}",
  fullyParallel: false,
  retries: process.env.CI ? 2 : 0,
  reporter: process.env.CI ? [["html", { open: "never" }], ["github"]] : "list",
  expect: {
    timeout: 8_000,
    toHaveScreenshot: { animations: "disabled", caret: "hide" },
  },
  use: {
    baseURL: "http://127.0.0.1:4173",
    locale: "ru-RU",
    timezoneId: "Europe/Volgograd",
    colorScheme: "dark",
    reducedMotion: "reduce",
    deviceScaleFactor: 1,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium-desktop",
      use: { ...devices["Desktop Chrome"], viewport: { width: 1920, height: 1080 } },
    },
  ],
  webServer: {
    command:
      "BOOKSHELF_BROWSER_ADAPTER=browser pnpm build && BOOKSHELF_BROWSER_ADAPTER=browser pnpm preview --host 127.0.0.1 --port 4173",
    port: 4173,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
