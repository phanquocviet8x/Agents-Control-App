# Agents Control Center - Review Tính Năng

## Tổng Quan

Agents Control Center là ứng dụng desktop Windows được xây bằng React, Vite, Tauri và Rust. Ứng dụng hỗ trợ cài đặt, cấu hình, chạy, dừng, sao lưu, khôi phục và xử lý lỗi cho các công cụ AI agent chạy cục bộ.

## Các Khu Vực Chính

### Tab Run

Tab Run điều khiển các app và service:

- Chạy OpenClaw Gateway.
- Dừng OpenClaw Gateway.
- Restart OpenClaw Gateway.
- Mở OpenClaw Dashboard.
- Chạy và dừng n8n.
- Chạy và dừng n8n kèm ngrok.
- Chạy và dừng Claude Code.
- Chạy và dừng 9router.
- Hiển thị trạng thái hoạt động của Node.js, OpenClaw Gateway, OpenClaw, n8n, ngrok, Claude Code, 9router, Git và Python.
- Kiểm tra lại trạng thái khi người dùng bấm Refresh.

App chỉ tự kiểm tra trạng thái một lần khi mở app. Sau đó chỉ kiểm tra lại khi người dùng bấm Refresh.

### Tab Terminal

Tab Terminal cung cấp terminal kiểu PowerShell trong app:

- Chạy lệnh local.
- Hiển thị output lệnh.
- Phát hiện một số lỗi phổ biến từ output terminal.
- Hiển thị phân tích lỗi cục bộ.
- Mở lệnh chẩn đoán khi thao tác app bị lỗi.

### Tab Install

Tab Install quản lý các tool được hỗ trợ:

- Node.js
- OpenClaw
- Claude Code
- 9router
- n8n
- ngrok
- Git
- Python

Các thao tác hỗ trợ:

- Install.
- Add PATH.
- Update.
- Uninstall.
- Backup và Restore dữ liệu app.
- Mở Guide song ngữ gồm hướng dẫn cài đặt và bảng tra cứu lỗi.

### Tab Setup

Tab Setup cấu hình OpenClaw:

- Gateway mode.
- Đường dẫn workspace.
- Model provider.
- Web Search.
- Web Fetch.
- Gateway port, bind mode, auth mode và token/password.
- Daemon runtime và service action.
- Channel như Telegram.
- Plugin.
- Skills.
- Health Check.

Riêng Telegram:

- Lưu `dmPolicy` dạng lowercase như `open`, `allowlist`, `pairing`.
- Bật `plugins.entries.telegram.enabled`.
- Giữ cấu hình group Telegram.
- Xóa `tokenFile` lỗi nếu đã có `botToken` trực tiếp.

### Tab API Keys

Tab API Keys lưu và chỉnh:

- API key cho model provider.
- Custom provider.
- Telegram bot token và group chat ID.
- Google API credentials.
- ngrok authtoken, domain và port.
- n8n URL và API key.

### Tab Logs

Tab Logs đọc log local cho:

- Gateway.
- Web UI.
- n8n.
- ngrok.
- Claude Code.
- 9router.

### Tab Thanks

Tab Thanks hiển thị thông tin dự án và nội dung ủng hộ/donate.

### Backup Và Restore

App có thể backup và restore dữ liệu cho:

- OpenClaw.
- Claude Code.
- n8n.
- ngrok.

Backup có manifest kiểm tra nguồn dữ liệu. App từ chối backup sai app hoặc nguồn không hợp lệ.

### Popup Trạng Thái

Popup chờ chỉ hiện khi thao tác đang chạy. Khi thao tác thành công, popup tự tắt nhanh. Nếu lỗi, popup giữ lại để người dùng đọc lỗi hoặc bấm Check Error.

### Bảo Vệ Tác Quyền

Project bắt buộc có `OWNERSHIP.md`. Rust build process kiểm tra marker bắt buộc và sẽ lỗi nếu file này bị thiếu.
