# Windows release verification

The signed release workflow is the executable acceptance check for the Windows-first package.

## Pinned application/Codex pair

- `CODEX_SIDECAR_WINDOWS_URL` points to the approved Codex executable for the release.
- `CODEX_SIDECAR_WINDOWS_SHA256` pins its exact bytes.
- The workflow rejects a checksum mismatch, generates the bundled version's App Server schema, and packages `codex.exe` through Tauri `externalBin`.
- Codex has no updater inside Bookshelf. A signed Tauri update replaces the application and bundled sidecar together.

## Clean-install checklist

1. Start from a supported Windows 11 VM without Node.js, Rust, Git, or Codex installed.
2. Install the signed NSIS artifact for the current user.
3. Launch Bookshelf and confirm the empty library opens. If WebView2 is unavailable, record the platform-provided WebView error and install the supported Evergreen runtime.
4. Import the PDF fixture, close the application, relaunch, and confirm the book and reading position remain.
5. Open Settings, start ChatGPT device login, and confirm the bundled Codex executable supplies the URL and code.
6. Install the previous signed candidate, import a book, then apply the new signed update. Confirm the book remains and the new bundled Codex version is present.
7. Attempt an unsigned/tampered update and confirm the updater rejects it while the installed version and library still open.
8. Uninstall Bookshelf and confirm the application entry is removed. Preserve or remove application data according to the installer prompt/policy under test.

The automated release job additionally installs silently, launches with a PATH containing only Windows system tools, and fails if the process exits during the smoke window.
