# Agents Control Center - Feature Review

## Overview

Agents Control Center is a Windows desktop application built with React, Vite, Tauri, and Rust. It helps users install, configure, run, stop, back up, restore, and troubleshoot local AI-agent tools.

## Main Areas

### Run Tab

The Run tab controls runtime services and apps:

- Run OpenClaw Gateway.
- Stop OpenClaw Gateway.
- Restart OpenClaw Gateway.
- Open the OpenClaw Dashboard.
- Run and stop n8n.
- Run and stop n8n with ngrok.
- Run and stop Claude Code.
- Run and stop 9router.
- Show activity status for Node.js, OpenClaw Gateway, OpenClaw, n8n, ngrok, Claude Code, 9router, Git, and Python.
- Refresh app status on demand.

Status checks run once when the app opens and again only when the user presses Refresh.

### Terminal Tab

The Terminal tab provides an in-app PowerShell-style command runner:

- Run local commands.
- Show command output.
- Detect common errors from terminal output.
- Offer a local error explanation.
- Open diagnostic commands from failed app actions.

### Install Tab

The Install tab manages supported tools:

- Node.js
- OpenClaw
- Claude Code
- 9router
- n8n
- ngrok
- Git
- Python

Supported actions:

- Install.
- Add PATH.
- Update.
- Uninstall.
- Backup and restore app data.
- Open a bilingual Guide with installation instructions and error lookup.

### Setup Tab

The Setup tab configures OpenClaw:

- Gateway mode.
- Workspace path.
- Model providers.
- Web Search.
- Web Fetch.
- Gateway port, bind mode, auth mode, and token/password.
- Daemon runtime and service action.
- Channels such as Telegram.
- Plugins.
- Skills.
- Health Check.

Telegram-specific behavior:

- Saves `dmPolicy` as lowercase values such as `open`, `allowlist`, or `pairing`.
- Enables `plugins.entries.telegram.enabled`.
- Preserves Telegram group settings.
- Removes a broken `tokenFile` if a direct `botToken` is available.

### API Keys Tab

The API Keys tab stores and edits settings for:

- AI model provider keys.
- Custom providers.
- Telegram bot token and group chat ID.
- Google API credentials.
- ngrok authtoken, domain, and port.
- n8n URL and API key.

### Logs Tab

The Logs tab reads local log files for:

- Gateway.
- Web UI.
- n8n.
- ngrok.
- Claude Code.
- 9router.

### Thanks Tab

The Thanks tab shows project information and donation/support content.

### Backup And Restore

The app can back up and restore app data for:

- OpenClaw.
- Claude Code.
- n8n.
- ngrok.

Backups use a manifest and reject unknown or mismatched backup sources.

### Popup Behavior

Waiting popups appear only while work is running. Successful actions close quickly. Errors remain visible so the user can read details or open Check Error.

### Ownership Protection

The project requires `OWNERSHIP.md`. The Rust build process checks the required marker and fails when it is missing.
