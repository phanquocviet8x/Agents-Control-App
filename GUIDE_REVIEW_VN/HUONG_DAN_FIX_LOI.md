# Agents Control Center - Hướng Dẫn Fix Lỗi Cơ Bản

## Không Nhận Lệnh

Dấu hiệu:

- `node is not recognized`
- `npm is not recognized`
- `openclaw is not recognized`
- `n8n is not recognized`

Cách fix:

1. Kiểm tra tool đã được cài chưa.
2. Bấm `Add PATH` cho tool đó trong tab Install.
3. Đóng và mở lại app.
4. Nếu Windows vẫn chưa nhận lệnh, restart Windows.

## npm Install Failed

Dấu hiệu:

- npm báo lỗi quyền.
- Tải package thất bại.
- Lệnh install kết thúc bằng error.

Cách fix:

1. Kiểm tra mạng internet.
2. Kiểm tra phiên bản Node.js và npm.
3. Tránh dùng lẫn Node từ Microsoft Store với Node.js bản chính thức.
4. Cài lại Node.js LTS nếu npm global path bị lỗi.
5. Mở app với quyền phù hợp nếu chính sách máy yêu cầu.

## Port Đang Bị Chiếm

Dấu hiệu:

- `EADDRINUSE`
- address already in use
- n8n hoặc gateway không start được.

Cách fix:

1. Dừng app trong tab Run.
2. Kiểm tra process nào đang dùng port.
3. Với n8n, kiểm tra port `5678`.
4. Với OpenClaw Gateway, kiểm tra port `18789`.
5. Đổi port cấu hình nếu cần.

## OpenClaw Gateway Không Chạy

Cách fix:

1. Kiểm tra OpenClaw đã cài chưa.
2. Vào Setup kiểm tra cấu hình gateway.
3. Chạy `Fix Gateway`.
4. Chạy `Doctor` nếu cần.
5. Restart Gateway.
6. Xem Logs nếu lỗi vẫn còn.

## Dashboard Không Mở

Cách fix:

1. Start hoặc restart OpenClaw Gateway.
2. Bấm `Open Dashboard`.
3. Nếu gateway bật token auth, kiểm tra token có trong `openclaw.json`.
4. Bấm Refresh nếu trạng thái hiển thị bị cũ.

## Telegram Bot Không Phản Hồi

Kiểm tra:

- `channels.telegram.enabled` là `true`.
- `channels.telegram.botToken` đã có token.
- `channels.telegram.defaultTo` đúng chat/group.
- `channels.telegram.dmPolicy` là chữ thường: `open`, `allowlist`, hoặc `pairing`.
- `channels.telegram.groups["*"].requireMention` đúng nhu cầu group.
- `plugins.entries.telegram.enabled` là `true`.

Nếu `tokenFile` trỏ tới file không tồn tại nhưng đã có `botToken`, hãy xóa `tokenFile` hoặc Save lại channel Telegram trong app.

## Lỗi ngrok Authtoken Hoặc Domain

Cách fix:

1. Nhập authtoken ngrok hợp lệ.
2. Nhập domain đã reserve, không có `https://`.
3. Xóa dấu `/` ở cuối domain.
4. Kiểm tra port đúng.
5. Kiểm tra tài khoản có quyền dùng domain đó.

## API 401 Hoặc 403

Cách fix:

1. Nhập lại API key.
2. Kiểm tra provider đã chọn đúng chưa.
3. Kiểm tra key có quyền dùng model không.
4. Save settings lại.
5. Thử chạy lại thao tác.

## Backup Hoặc Restore Lỗi

Cách fix:

1. Dùng đúng file backup zip do app tạo.
2. Không sửa manifest backup thủ công.
3. Chọn đúng app với file backup.
4. Dừng service đang chạy trước khi restore nếu file bị khóa.
