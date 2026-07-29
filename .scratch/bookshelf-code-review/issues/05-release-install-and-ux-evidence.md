# 05 — Review release, clean install, and UX evidence

Type: task
Status: resolved

## Review findings

- Automated Windows smoke is not a clean VM test and omits uninstall.
- The generated App Server schema is not compared to adapter expectations.
- macOS/Linux do not package the paired sidecar or build a desktop candidate.
- UX documentation claims screenshots that the workflow does not create.

## Acceptance

- [x] Windows release automation verifies install, launch, and uninstall with explicit environment limits; the release checklist blocks publication on update preservation and tamper rejection.
- [x] Sidecar compatibility assertions run against the pinned binary.
- [x] Portability jobs build paired desktop candidates for macOS and Linux.
- [x] Desktop smoke captures and uploads honest visual evidence; documentation describes only implemented evidence.

## Answer

The signed release workflow now pairs checksum-pinned sidecars with Windows/macOS/Linux candidates, checks generated App Server schemas, and automates Windows install/launch/screenshot/uninstall. Signed-update preservation and tamper cases remain explicit blocking steps in the clean-install release checklist because they require configured release endpoints and signing secrets.
