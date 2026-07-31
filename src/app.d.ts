declare const __BOOKSHELF_BROWSER_ADAPTER__: boolean;

interface Window {
  __BOOKSHELF_TEST__?: {
    commands: unknown[];
    reset(fixture?: "active" | "empty"): void;
    scenario: "success" | "loading" | "error" | "codex-no-login" | "codex-crash" | "codex-cancel";
  };
}
