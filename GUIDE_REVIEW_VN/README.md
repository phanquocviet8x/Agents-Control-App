# Agents Control Center - Mục Lục Tài Liệu

Thư mục này chứa tài liệu tiếng Việt cho Agents Control Center.

## Tìm File Build Ở Đâu

Sau khi build production, file sinh ra nằm ở:

- Bản portable: `src-tauri/target/release/agents-control-center.exe`
- Bản installer: `src-tauri/target/release/bundle/nsis/Agents Setup Center_2026.0.2_x64-setup.exe`

Nếu người build đã copy file ra Desktop, tìm các file:

- `Agents Control Center Portable.exe`
- `Agents Control Center Setup.exe`

## Tìm File Hướng Dẫn Và Review Ở Đâu

Tài liệu tiếng Anh:

- `GUIDE_REVIEW_EN/README.md`
- `GUIDE_REVIEW_EN/BUILD_GUIDE.md`
- `GUIDE_REVIEW_EN/FEATURE_REVIEW.md`
- `GUIDE_REVIEW_EN/TROUBLESHOOTING.md`

Tài liệu tiếng Việt:

- `GUIDE_REVIEW_VN/README.md`
- `GUIDE_REVIEW_VN/HUONG_DAN_BUILD.md`
- `GUIDE_REVIEW_VN/REVIEW_TINH_NANG.md`
- `GUIDE_REVIEW_VN/HUONG_DAN_FIX_LOI.md`

## Các File Source Cần Có

Một bộ source có thể build lại cần có:

- `src/`
- `src-tauri/`
- `public/`
- `ico/`
- `.github/` nếu còn dùng GitHub Actions
- `package.json`
- `package-lock.json`
- `vite.config.js`
- `eslint.config.js`
- `index.html`
- `OWNERSHIP.md`

Không cần giữ `node_modules`, `dist`, hoặc `src-tauri/target` trong bản backup source sạch. Các thư mục này sẽ được sinh lại khi chạy `npm install` và build.
