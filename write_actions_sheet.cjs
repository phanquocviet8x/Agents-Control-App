const { google } = require('googleapis');
const auth = new google.auth.GoogleAuth({
  keyFile: 'C:\\Users\\caxin\\Desktop\\account-google.json',
  scopes: ['https://www.googleapis.com/auth/spreadsheets']
});

const SHEET_ID = '1n5mSJOJ5jno5FZ27zTibjXTFwdY68F47EiO8aAeZw-A';

const actions = [
  ['Tab', 'Thẻ', 'Action Key', 'Mô Tả Lệnh', 'Type', 'PowerShell Command'],
  // === TAB RUN ===
  ['Run', 'OpenClaw', 'gateway-run', 'Khởi động OpenClaw Gateway', 'spawn', 'openclaw gateway run'],
  ['Run', 'OpenClaw', 'gateway-stop', 'Dừng OpenClaw Gateway', 'run_ps', 'openclaw gateway stop'],
  ['Run', 'OpenClaw', 'gateway-restart', 'Khởi động lại OpenClaw Gateway', 'spawn', 'openclaw gateway restart'],
  ['Run', 'OpenClaw', 'webui-run', 'Mở Dashboard trên trình duyệt', 'spawn', 'openclaw dashboard'],
  ['Run', 'N8N (Only)', 'n8n-run', 'Khởi động N8N và mở trình duyệt', 'spawn', 'n8n start --open'],
  ['Run', 'N8N (Only)', 'n8n-stop', 'Dừng tất cả process N8N (không kill OpenClaw)', 'run_ps', "Get-CimInstance Win32_Process -EA SilentlyContinue|Where-Object{$_.CommandLine -like '*n8n*' -and $_.CommandLine -notlike '*openclaw*'}|ForEach-Object{Stop-Process -Id $_.ProcessId -Force}"],
  ['Run', 'N8N (with Ngrok)', 'n8n-ngrok-run', 'Kill ngrok cũ → start N8N (ẩn browser) → chờ 8s → start ngrok ẩn → chờ 3s → mở browser domain', 'combo', "$env:N8N_OPEN_BROWSER='false';n8n start → wait 8s → Start-Process ngrok http <port> --url=<domain> -WindowStyle Hidden → wait 3s → Start-Process https://<domain>"],
  ['Run', 'N8N (with Ngrok)', 'n8n-ngrok-stop', 'Dừng cả N8N và Ngrok cùng lúc', 'run_ps', "Get-CimInstance Win32_Process -EA SilentlyContinue|Where-Object{$_.CommandLine -like '*n8n*' -and $_.CommandLine -notlike '*openclaw*'}|ForEach-Object{Stop-Process -Id $_.ProcessId -Force -EA SilentlyContinue}; taskkill /F /IM ngrok.exe 2>$null"],
  ['Run', 'Claude Code', 'claude-code-run', 'Khởi động Claude Code', 'spawn', 'claude'],
  ['Run', 'Claude Code', 'claude-code-stop', 'Dừng tất cả process Claude', 'run_ps', "Get-CimInstance Win32_Process -EA SilentlyContinue|Where-Object{$_.Name -like 'claude*' -or $_.CommandLine -like '*claude*'}|ForEach-Object{Stop-Process -Id $_.ProcessId -Force}"],
  ['Run', '9router', '9router-run', 'Khởi động 9router', 'spawn', '9router'],
  ['Run', '9router', '9router-stop', 'Dừng tất cả process 9router', 'run_ps', "Get-CimInstance Win32_Process -EA SilentlyContinue|Where-Object{$_.Name -like '9router*' -or $_.CommandLine -like '*9router*'}|ForEach-Object{Stop-Process -Id $_.ProcessId -Force}"],
  // === TAB SETUP ===
  ['Setup', 'Node.js', 'install-node', 'Cài đặt Node.js LTS qua winget', 'run_ps_visible', 'winget install OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements'],
  ['Setup', 'Node.js', 'update-node', 'Cập nhật Node.js lên bản mới nhất', 'run_ps_visible', 'winget upgrade OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements'],
  ['Setup', 'Node.js', 'uninstall-node', 'Gỡ cài đặt Node.js', 'run_ps_visible', 'winget uninstall OpenJS.NodeJS.LTS'],
  ['Setup', 'OpenClaw', 'install-openclaw', 'Cài đặt OpenClaw phiên bản 2026.4.23 qua npm', 'run_ps_visible', 'npm install -g openclaw@2026.4.23;openclaw --version'],
  ['Setup', 'OpenClaw', 'update-openclaw', 'Cập nhật OpenClaw lên bản mới nhất', 'run_ps_visible', 'npm update -g openclaw;openclaw --version'],
  ['Setup', 'OpenClaw', 'uninstall-openclaw', 'Gỡ cài đặt OpenClaw', 'run_ps_visible', 'npm uninstall -g openclaw'],
  ['Setup', 'Claude Code', 'install-claude', 'Cài đặt Claude Code qua npm', 'run_ps_visible', 'npm install -g @anthropic-ai/claude-code;claude --version'],
  ['Setup', 'Claude Code', 'update-claude', 'Cập nhật Claude Code lên bản mới nhất', 'run_ps_visible', 'npm update -g @anthropic-ai/claude-code;claude --version'],
  ['Setup', 'Claude Code', 'uninstall-claude', 'Gỡ cài đặt Claude Code', 'run_ps_visible', 'npm uninstall -g @anthropic-ai/claude-code'],
  ['Setup', '9router', 'install-9router', 'Cài đặt 9router qua npm', 'run_ps_visible', 'npm install -g 9router;9router --version'],
  ['Setup', '9router', 'update-9router', 'Cập nhật 9router lên bản mới nhất', 'run_ps_visible', 'npm update -g 9router;9router --version'],
  ['Setup', '9router', 'uninstall-9router', 'Gỡ cài đặt 9router', 'run_ps_visible', 'npm uninstall -g 9router'],
  ['Setup', 'N8N', 'install-n8n', 'Cài đặt N8N phiên bản mới nhất qua npm', 'run_ps_visible', 'npm install -g n8n@latest;n8n --version'],
  ['Setup', 'N8N', 'update-n8n', 'Cập nhật N8N lên bản mới nhất', 'run_ps_visible', 'npm update -g n8n;n8n --version'],
  ['Setup', 'N8N', 'uninstall-n8n', 'Gỡ cài đặt N8N', 'run_ps_visible', 'npm uninstall -g n8n'],
  ['Setup', 'Ngrok', 'install-ngrok', 'Cài đặt Ngrok qua winget', 'run_ps_visible', "if(Get-Command winget -EA SilentlyContinue){winget install ngrok.ngrok --accept-package-agreements --accept-source-agreements}else{Write-Host 'Install from ngrok.com'};ngrok version"],
  ['Setup', 'Ngrok', 'uninstall-ngrok', 'Gỡ cài đặt Ngrok', 'run_ps_visible', 'winget uninstall ngrok.ngrok'],
  // === TAB HELP ===
  ['Help', 'OpenClaw', 'openclaw-status', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'OpenClaw', 'openclaw-doctor', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'OpenClaw', 'openclaw-config', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'N8N', 'n8n-status', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'N8N', 'n8n-doctor', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'N8N', 'n8n-config', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'Ngrok', 'ngrok-status', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'Ngrok', 'ngrok-view-account', 'Xem thông tin tài khoản Ngrok đã lưu trong app', 'read_config', 'Đọc authtoken/domain/port từ app config'],
  ['Help', 'Claude Code', 'claude-code-status', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'Claude Code', 'claude-code-doctor', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  ['Help', 'Claude Code', 'claude-code-config', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND', 'UNKNOWN COMMAND'],
  // === TAB API KEYS > Ngrok ===
  ['API Keys', 'Ngrok', 'map-authtoken-<token>', 'Đăng ký authtoken vào file cấu hình ngrok', 'run_ps', 'ngrok config add-authtoken <token>'],
  ['API Keys', 'Ngrok', 'reset-ngrok-mapping', 'Xóa toàn bộ cấu hình ngrok+n8n+config app', 'run_ps', "taskkill /F /IM ngrok.exe; kill n8n (not openclaw); wait 2s; Remove-Item ~/.n8n; Remove-Item ngrok.yml; clear app ngrok config"],
];

async function main() {
  const client = await auth.getClient();
  const sheets = google.sheets({ version: 'v4', auth: client });
  
  await sheets.spreadsheets.values.clear({
    spreadsheetId: SHEET_ID,
    range: 'App_Actions!A:F'
  });
  
  await sheets.spreadsheets.values.update({
    spreadsheetId: SHEET_ID,
    range: 'App_Actions!A1',
    valueInputOption: 'RAW',
    requestBody: { values: actions }
  });
  
  console.log(`Done! ${actions.length - 1} actions written.`);
}

main().catch(console.error);
