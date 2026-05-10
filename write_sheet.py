import gspread
from google.oauth2.service_account import Credentials

SCOPES = ['https://www.googleapis.com/auth/spreadsheets','https://www.googleapis.com/auth/drive']
creds = Credentials.from_service_account_file(r'C:\Users\caxin\Desktop\account-google.json', scopes=SCOPES)
gc = gspread.authorize(creds)

SHEET_ID = '1n5mSJOJ5jno5FZ27zTibjXTFwdY68F47EiO8aAeZw-A'
sh = gc.open_by_key(SHEET_ID)
ws = sh.worksheet('App_Actions')

headers = ["Tab", u"Th\u1ebb/M\u1ee5c", u"T\u00ean N\u00fat", "Action ID", u"L\u1ec7nh Terminal/PowerShell", u"M\u00f4 T\u1ea3 L\u1ec7nh", u"Ghi Ch\u00fa"]

data = [
    headers,
    ["Run", "OpenClaw", "Run", "gateway-run", "openclaw gateway run --force", u"Kh\u1edfi ch\u1ea1y OpenClaw gateway", ""],
    ["Run", "OpenClaw", "Stop", "gateway-stop", "openclaw gateway stop", u"D\u1eebng OpenClaw gateway + kill process", ""],
    ["Run", "OpenClaw", "Restart", "gateway-restart", "openclaw gateway stop; openclaw gateway run --force", "Restart OpenClaw gateway", ""],
    ["Run", "OpenClaw", "Open Dashboard", "webui-run", "openclaw dashboard", u"M\u1edf giao di\u1ec7n web OpenClaw", ""],
    ["Run", "N8N & Ngrok", "N8N Run", "n8n-run", "n8n start", u"Kh\u1edfi ch\u1ea1y n8n server", ""],
    ["Run", "N8N & Ngrok", "N8N Stop", "n8n-stop", "Stop-Process n8n", u"D\u1eebng n8n", ""],
    ["Run", "N8N & Ngrok", "Ngrok Run", "ngrok-run", "ngrok http 5678", u"M\u1edf tunnel ngrok t\u1edbi port 5678", ""],
    ["Run", "N8N & Ngrok", "Ngrok Stop", "ngrok-stop", "Stop-Process ngrok", u"D\u1eebng ngrok", ""],
    ["Run", "Claude Code", "Run", "claude-code-run", "claude", u"Kh\u1edfi ch\u1ea1y Claude Code", ""],
    ["Run", "Claude Code", "Stop", "claude-code-stop", "Stop-Process claude", u"D\u1eebng Claude Code", ""],
    ["Run", "9router", "Run", "9router-run", "9router", u"Kh\u1edfi ch\u1ea1y 9router", ""],
    ["Run", "9router", "Stop", "9router-stop", "Stop-Process 9router", u"D\u1eebng 9router", ""],
    ["Run", "App Activity Status", "Refresh", "app_statuses", "(backend check)", u"Ki\u1ec3m tra tr\u1ea1ng th\u00e1i t\u1ea5t c\u1ea3 app", ""],
    ["", "", "", "", "", "", ""],
    ["Setup", "Node.js", "Install", "install-node-<version>", "winget install OpenJS.NodeJS.LTS --version <ver>", u"C\u00e0i Node.js theo version", ""],
    ["Setup", "OpenClaw", "Install", "install-openclaw-<version>", "npm install -g openclaw@<ver>", u"C\u00e0i OpenClaw theo version", ""],
    ["Setup", "Claude Code", "Install", "install-claude-code-<version>", "npm install -g @anthropic-ai/claude-code@<ver>", u"C\u00e0i Claude Code theo version", ""],
    ["Setup", "9router", "Install", "install-9router-<version>", "npm install -g 9router@<ver>", u"C\u00e0i 9router theo version", ""],
    ["Setup", "n8n", "Install", "install-n8n-<version>", "npm install -g n8n@<ver>", u"C\u00e0i n8n theo version", ""],
    ["Setup", "ngrok", "Install", "install-ngrok-<version>", "winget install ngrok.ngrok", u"C\u00e0i ngrok", ""],
    ["Setup", "Node.js", "Uninstall", "uninstall-node", "winget uninstall OpenJS.NodeJS.LTS", u"G\u1ee1 Node.js", ""],
    ["Setup", "OpenClaw", "Uninstall", "uninstall-openclaw", "npm uninstall -g openclaw", u"G\u1ee1 OpenClaw", ""],
    ["Setup", "Claude Code", "Uninstall", "uninstall-claude-code", "npm uninstall -g @anthropic-ai/claude-code", u"G\u1ee1 Claude Code", ""],
    ["Setup", "9router", "Uninstall", "uninstall-9router", "npm uninstall -g 9router", u"G\u1ee1 9router", ""],
    ["Setup", "n8n", "Uninstall", "uninstall-n8n", "npm uninstall -g n8n", u"G\u1ee1 n8n", ""],
    ["Setup", "ngrok", "Uninstall", "uninstall-ngrok", "winget uninstall ngrok.ngrok", u"G\u1ee1 ngrok", ""],
    ["", "", "", "", "", "", ""],
    ["Help", "OpenClaw", "Status", "openclaw-status", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "OpenClaw", "Doctor", "openclaw-doctor", "openclaw doctor --fix", u"Ch\u1ea1y doctor s\u1eeda l\u1ed7i OpenClaw", ""],
    ["Help", "OpenClaw", "Config", "openclaw-config", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "N8N", "Status", "n8n-status", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "N8N", "Doctor", "n8n-doctor", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "N8N", "Config", "n8n-config", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "Ngrok", "Status", "ngrok-status", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "Ngrok", "Doctor", "ngrok-doctor", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "Ngrok", "Config", "ngrok-config", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "Claude Code", "Status", "claude-code-status", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "Claude Code", "Doctor", "claude-code-doctor", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["Help", "Claude Code", "Config", "claude-code-config", "", u"CH\u01afA C\u00d3 L\u1ec6NH - c\u1ea7n b\u1ed5 sung", ""],
    ["", "", "", "", "", "", ""],
    ["Terminal", u"(to\u00e0n b\u1ed9)", u"Submit l\u1ec7nh", "run_terminal_cmd", u"(user nh\u1eadp l\u1ec7nh b\u1ea5t k\u1ef3)", u"Ch\u1ea1y PowerShell command", ""],
    ["", "", "", "", "", "", ""],
    ["API Keys", u"(to\u00e0n b\u1ed9)", "Save", "save_settings", u"(ghi file config)", u"L\u01b0u settings v\u00e0o config.json + sync openclaw.json", ""],
]

ws.clear()
ws.update(data, 'A1')
print('Done')
