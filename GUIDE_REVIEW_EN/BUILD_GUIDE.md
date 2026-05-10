# Agents Control Center - Build Guide

## Required Environment

Build on Windows 10 or Windows 11 64-bit.

Required tools:

- Node.js LTS or newer, including npm.
- Rust stable toolchain, including Cargo.
- Microsoft Edge WebView2 Runtime.
- Visual Studio Build Tools with the C++ Desktop Development workload.
- Git, if the source is cloned from a repository.
- Internet access for downloading npm and Cargo dependencies.

Optional but useful:

- PowerShell 7 or Windows PowerShell.
- NSIS support through Tauri bundling, normally handled by `npx tauri build`.

## First-Time Setup

Open PowerShell in the source folder and run:

```powershell
npm install
```

This installs frontend and Tauri CLI dependencies from `package-lock.json`.

## Verify The Source

Run:

```powershell
npm run lint
npm run build
cd src-tauri
cargo test
cd ..
```

Expected result:

- ESLint passes.
- Vite production build succeeds.
- Rust tests pass.

## Build The Application

Run:

```powershell
npx tauri build
```

Expected outputs:

- Portable executable: `src-tauri/target/release/agents-control-center.exe`
- Installer executable: `src-tauri/target/release/bundle/nsis/Agents Setup Center_2026.0.2_x64-setup.exe`

## Copy Release Files To Desktop

Example:

```powershell
Copy-Item "src-tauri/target/release/agents-control-center.exe" "$env:USERPROFILE/Desktop/Agents Control Center Portable.exe" -Force
Copy-Item "src-tauri/target/release/bundle/nsis/Agents Setup Center_2026.0.2_x64-setup.exe" "$env:USERPROFILE/Desktop/Agents Control Center Setup.exe" -Force
```

## Important Ownership File

`OWNERSHIP.md` is required. The Rust build script checks this file and the marker:

```text
AGENTS_CONTROL_CENTER_OWNER_FILE_V1
```

If the file or marker is missing, the project must not build successfully.

## Clean Source Backup

Include source, assets, config, lockfiles, ownership, and documentation.

Exclude generated folders:

- `node_modules`
- `dist`
- `src-tauri/target`

Users can rebuild those folders with `npm install` and `npx tauri build`.
