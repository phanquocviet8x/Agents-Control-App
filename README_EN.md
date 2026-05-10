# Agents Control Center - Documentation Index

This folder contains the English documentation for Agents Control Center.

## Where To Find Build Outputs

After running a production build, the generated files are in:

- Portable app: `src-tauri/target/release/agents-control-center.exe`
- Installer: `src-tauri/target/release/bundle/nsis/Agents Setup Center_2026.0.2_x64-setup.exe`

If the maintainer copied release files to the Desktop, look for:

- `Agents Control Center Portable.exe`
- `Agents Control Center Setup.exe`

## Where To Find Review And Guide Files

English documentation:

- `GUIDE_REVIEW_EN/README.md`
- `GUIDE_REVIEW_EN/BUILD_GUIDE.md`
- `GUIDE_REVIEW_EN/FEATURE_REVIEW.md`
- `GUIDE_REVIEW_EN/TROUBLESHOOTING.md`

Vietnamese documentation:

- `GUIDE_REVIEW_VN/README.md`
- `GUIDE_REVIEW_VN/HUONG_DAN_BUILD.md`
- `GUIDE_REVIEW_VN/REVIEW_TINH_NANG.md`
- `GUIDE_REVIEW_VN/HUONG_DAN_FIX_LOI.md`

## Required Source Files

A buildable source package should include:

- `src/`
- `src-tauri/`
- `public/`
- `ico/`
- `.github/` if GitHub Actions are used
- `package.json`
- `package-lock.json`
- `vite.config.js`
- `eslint.config.js`
- `index.html`
- `OWNERSHIP.md`

The project does not need `node_modules`, `dist`, or `src-tauri/target` inside a clean source backup. These folders are generated again during install/build.
