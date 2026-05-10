# Agents Control Center - Hướng Dẫn Build

## Môi Trường Cần Thiết

Build trên Windows 10 hoặc Windows 11 bản 64-bit.

Cần cài:

- Node.js LTS hoặc mới hơn, bao gồm npm.
- Rust stable toolchain, bao gồm Cargo.
- Microsoft Edge WebView2 Runtime.
- Visual Studio Build Tools với workload C++ Desktop Development.
- Git nếu lấy source từ repository.
- Kết nối internet để tải dependency npm và Cargo.

Khuyến nghị thêm:

- PowerShell 7 hoặc Windows PowerShell.
- Tauri sẽ tự xử lý phần đóng gói NSIS khi chạy `npx tauri build`.

## Cài Dependency Lần Đầu

Mở PowerShell tại thư mục source và chạy:

```powershell
npm install
```

Lệnh này cài dependency frontend và Tauri CLI theo `package-lock.json`.

## Kiểm Tra Source

Chạy:

```powershell
npm run lint
npm run build
cd src-tauri
cargo test
cd ..
```

Kết quả mong đợi:

- ESLint không báo lỗi.
- Vite build frontend thành công.
- Test Rust chạy pass.

## Build Ứng Dụng

Chạy:

```powershell
npx tauri build
```

File build sinh ra:

- Bản portable: `src-tauri/target/release/agents-control-center.exe`
- Bản installer: `src-tauri/target/release/bundle/nsis/Agents Setup Center_2026.0.2_x64-setup.exe`

## Copy File Release Ra Desktop

Ví dụ:

```powershell
Copy-Item "src-tauri/target/release/agents-control-center.exe" "$env:USERPROFILE/Desktop/Agents Control Center Portable.exe" -Force
Copy-Item "src-tauri/target/release/bundle/nsis/Agents Setup Center_2026.0.2_x64-setup.exe" "$env:USERPROFILE/Desktop/Agents Control Center Setup.exe" -Force
```

## File Tác Quyền Bắt Buộc

`OWNERSHIP.md` là file bắt buộc. Rust build script kiểm tra file này và marker:

```text
AGENTS_CONTROL_CENTER_OWNER_FILE_V1
```

Nếu file hoặc marker bị mất, project không được build thành công.

## Backup Source Sạch

Nên giữ source, asset, config, lockfile, file tác quyền và tài liệu.

Không cần backup các thư mục sinh tự động:

- `node_modules`
- `dist`
- `src-tauri/target`

Người nhận source có thể sinh lại các thư mục này bằng `npm install` và `npx tauri build`.
