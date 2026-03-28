# Story: NPM Binary Wrapper

**ID:** story-01-npm-wrapper
**Epic:** Epic 1: Project Core & Infrastructure
**Status:** TODO

## Description
Create the NPM package with an OS and architecture detection shim. This allows users to run `npx skill manage` (or similar) to execute the native binary without manual installation.

## Acceptance Criteria
- [ ] `package.json` created in project root with proper metadata.
- [ ] JavaScript shim (`bin/skill-manage.js`) that detects:
  - OS: `win32`, `darwin`, `linux`.
  - Architecture: `x64`, `arm64`.
- [ ] The shim identifies and spawns the correct pre-built binary for the host platform.
- [ ] The `package.json` includes `optionalDependencies` for platform-specific binary packages (e.g., `@skill-manage/win32-x64`).
- [ ] Local testing of the shim works by manually placing a dummy binary.

## Implementation Details
- Refer to the architecture diagram for binary paths.
- Ensure proper use of `child_process.spawn` for performance and stream forwarding.
- Implement SHA-256 checksum verification (optional for first pass, mandatory for release).

## Verification Plan
- Run `node bin/skill-manage.js --version` and ensure it invokes the binary.
- Test across multiple available OS environments if possible.
