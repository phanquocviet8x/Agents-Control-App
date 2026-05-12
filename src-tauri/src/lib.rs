// Copyright (c) 2026 Vu. All rights reserved.
// Proprietary source. See OWNERSHIP.md at the repository root.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const OWNERSHIP_NOTICE: &str = include_str!("../../OWNERSHIP.md");
const OWNERSHIP_MARKER: &str = "AGENTS_CONTROL_CENTER_OWNER_FILE_V1";
const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const ANTHROPIC_DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const GOOGLE_DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const GROQ_DEFAULT_BASE_URL: &str = "https://api.groq.com/openai/v1";
const OPENROUTER_DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";
const XAI_DEFAULT_BASE_URL: &str = "https://api.x.ai/v1";
const NINE_ROUTER_DEFAULT_BASE_URL: &str = "http://localhost:20128/v1";
const DEFAULT_CONTEXT_WINDOW: u64 = 500000;
const DEFAULT_OPENAI_COMPAT_MAX_TOKENS: u64 = 8192;
const DEFAULT_ANTHROPIC_MAX_TOKENS: u64 = 4096;
const DEFAULT_CODEX_MAX_TOKENS: u64 = 128000;
const BACKUP_FORMAT: &str = "agents-control-center-backup-v2";

fn verify_ownership_notice() {
    if !OWNERSHIP_NOTICE.contains(OWNERSHIP_MARKER) {
        panic!("Missing required ownership marker. The application cannot run.");
    }
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\Users\\caxin"))
}
fn config_dir() -> PathBuf {
    home_dir().join(".agents-control-center")
}
fn config_file() -> PathBuf {
    config_dir().join("config.json")
}
fn log_dir() -> PathBuf {
    home_dir().join(".openclaw-control").join("logs")
}

#[derive(Clone)]
struct BackupSource {
    label: &'static str,
    path: PathBuf,
}

fn appdata_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}
fn localappdata_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}
fn programdata_dir() -> Option<PathBuf> {
    std::env::var_os("PROGRAMDATA").map(PathBuf::from)
}
fn programfiles_dir() -> Option<PathBuf> {
    std::env::var_os("ProgramFiles").map(PathBuf::from)
}
fn programfiles_x86_dir() -> Option<PathBuf> {
    std::env::var_os("ProgramFiles(x86)").map(PathBuf::from)
}

fn read_app_config_value(paths: &[&str]) -> Option<String> {
    let raw = fs::read_to_string(config_file()).ok()?;
    let cfg = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    for path in paths {
        let mut current = &cfg;
        for part in path.split('.') {
            current = current.get(part)?;
        }
        if let Some(value) = current.as_str().filter(|s| !s.trim().is_empty()) {
            return Some(value.to_string());
        }
    }
    None
}

fn configured_openclaw_dir() -> PathBuf {
    read_app_config_value(&[
        "openclaw.install_dir",
        "openclaw.installDir",
        "workspace.openclawDir",
        "agents.defaults.openclawDir",
    ])
    .map(PathBuf::from)
    .unwrap_or_else(|| home_dir().join(".openclaw"))
}

fn configured_openclaw_workspace_dir() -> PathBuf {
    read_app_config_value(&[
        "openclaw.workspace_dir",
        "agents.defaults.workspace",
        "agents.defaults.workspaceDir",
    ])
    .map(PathBuf::from)
    .unwrap_or_else(|| home_dir().join("workspace"))
}

fn backup_sources(app: &str) -> Result<Vec<BackupSource>, String> {
    let home = home_dir();
    let sources = match app {
        "openclaw" => vec![
            BackupSource {
                label: ".openclaw",
                path: configured_openclaw_dir(),
            },
            BackupSource {
                label: "workspace",
                path: configured_openclaw_workspace_dir(),
            },
        ],
        "claude-code" => {
            let mut v = vec![BackupSource {
                label: "home_dot_claude",
                path: home.join(".claude"),
            }];
            if let Some(p) = programdata_dir() {
                v.push(BackupSource {
                    label: "programdata_claudecode",
                    path: p.join("ClaudeCode"),
                });
            }
            v
        }
        "n8n" => vec![BackupSource {
            label: "home_dot_n8n",
            path: home.join(".n8n"),
        }],
        "ngrok" => {
            let mut v = vec![BackupSource {
                label: "home_dot_ngrok2",
                path: home.join(".ngrok2"),
            }];
            if let Some(p) = localappdata_dir() {
                v.push(BackupSource {
                    label: "localappdata_ngrok",
                    path: p.join("ngrok"),
                });
                v.push(BackupSource {
                    label: "localappdata_ngrok_tunnel",
                    path: p.join("Ngrok Tunnel"),
                });
            }
            if let Some(p) = appdata_dir() {
                v.push(BackupSource {
                    label: "appdata_ngrok",
                    path: p.join("ngrok"),
                });
            }
            v
        }
        _ => return Err("Unsupported app".into()),
    };

    Ok(sources)
}

fn restore_target(app: &str, label: &str) -> Option<PathBuf> {
    let home = home_dir();
    match (app, label) {
        ("openclaw", ".openclaw") => Some(configured_openclaw_dir()),
        ("openclaw", "workspace") => Some(configured_openclaw_workspace_dir()),
        ("openclaw", "home_dot_openclaw") => Some(home.join(".openclaw")),
        ("openclaw", "home_openclaw_control") => Some(home.join(".openclaw-control")),
        ("openclaw", "agents_control_center_config") => Some(config_dir()),
        ("openclaw", "home_workspace")
        | ("openclaw", "configured_workspace")
        | ("openclaw", "configured_workspace_alt") => Some(configured_openclaw_workspace_dir()),
        ("claude-code", "home_dot_claude") => Some(home.join(".claude")),
        ("claude-code", "programdata_claudecode") => {
            programdata_dir().map(|p| p.join("ClaudeCode"))
        }
        ("n8n", "home_dot_n8n") => Some(home.join(".n8n")),
        ("ngrok", "home_dot_ngrok2") => Some(home.join(".ngrok2")),
        ("ngrok", "localappdata_ngrok") => localappdata_dir().map(|p| p.join("ngrok")),
        ("ngrok", "localappdata_ngrok_tunnel") => {
            localappdata_dir().map(|p| p.join("Ngrok Tunnel"))
        }
        ("ngrok", "appdata_ngrok") => appdata_dir().map(|p| p.join("ngrok")),
        _ => None,
    }
}

fn validate_backup_manifest_value(
    manifest: &serde_json::Value,
    app_key: &str,
) -> Result<Vec<String>, String> {
    let format = manifest
        .get("format")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Backup manifest is missing format".to_string())?;
    if format != BACKUP_FORMAT {
        return Err("Backup manifest format is not supported".into());
    }

    let app = manifest
        .get("app")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .ok_or_else(|| "Backup manifest is missing app".to_string())?;
    if !app.eq_ignore_ascii_case(app_key) {
        return Err(format!(
            "Backup is for '{}', but '{}' was selected",
            app, app_key
        ));
    }

    let sources = manifest
        .get("sources")
        .and_then(|v| v.as_array())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "Backup manifest is missing sources".to_string())?;

    let mut labels = Vec::new();
    for source in sources {
        let label = source
            .get("folder")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| "Backup manifest contains an invalid source".to_string())?;
        if restore_target(app_key, label).is_none() {
            return Err(format!(
                "Backup manifest contains unknown source: {}",
                label
            ));
        }
        if !labels
            .iter()
            .any(|v: &String| v.eq_ignore_ascii_case(label))
        {
            labels.push(label.to_string());
        }
    }

    Ok(labels)
}

fn validate_backup_manifest(
    archive: &mut ZipArchive<File>,
    app_key: &str,
) -> Result<Vec<String>, String> {
    let mut entry = archive
        .by_name("manifest.json")
        .map_err(|_| "Backup manifest.json is required".to_string())?;
    let mut raw = String::new();
    entry.read_to_string(&mut raw).map_err(|e| e.to_string())?;
    let manifest = serde_json::from_str::<serde_json::Value>(&raw)
        .map_err(|e| format!("Backup manifest.json is invalid: {}", e))?;
    validate_backup_manifest_value(&manifest, app_key)
}

fn safe_zip_name(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn add_path_to_zip(
    zip: &mut ZipWriter<File>,
    root: &Path,
    current: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<u64, String> {
    let meta = fs::symlink_metadata(current).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Ok(0);
    }

    let rel = current.strip_prefix(root).map_err(|e| e.to_string())?;
    let rel_name = rel.to_string_lossy().replace('\\', "/");
    let zip_name = if zip_prefix.is_empty() {
        rel_name.clone()
    } else if rel_name.is_empty() {
        zip_prefix.trim_matches('/').to_string()
    } else {
        format!("{}/{}", zip_prefix.trim_matches('/'), rel_name)
    };
    let mut count = 0;

    if meta.is_dir() {
        if !zip_name.is_empty() {
            zip.add_directory(format!("{}/", zip_name), options)
                .map_err(|e| e.to_string())?;
        }
        for entry in fs::read_dir(current).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            count += add_path_to_zip(zip, root, &path, zip_prefix, options)?;
        }
    } else if meta.is_file() {
        zip.start_file(zip_name, options)
            .map_err(|e| e.to_string())?;
        let mut file = File::open(current).map_err(|e| e.to_string())?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if read == 0 {
                break;
            }
            zip.write_all(&buffer[..read]).map_err(|e| e.to_string())?;
        }
        count += 1;
    }

    if count == 0 && zip_prefix.is_empty() {
        Ok(0)
    } else {
        Ok(count)
    }
}

fn copy_source_to_zip(
    zip: &mut ZipWriter<File>,
    app: &str,
    source: &BackupSource,
    options: FileOptions,
) -> Result<u64, String> {
    if !source.path.exists() {
        return Ok(0);
    }
    let prefix = if app == "openclaw" {
        source.label.to_string()
    } else {
        let source_name = source
            .path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or(source.label);
        PathBuf::from(source.label)
            .join(source_name)
            .to_string_lossy()
            .replace('\\', "/")
    };
    zip.add_directory(format!("{}/", prefix), options)
        .map_err(|e| e.to_string())?;
    if source.path.is_file() {
        let root = source.path.parent().unwrap_or_else(|| Path::new(""));
        return add_path_to_zip(zip, root, &source.path, &prefix, options);
    }
    let mut count = 0;
    for entry in fs::read_dir(&source.path).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        count += add_path_to_zip(zip, &source.path, &path, &prefix, options)?;
    }
    Ok(count)
}

fn is_safe_restore_path(path: &Path) -> bool {
    !path.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    })
}

fn join_restore_target(target_root: &Path, remainder: PathBuf) -> PathBuf {
    if remainder.components().next().is_none() {
        return target_root.to_path_buf();
    }
    let duplicate_target_folder = remainder
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .zip(target_root.file_name().and_then(|v| v.to_str()))
        .map(|(first, target_name)| first.eq_ignore_ascii_case(target_name))
        .unwrap_or(false);
    if duplicate_target_folder {
        let mut rem_parts = remainder.components();
        let _ = rem_parts.next();
        target_root.join(rem_parts.collect::<PathBuf>())
    } else {
        target_root.join(remainder)
    }
}

fn openclaw_base_dir(input: &str) -> PathBuf {
    if input.trim().is_empty() {
        return home_dir().join(".openclaw");
    }
    let p = PathBuf::from(input);
    if p.file_name()
        .map(|name| name.eq_ignore_ascii_case(".openclaw"))
        .unwrap_or(false)
    {
        p
    } else {
        p.join(".openclaw")
    }
}

fn resolve_openclaw_config_path() -> PathBuf {
    if let Some(dir) = read_app_config_value(&[
        "openclaw.install_dir",
        "openclaw.installDir",
        "workspace.openclawDir",
        "agents.defaults.openclawDir",
    ]) {
        return openclaw_base_dir(&dir).join("openclaw.json");
    }
    home_dir().join(".openclaw").join("openclaw.json")
}

fn json_string_at<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for part in path {
        current = current.get(*part)?;
    }
    current.as_str().map(str::trim).filter(|s| !s.is_empty())
}

fn requested_openclaw_base(config: &serde_json::Value) -> Option<PathBuf> {
    json_string_at(config, &["agents", "defaults", "openclawDir"])
        .or_else(|| json_string_at(config, &["openclaw", "install_dir"]))
        .or_else(|| json_string_at(config, &["openclaw", "installDir"]))
        .map(openclaw_base_dir)
}

fn requested_workspace_root(config: &serde_json::Value) -> Option<PathBuf> {
    json_string_at(config, &["agents", "defaults", "workspace"])
        .or_else(|| json_string_at(config, &["openclaw", "workspace_dir"]))
        .or_else(|| json_string_at(config, &["openclaw", "workspaceDir"]))
        .map(PathBuf::from)
}

fn push_existing_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.exists() && !paths.iter().any(|p| p.eq(&path)) {
        paths.push(path);
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    let key = path
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    if !paths
        .iter()
        .any(|p| p.to_string_lossy().replace('/', "\\").to_ascii_lowercase() == key)
    {
        paths.push(path);
    }
}

fn known_openclaw_bases(extra_base: Option<&Path>) -> Vec<PathBuf> {
    let mut bases = Vec::new();
    push_unique_path(&mut bases, home_dir().join(".openclaw"));
    push_unique_path(&mut bases, configured_openclaw_dir());
    if let Some(base) = extra_base {
        push_unique_path(&mut bases, base.to_path_buf());
    }
    for path in [
        home_dir().join(".openclaw").join("openclaw.json"),
        configured_openclaw_dir().join("openclaw.json"),
    ] {
        if let Some(base) = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|config| requested_openclaw_base(&config))
        {
            push_unique_path(&mut bases, base);
        }
    }
    if let Some(desktop) = dirs::desktop_dir() {
        let desktop_openclaw = desktop.join("openclaw").join(".openclaw");
        if desktop_openclaw.exists() {
            push_unique_path(&mut bases, desktop_openclaw);
        }
    }
    bases
}

fn python_install_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let roots = [
        programfiles_dir(),
        programfiles_x86_dir(),
        localappdata_dir().map(|p| p.join("Programs").join("Python")),
    ];

    for root in roots.into_iter().flatten() {
        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if name.starts_with("python") && path.join("python.exe").exists() {
                push_existing_path(&mut dirs, path.clone());
                push_existing_path(&mut dirs, path.join("Scripts"));
            }
        }
    }
    dirs
}

fn extra_path() -> String {
    let ap = std::env::var("APPDATA").unwrap_or_default();
    let lp = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let sp = std::env::var("PATH").unwrap_or_default();
    let mut parts = vec![
        sp,
        format!("{}\\npm", ap),
        "C:\\Program Files\\nodejs".to_string(),
        format!("{}\\Programs\\nodejs", lp),
        format!("{}\\Microsoft\\WindowsApps", lp),
        "C:\\Program Files\\Git\\cmd".to_string(),
        format!("{}\\ngrok", lp),
        format!("{}\\Ngrok Tunnel", lp),
        home_dir().join(".cargo\\bin").display().to_string(),
    ];
    parts.extend(
        python_install_dirs()
            .into_iter()
            .map(|p| p.display().to_string()),
    );
    parts.join(";")
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let mut c = Command::new(cmd);
    c.args(args).env("PATH", extra_path()).env("NO_COLOR", "1");
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c.output()
        .and_then(|o| Ok(command_output_result(cmd, o)))
        .map_err(|e| e.to_string())
        .and_then(|r| r)
}

fn ps_path() -> &'static str {
    "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
}
fn path_refresh() -> &'static str {
    "$env:Path=[Environment]::GetEnvironmentVariable('Path','Machine')+';'+[Environment]::GetEnvironmentVariable('Path','User')+';'+(Join-Path $env:APPDATA 'npm')+';C:\\Program Files\\nodejs;'+(Join-Path $env:LOCALAPPDATA 'Programs\\nodejs')+';'+(Join-Path $env:LOCALAPPDATA 'Microsoft\\WindowsApps')+';C:\\Program Files\\Git\\cmd;'+(Join-Path $env:LOCALAPPDATA 'ngrok')+';'+(Join-Path $env:LOCALAPPDATA 'Ngrok Tunnel'); "
}

fn command_output_text(o: &Output) -> String {
    let s = String::from_utf8_lossy(&o.stdout);
    let e = String::from_utf8_lossy(&o.stderr);
    format!(
        "{}{}",
        s.trim(),
        if e.trim().is_empty() {
            "".into()
        } else {
            format!("\n{}", e.trim())
        }
    )
    .trim()
    .to_string()
}

fn command_output_result(label: &str, o: Output) -> Result<String, String> {
    let text = command_output_text(&o);
    if o.status.success() {
        Ok(text)
    } else {
        let code = o
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".into());
        if text.is_empty() {
            Err(format!("{} failed with exit code {}", label, code))
        } else {
            Err(format!(
                "{} failed with exit code {}: {}",
                label, code, text
            ))
        }
    }
}

fn run_ps(script: &str) -> Result<String, String> {
    let full = format!("{}{}", path_refresh(), script);
    let mut c = Command::new(ps_path());
    c.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &full,
    ]);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c.output()
        .and_then(|o| Ok(command_output_result("PowerShell", o)))
        .map_err(|e| e.to_string())
        .and_then(|r| r)
}

fn run_ps_with_env(script: &str, extra_env: &[(String, String)]) -> Result<String, String> {
    let full = format!("{}{}", path_refresh(), script);
    let mut c = Command::new(ps_path());
    c.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &full,
    ]);
    c.env("PATH", extra_path()).env("NO_COLOR", "1");
    for (k, v) in extra_env {
        c.env(k, v);
    }
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c.output()
        .and_then(|o| Ok(command_output_result("PowerShell", o)))
        .map_err(|e| e.to_string())
        .and_then(|r| r)
}

fn gateway_ready_from_status(status: &str) -> bool {
    let s = status.to_lowercase();
    if s.contains("not ready")
        || s.contains("stopped")
        || s.contains("not running")
        || s.contains("failed")
        || s.contains("error")
    {
        return false;
    }
    s.contains("connectivity probe: ok")
        || s.contains("listener detected")
        || s.contains("runtime: running")
        || s.contains("gateway: running")
        || s.contains("status: running")
        || s.contains("gateway is running")
        || s.contains("gateway is ready")
        || s.contains("listening on")
        || s.contains("ready")
}

fn gateway_status_output() -> Result<String, String> {
    let env = configured_openclaw_env();
    run_ps_with_env("openclaw gateway status", &env)
}

fn dashboard_url_from_status(status: &str) -> Option<String> {
    for line in status.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("Dashboard:") else {
            continue;
        };
        let url = rest.trim();
        if url.starts_with("http://") || url.starts_with("https://") {
            return Some(url.to_string());
        }
    }
    None
}

fn dashboard_url_from_output(output: &str) -> Option<String> {
    for token in output.split_whitespace() {
        let url = token.trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == ';');
        if url.starts_with("http://") || url.starts_with("https://") {
            return Some(url.to_string());
        }
    }
    None
}

fn configured_gateway_auth_token() -> Option<String> {
    let raw = fs::read_to_string(resolve_openclaw_config_path()).ok()?;
    let config = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    let auth = config.get("gateway")?.get("auth")?;
    let mode = auth
        .get("mode")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if !mode.eq_ignore_ascii_case("token") {
        return None;
    }
    auth.get("token")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn percent_encode_query_value(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~');
        if keep {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{:02X}", byte));
        }
    }
    encoded
}

fn url_has_query_param(url: &str, key: &str) -> bool {
    let Some(query_start) = url.find('?') else {
        return false;
    };
    let query = &url[query_start + 1..];
    let query = query.split('#').next().unwrap_or(query);
    query.split('&').any(|part| {
        part.split('=')
            .next()
            .map(|name| name.eq_ignore_ascii_case(key))
            .unwrap_or(false)
    })
}

fn append_query_param(url: &str, key: &str, value: &str) -> String {
    if value.trim().is_empty() || url_has_query_param(url, key) {
        return url.to_string();
    }
    let (base, fragment) = match url.split_once('#') {
        Some((base, fragment)) => (base, format!("#{}", fragment)),
        None => (url, String::new()),
    };
    let separator = if base.contains('?') {
        if base.ends_with('?') || base.ends_with('&') {
            ""
        } else {
            "&"
        }
    } else {
        "?"
    };
    format!(
        "{}{}{}={}{}",
        base,
        separator,
        key,
        percent_encode_query_value(value),
        fragment
    )
}

fn open_openclaw_dashboard(env: &[(String, String)]) -> Result<String, String> {
    let cli_output = run_ps_with_env("openclaw dashboard --no-open", env)?;
    let status = gateway_status_output().unwrap_or_default();
    let mut url = dashboard_url_from_output(&cli_output)
        .or_else(|| dashboard_url_from_status(&status))
        .unwrap_or_else(|| format!("http://127.0.0.1:{}/", configured_gateway_port()));
    if let Some(token) = configured_gateway_auth_token() {
        url = append_query_param(&url, "token", &token);
    }
    opener::open(&url).map_err(|e| e.to_string())?;
    Ok(format!("dashboard opened: {}", url))
}

fn json_port_at(value: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for part in path {
        current = current.get(*part)?;
    }
    if let Some(value) = current.as_u64().filter(|v| *v > 0 && *v <= u16::MAX as u64) {
        return Some(value.to_string());
    }
    current
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn configured_gateway_port() -> String {
    if let Ok(raw) = fs::read_to_string(resolve_openclaw_config_path()) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(port) = json_port_at(&config, &["gateway", "port"]) {
                return port;
            }
        }
    }
    std::env::var("OPENCLAW_GATEWAY_PORT")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "18789".to_string())
}

fn stop_openclaw_gateway() -> Result<String, String> {
    let port = configured_gateway_port();
    validate_port(&port)?;
    let script = r#"
$ErrorActionPreference = 'Continue'
$port = __PORT__
$out = New-Object System.Collections.Generic.List[string]
try {
    $msg = (openclaw gateway stop 2>&1 | Out-String).Trim()
    if ($msg) { [void]$out.Add($msg) }
} catch {
    [void]$out.Add($_.Exception.Message)
}
Start-Sleep -Milliseconds 700
for ($i = 0; $i -lt 6; $i++) {
    $listeners = Get-NetTCPConnection -LocalPort $port -State Listen -EA SilentlyContinue
    if (-not $listeners) { break }
    foreach ($ownerPid in ($listeners | Select-Object -ExpandProperty OwningProcess -Unique)) {
        if (-not $ownerPid -or $ownerPid -eq $PID) { continue }
        $proc = Get-CimInstance Win32_Process -Filter "ProcessId=$ownerPid" -EA SilentlyContinue
        $cmdLine = [string]$proc.CommandLine
        $name = [string]$proc.Name
        $isOpenClawListener = $cmdLine -match '(?i)openclaw'
        $isOpenClawListener = $isOpenClawListener -or ($name -match '(?i)^openclaw(\.exe)?$')
        if ($isOpenClawListener) {
            Stop-Process -Id $ownerPid -Force -EA SilentlyContinue
            [void]$out.Add("Killed OpenClaw gateway listener PID $ownerPid on port $port")
        }
    }
    Start-Sleep -Milliseconds 800
}
$remaining = Get-NetTCPConnection -LocalPort $port -State Listen -EA SilentlyContinue
if ($remaining) {
    $details = foreach ($ownerPid in ($remaining | Select-Object -ExpandProperty OwningProcess -Unique)) {
        $proc = Get-CimInstance Win32_Process -Filter "ProcessId=$ownerPid" -EA SilentlyContinue
        "$ownerPid $($proc.Name) $($proc.CommandLine)"
    }
    throw ("OpenClaw gateway still listening on port ${port}: " + (($details | Where-Object { $_ }) -join '; '))
}
if ($out.Count -gt 0) {
    ($out -join "`n").Trim()
} else {
    "stopped openclaw gateway"
}
"#
    .replace("__PORT__", &port);
    let env = configured_openclaw_env();
    run_ps_with_env(&script, &env).map(|out| {
        if out.trim().is_empty() {
            "stopped openclaw gateway".into()
        } else {
            out
        }
    })
}

fn restart_openclaw_gateway() -> Result<String, String> {
    let stopped = stop_openclaw_gateway()?;
    let env = configured_openclaw_env();
    let pid = spawn_cmd_owned_env("openclaw", &["gateway", "run"], &env)?;
    thread::sleep(Duration::from_millis(1800));
    let status = gateway_status_output().unwrap_or_default();
    if !gateway_ready_from_status(&status) {
        return Err(format!(
            "Gateway restart started pid {}, but readiness check did not pass yet.\n{}",
            pid,
            status.trim()
        ));
    }
    Ok(format!(
        "{}\nrestarted openclaw gateway pid {}",
        stopped, pid
    ))
}

fn quote_ps_arg(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn validate_ngrok_domain(domain: &str) -> Result<(), String> {
    let d = domain.trim();
    if d.is_empty() {
        return Err("Domain not configured".into());
    }
    if d.len() > 253
        || d.contains('/')
        || d.contains(':')
        || !d
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        || !d.contains('.')
    {
        return Err("Invalid ngrok domain".into());
    }
    Ok(())
}

fn validate_port(port: &str) -> Result<(), String> {
    let p = port.trim();
    if p.is_empty() {
        return Ok(());
    }
    let value: u16 = p.parse().map_err(|_| "Invalid port".to_string())?;
    if value == 0 {
        return Err("Invalid port".into());
    }
    Ok(())
}

fn ps_single_quote(s: &str) -> String {
    quote_ps_arg(s)
}

fn is_safe_ngrok_token(s: &str) -> bool {
    let t = s.trim();
    !t.is_empty()
        && t.len() <= 4096
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'))
}

fn validate_ngrok_token(token: &str) -> Result<(), String> {
    let t = token.trim();
    if t.is_empty() {
        return Err("Authtoken not configured".into());
    }
    if !is_safe_ngrok_token(t) {
        return Err("Invalid authtoken".into());
    }
    Ok(())
}

fn spawn_cmd(cmd: &str, args: &[&str], extra_env: &[(&str, &str)]) -> Result<u32, String> {
    let args_str = std::iter::once(cmd.to_string())
        .chain(args.iter().map(|a| a.to_string()))
        .collect::<Vec<_>>()
        .join(" ");
    let full = format!("{}{}", path_refresh(), args_str);
    let mut c = Command::new(ps_path());
    c.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &full,
    ]);
    c.env("PATH", extra_path()).env("NO_COLOR", "1");
    for (k, v) in extra_env {
        c.env(k, v);
    }
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c.spawn()
        .map(|child| child.id())
        .map_err(|e| format!("{}: {}", cmd, e))
}

fn spawn_cmd_owned_env(
    cmd: &str,
    args: &[&str],
    extra_env: &[(String, String)],
) -> Result<u32, String> {
    let args_str = std::iter::once(cmd.to_string())
        .chain(args.iter().map(|a| a.to_string()))
        .collect::<Vec<_>>()
        .join(" ");
    let full = format!("{}{}", path_refresh(), args_str);
    let mut c = Command::new(ps_path());
    c.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &full,
    ]);
    c.env("PATH", extra_path()).env("NO_COLOR", "1");
    for (k, v) in extra_env {
        c.env(k, v);
    }
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c.spawn()
        .map(|child| child.id())
        .map_err(|e| format!("{}: {}", cmd, e))
}

fn configured_openclaw_env() -> Vec<(String, String)> {
    let base = configured_openclaw_dir();
    vec![
        (
            "OPENCLAW_STATE_DIR".to_string(),
            base.to_string_lossy().to_string(),
        ),
        (
            "OPENCLAW_CONFIG_PATH".to_string(),
            base.join("openclaw.json").to_string_lossy().to_string(),
        ),
    ]
}

fn tool_command_name(name: &str) -> &str {
    match name {
        "claude-code" => "claude",
        other => other,
    }
}

fn tool_version_args(name: &str) -> &'static [&'static str] {
    match name {
        "ngrok" => &["version"],
        _ => &["--version"],
    }
}

fn npm_package_name(tool: &str) -> Option<&'static str> {
    match tool {
        "openclaw" => Some("openclaw"),
        "claude-code" | "claude" => Some("@anthropic-ai/claude-code"),
        "9router" => Some("9router"),
        "n8n" => Some("n8n"),
        _ => None,
    }
}

fn npm_package_version(tool: &str) -> Option<String> {
    let package = npm_package_name(tool)?;
    let mut roots = Vec::new();
    if let Some(appdata) = appdata_dir() {
        roots.push(appdata.join("npm").join("node_modules"));
    }
    if let Ok(root) = run_ps("npm root -g") {
        let trimmed = root.trim();
        if !trimmed.is_empty() {
            roots.push(PathBuf::from(trimmed));
        }
    }

    for root in roots {
        let package_dir = package.split('/').fold(root, |path, part| path.join(part));
        let package_json = package_dir.join("package.json");
        let raw = match fs::read_to_string(package_json) {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        let value = match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if let Some(version) = value
            .get("version")
            .and_then(|v| v.as_str())
            .and_then(extract_version_number)
        {
            return Some(version);
        }
    }
    None
}

fn ps_version_command(cmd: &str, args: &[&str]) -> String {
    let mut parts = vec![format!("& {}", ps_single_quote(cmd))];
    parts.extend(args.iter().map(|arg| ps_single_quote(arg)));
    format!("{} 2>&1 | Out-String", parts.join(" "))
}

fn command_candidates_for_tool(name: &str) -> Vec<String> {
    let cmd = tool_command_name(name).to_string();
    let mut commands = vec![cmd.clone()];
    #[cfg(windows)]
    {
        if !cmd.ends_with(".exe") && !cmd.ends_with(".cmd") {
            commands.push(format!("{}.cmd", cmd));
        }
    }
    if let Some(appdata) = appdata_dir() {
        commands.push(
            appdata
                .join("npm")
                .join(format!("{}.cmd", cmd))
                .display()
                .to_string(),
        );
    }
    match name {
        "node" => {
            commands.push("C:\\Program Files\\nodejs\\node.exe".to_string());
            if let Some(local) = localappdata_dir() {
                commands.push(
                    local
                        .join("Programs")
                        .join("nodejs")
                        .join("node.exe")
                        .display()
                        .to_string(),
                );
            }
        }
        "git" => commands.push("C:\\Program Files\\Git\\cmd\\git.exe".to_string()),
        "ngrok" => {
            if let Some(local) = localappdata_dir() {
                commands.push(
                    local
                        .join("Ngrok Tunnel")
                        .join("ngrok.exe")
                        .display()
                        .to_string(),
                );
                commands.push(local.join("ngrok").join("ngrok.exe").display().to_string());
            }
        }
        _ => {}
    }
    commands
}

fn python_candidate_exes() -> Vec<PathBuf> {
    let mut exes = Vec::new();
    for dir in python_install_dirs() {
        let exe = dir.join("python.exe");
        if exe.exists() {
            push_existing_path(&mut exes, exe);
        }
    }
    push_existing_path(&mut exes, PathBuf::from("C:\\Windows\\py.exe"));
    exes
}

fn python_version() -> Option<String> {
    for (cmd, args) in [
        ("python", vec!["--version"]),
        ("py", vec!["-3", "--version"]),
        ("py", vec!["--version"]),
    ] {
        if let Some(version) = run_cmd(cmd, &args)
            .ok()
            .and_then(|out| extract_version_number(&out))
        {
            return Some(version);
        }
    }
    for exe in python_candidate_exes() {
        let exe = exe.display().to_string();
        if exe.ends_with("\\py.exe") {
            if let Some(version) = run_cmd(&exe, &["-3", "--version"])
                .ok()
                .and_then(|out| extract_version_number(&out))
            {
                return Some(version);
            }
        } else if let Some(version) = run_cmd(&exe, &["--version"])
            .ok()
            .and_then(|out| extract_version_number(&out))
        {
            return Some(version);
        }
    }
    None
}

fn tool_command_version(name: &str) -> Option<String> {
    if name == "python" {
        return python_version();
    }
    let version_args = tool_version_args(name);
    for cmd in command_candidates_for_tool(name) {
        if let Some(version) = run_cmd(&cmd, version_args)
            .ok()
            .and_then(|out| extract_version_number(&out))
        {
            return Some(version);
        }
    }

    let cmd = tool_command_name(name);
    run_ps(&ps_version_command(cmd, version_args))
        .ok()
        .and_then(|out| extract_version_number(&out))
}

fn is_tool_installed(name: &str) -> bool {
    if tool_version(name).is_some() {
        return true;
    }
    let cmd = tool_command_name(name);
    // Fallback: check Get-Command (but verify it's not a WindowsApps stub)
    let ps = if name == "python" || name == "node" {
        format!(
            "$c=Get-Command '{}' -All -EA SilentlyContinue | Where-Object {{ $_.Source -notmatch 'WindowsApps' }} | Select-Object -First 1; if($c){{'true'}}else{{'false'}}",
            cmd
        )
    } else {
        format!(
            "$cmd='{}'; $found=$false; if(Get-Command $cmd -EA SilentlyContinue){{$found=$true}}; if(-not $found -and (Get-Command npm -EA SilentlyContinue)){{$root=(npm root -g 2>$null); if($root -and (Test-Path (Join-Path $root '{}'))){{$found=$true}}}}; if($found){{'true'}}else{{'false'}}",
            cmd, name
        )
    };
    let cmd_exists = run_ps(&ps)
        .map(|s| s.to_lowercase().contains("true"))
        .unwrap_or(false);
    if !cmd_exists {
        return false;
    }
    // Command exists but version failed — try via PowerShell Invoke-Expression
    let version_cmd = match name {
        "ngrok" => format!("{} version", cmd),
        _ => format!("{} --version", cmd),
    };
    let ps_version = format!("try {{ $out = Invoke-Expression '{}' 2>&1 | Out-String; $out.Trim() }} catch {{ 'NOT_INSTALLED' }}", version_cmd);
    if let Ok(out) = run_ps(&ps_version) {
        if extract_version_number(&out).is_some() {
            return true;
        }
    }
    false
}

fn extract_version_number(s: &str) -> Option<String> {
    let normalized = s.replace('\r', "\n");
    for line in normalized.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        let lower = l.to_lowercase();
        if [
            "not recognized",
            "is not recognized",
            "not found",
            "cannot find",
            "error:",
            "err!",
            "could not",
        ]
        .iter()
        .any(|b| lower.contains(b))
        {
            continue;
        }
        let chars: Vec<char> = l.chars().collect();
        for i in 0..chars.len() {
            if chars[i].is_ascii_digit()
                || (chars[i] == 'v' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
            {
                let mut j = i;
                while j < chars.len()
                    && (chars[j].is_ascii_alphanumeric()
                        || chars[j] == '.'
                        || chars[j] == '-'
                        || chars[j] == '_'
                        || chars[j] == '+')
                {
                    j += 1;
                }
                let candidate: String = chars[i..j].iter().collect();
                let cleaned = candidate
                    .trim_start_matches('v')
                    .trim_matches('.')
                    .to_string();
                if cleaned
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

fn clean_version_output(s: String) -> String {
    extract_version_number(&s).unwrap_or_else(|| "Not Installed".into())
}

fn tool_version(name: &str) -> Option<String> {
    tool_command_version(name).or_else(|| npm_package_version(name))
}

fn display_name_for_tool(tool: &str) -> &str {
    match tool {
        "node" => "Node.js",
        "openclaw" => "OpenClaw",
        "claude-code" | "claude" => "Claude Code",
        "9router" => "9router",
        "n8n" => "n8n",
        "ngrok" => "ngrok",
        "git" => "Git",
        "python" => "Python",
        _ => tool,
    }
}

fn validate_tool_version(version: &str) -> Result<(), String> {
    let v = version.trim();
    if v.is_empty() {
        return Ok(());
    }
    if v.len() > 64
        || !v
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
    {
        return Err("Invalid version".into());
    }
    Ok(())
}

fn version_matches(actual: &str, expected: &str) -> bool {
    let normalize = |s: &str| s.trim().trim_start_matches('v').to_ascii_lowercase();
    let a = normalize(actual);
    let e = normalize(expected);
    e.is_empty() || a == e || a.starts_with(&format!("{}.", e)) || a.contains(&e)
}

fn action_output_version(tool: &str, expected: &str, output: &str) -> Option<String> {
    let normalized_output = output.to_ascii_lowercase();
    let expected_norm = expected.trim().trim_start_matches('v').to_ascii_lowercase();
    if !expected_norm.is_empty() && normalized_output.contains(&expected_norm) {
        return Some(expected.trim().trim_start_matches('v').to_string());
    }

    let tool_name = display_name_for_tool(tool).to_ascii_lowercase();
    for line in output.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains(&tool_name) || lower.contains(tool) {
            if let Some(version) = extract_version_number(line) {
                return Some(version);
            }
        }
    }
    None
}

fn detected_version_after_action(
    tool: &str,
    expected: &str,
    result: &Result<String, String>,
) -> Option<String> {
    tool_command_version(tool)
        .or_else(|| npm_package_version(tool))
        .or_else(|| {
            result
                .as_ref()
                .ok()
                .and_then(|out| action_output_version(tool, expected, out))
        })
}

fn run_install_and_verify(tool: &str, version: &str, script: String) -> Result<String, String> {
    validate_tool_version(version)?;
    let display = display_name_for_tool(tool);
    let result = run_ps(&script);
    thread::sleep(std::time::Duration::from_millis(600));
    if let Err(e) = &result {
        let lower = e.to_ascii_lowercase();
        if lower.contains("npm error") || lower.contains("npm err!") {
            return Err(format!("{} install failed: {}", display, e));
        }
    }
    let detected = detected_version_after_action(tool, version, &result);
    match detected {
        Some(actual) if version_matches(&actual, version) => {
            Ok(format!("{} installed: {}", display, actual))
        }
        Some(actual) => Err(format!(
            "{} install version mismatch. Expected {}, found {}",
            display, version, actual
        )),
        None => match result {
            Ok(out) if !out.trim().is_empty() => {
                Err(format!("{} install did not complete: {}", display, out))
            }
            Ok(_) => Err(format!("{} install did not complete", display)),
            Err(e) => Err(format!("{} install failed: {}", display, e)),
        },
    }
}

fn is_benign_update_result(message: &str) -> bool {
    let lower = message.to_lowercase();
    [
        "no available upgrade found",
        "no newer package versions are available",
        "no installed package found matching input criteria",
        "no applicable update found",
        "no updates available",
        "already installed",
        "already up-to-date",
        "already up to date",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn run_update_and_verify(tool: &str, script: &str) -> Result<String, String> {
    let display = display_name_for_tool(tool);
    let result = run_ps(script);
    thread::sleep(std::time::Duration::from_millis(600));
    let detected = detected_version_after_action(tool, "", &result);
    match (result, detected) {
        (Ok(_), Some(v)) => Ok(format!("{} updated: {}", display, v)),
        (Ok(_), None) if is_tool_installed(tool) => Ok(format!("{} update completed", display)),
        (Ok(_), None) => Err(format!(
            "{} update finished but the command is not detected",
            display
        )),
        (Err(e), Some(v)) if is_benign_update_result(&e) => {
            Ok(format!("{} is already up to date: {}", display, v))
        }
        (Err(e), None) if is_benign_update_result(&e) && is_tool_installed(tool) => {
            Ok(format!("{} is already up to date", display))
        }
        (Err(e), _) => Err(e),
    }
}

fn run_ps_async(script: String) -> Result<String, String> {
    let full = format!("{}{}", path_refresh(), script);
    let mut c = Command::new(ps_path());
    c.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &full,
    ]);
    c.env("PATH", extra_path()).env("NO_COLOR", "1");
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c.spawn()
        .map(|child| format!("started background pid {}", child.id()))
        .map_err(|e| e.to_string())
}

fn run_ps_async_with_env(script: String, extra_env: &[(String, String)]) -> Result<String, String> {
    let full = format!("{}{}", path_refresh(), script);
    let mut c = Command::new(ps_path());
    c.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &full,
    ]);
    c.env("PATH", extra_path()).env("NO_COLOR", "1");
    for (k, v) in extra_env {
        c.env(k, v);
    }
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c.spawn()
        .map(|child| format!("started background pid {}", child.id()))
        .map_err(|e| e.to_string())
}

fn spawn_visible_terminal(script: &str) -> Result<String, String> {
    let full = format!("{}{}", path_refresh(), script);
    let mut c = Command::new(ps_path());
    c.args(["-NoExit", "-ExecutionPolicy", "Bypass", "-Command", &full]);
    c.env("PATH", extra_path()).env("NO_COLOR", "1");
    c.spawn()
        .map(|child| format!("started visible terminal pid {}", child.id()))
        .map_err(|e| e.to_string())
}

fn validate_terminal_cmd(cmd: &str) -> Result<(), String> {
    let raw = cmd.trim();
    if raw.is_empty() {
        return Err("Command is empty".into());
    }
    Ok(())
}

fn make_download_script(
    url: &str,
    file_name: &str,
    install_cmd: &str,
    verify_cmd: &str,
    success_msg: &str,
) -> String {
    format!(
        "$ErrorActionPreference='Stop'; [Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; \
$dest=Join-Path $env:TEMP '{}'; \
$wc=New-Object System.Net.WebClient; \
$wc.DownloadFile('{}',$dest); \
{}; \
Remove-Item $dest -Force -EA SilentlyContinue; \
if({}){{'{}'}}else{{throw 'Install failed'}}",
        file_name, url, install_cmd, verify_cmd, success_msg
    )
}

fn winget_node_install_script(version: &str) -> String {
    let version_arg = ps_single_quote(version.trim());
    r#"
$ErrorActionPreference = 'Stop'
$requested = __NODE_VERSION__
function Refresh-ToolPath {
    $machine = [Environment]::GetEnvironmentVariable('Path','Machine')
    $user = [Environment]::GetEnvironmentVariable('Path','User')
    $windowsApps = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps'
    $localNode = Join-Path $env:LOCALAPPDATA 'Programs\nodejs'
    $npm = Join-Path $env:APPDATA 'npm'
    $env:Path = "$machine;$user;$windowsApps;C:\Program Files\nodejs;$localNode;$npm"
}
function Test-WingetReady {
    Refresh-ToolPath
    $cmd = Get-Command winget -EA SilentlyContinue
    if (-not $cmd) { return $false }
    try {
        & $cmd.Source --version | Out-Null
        return $LASTEXITCODE -eq 0
    } catch {
        return $false
    }
}
function Ensure-Winget {
    if (Test-WingetReady) { return }

    try {
        Add-AppxPackage -RegisterByFamilyName -MainPackage Microsoft.DesktopAppInstaller_8wekyb3d8bbwe -EA Stop
    } catch {
        Write-Host "WinGet registration skipped/failed: $($_.Exception.Message)"
    }
    if (Test-WingetReady) { return }

    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $bundle = Join-Path $env:TEMP 'Microsoft.DesktopAppInstaller.msixbundle'
    $wc = New-Object System.Net.WebClient
    $wc.DownloadFile('https://aka.ms/getwinget', $bundle)
    try {
        Add-AppxPackage -Path $bundle -EA Stop
    } catch {
        Write-Host "WinGet bundle install failed once: $($_.Exception.Message)"
        $deps = @()
        $arch = if ($env:PROCESSOR_ARCHITECTURE -match 'ARM64' -or $env:PROCESSOR_ARCHITEW6432 -match 'ARM64') { 'arm64' } elseif ($env:PROCESSOR_ARCHITECTURE -match 'AMD64' -or $env:PROCESSOR_ARCHITEW6432 -match 'AMD64') { 'x64' } else { 'x86' }
        $vclibs = Join-Path $env:TEMP "Microsoft.VCLibs.$arch.14.00.Desktop.appx"
        $wc.DownloadFile("https://aka.ms/Microsoft.VCLibs.$arch.14.00.Desktop.appx", $vclibs)
        $deps += $vclibs
        if ($arch -eq 'x64') {
            $xaml = Join-Path $env:TEMP 'Microsoft.UI.Xaml.2.8.x64.appx'
            $wc.DownloadFile('https://github.com/microsoft/microsoft-ui-xaml/releases/download/v2.8.6/Microsoft.UI.Xaml.2.8.x64.appx', $xaml)
            $deps += $xaml
        }
        Add-AppxPackage -Path $bundle -DependencyPath $deps -EA Stop
    } finally {
        Remove-Item $bundle -Force -EA SilentlyContinue
    }

    if (-not (Test-WingetReady)) {
        throw 'WinGet install finished but winget command is still not available. Restart Windows or install App Installer from Microsoft Store.'
    }
}

Ensure-Winget
Refresh-ToolPath
if ([string]::IsNullOrWhiteSpace($requested)) {
    winget install --id OpenJS.NodeJS.LTS --exact --accept-package-agreements --accept-source-agreements --disable-interactivity
} else {
    $wingetVersion = $requested.TrimStart('v')
    winget install --id OpenJS.NodeJS.LTS --exact --version $wingetVersion --accept-package-agreements --accept-source-agreements --disable-interactivity
}
if ($LASTEXITCODE -ne 0) { throw "Node.js winget install exit code $LASTEXITCODE" }
Refresh-ToolPath
node --version
npm --version
"#
    .replace("__NODE_VERSION__", &version_arg)
}

fn npm_global_package_script(
    package: &str,
    command: &str,
    version: &str,
    update: bool,
    cleanup_first: bool,
) -> String {
    let package_arg = ps_single_quote(package);
    let command_arg = ps_single_quote(command);
    let spec = if update {
        format!("{}@latest", package)
    } else if version.trim().is_empty() {
        package.to_string()
    } else {
        format!("{}@{}", package, version.trim())
    };
    let spec_arg = ps_single_quote(&spec);
    let package_parts = package
        .split('/')
        .map(ps_single_quote)
        .collect::<Vec<_>>()
        .join(",");
    let cleanup = if cleanup_first {
        r#"
npm uninstall -g $pkg 2>$null | Out-Null
Remove-Item -LiteralPath $pkgDir -Recurse -Force -EA SilentlyContinue
Remove-Item -Path (Join-Path $env:APPDATA "npm\claude*") -Force -EA SilentlyContinue
npm cache verify 2>$null | Out-Null
"#
    } else {
        ""
    };
    format!(
        r#"
$ErrorActionPreference = 'Continue'
$pkg = {package_arg}
$cmd = {command_arg}
$spec = {spec_arg}
$packageParts = @({package_parts})
function Get-GlobalPackageDir {{
    $root = npm root -g 2>$null
    if (-not $root) {{ $root = Join-Path $env:APPDATA 'npm\node_modules' }}
    $dir = $root
    foreach ($part in $packageParts) {{ $dir = Join-Path $dir $part }}
    $dir
}}
function Get-GlobalPackageVersion {{
    $json = Join-Path (Get-GlobalPackageDir) 'package.json'
    if (Test-Path -LiteralPath $json) {{
        try {{ return ((Get-Content -LiteralPath $json -Raw | ConvertFrom-Json).version) }} catch {{ return $null }}
    }}
    return $null
}}
$pkgDir = Get-GlobalPackageDir
{cleanup}
npm install -g $spec
$code = $LASTEXITCODE
if ($code -ne 0) {{
    npm uninstall -g $pkg 2>$null | Out-Null
    Remove-Item -LiteralPath $pkgDir -Recurse -Force -EA SilentlyContinue
    npm install -g $spec
    $code = $LASTEXITCODE
}}
if ($code -ne 0) {{ exit $code }}
$out = & $cmd --version 2>&1 | Out-String
if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($out)) {{
    $out.Trim()
}} else {{
    $ver = Get-GlobalPackageVersion
    if ($ver) {{ $ver }} else {{ exit 1 }}
}}
"#
    )
}

#[tauri::command]
fn gateway_status() -> HashMap<String, bool> {
    let ready = gateway_status_output()
        .map(|s| gateway_ready_from_status(&s))
        .unwrap_or(false);
    HashMap::from([("ready".into(), ready), ("running".into(), ready)])
}

fn persist_openclaw_setup_paths(openclaw_base: &Path, workspace_root: &Path) -> Result<(), String> {
    fs::create_dir_all(config_dir()).map_err(|e| e.to_string())?;
    let mut cfg = if let Ok(raw) = fs::read_to_string(config_file()) {
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if !cfg.is_object() {
        cfg = serde_json::json!({});
    }

    cfg["openclaw"]["install_dir"] = serde_json::json!(openclaw_base.to_string_lossy().to_string());
    cfg["openclaw"]["workspace_dir"] =
        serde_json::json!(workspace_root.to_string_lossy().to_string());
    cfg["agents"]["defaults"]["openclawDir"] =
        serde_json::json!(openclaw_base.to_string_lossy().to_string());
    cfg["agents"]["defaults"]["workspace"] =
        serde_json::json!(workspace_root.to_string_lossy().to_string());

    fs::write(
        config_file(),
        serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn backup_existing_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "openclaw.json".to_string());
    let backup = path.with_file_name(format!("{}.{}.bak", file_name, stamp));
    fs::copy(path, backup)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn write_json_pretty(path: &Path, value: &serde_json::Value, backup: bool) -> Result<bool, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = format!(
        "{}\n",
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())?
    );
    if path.exists() {
        if let Ok(existing) = fs::read_to_string(path) {
            if existing == content {
                return Ok(false);
            }
        }
        if backup {
            backup_existing_file(path)?;
        }
    }
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(true)
}

fn write_openclaw_config_file(path: &Path, config: &mut serde_json::Value) -> Result<bool, String> {
    repair_openclaw_config_value(config, path.parent());
    normalize_openclaw_model_config(config);
    sync_agent_default_model_refs(config);
    write_json_pretty(path, config, true)
}

fn merge_missing_json(target: &mut serde_json::Value, defaults: &serde_json::Value) {
    match (target, defaults) {
        (serde_json::Value::Object(target_obj), serde_json::Value::Object(default_obj)) => {
            for (key, default_value) in default_obj {
                match target_obj.get_mut(key) {
                    Some(current_value) => merge_missing_json(current_value, default_value),
                    None => {
                        target_obj.insert(key.clone(), default_value.clone());
                    }
                }
            }
        }
        (target_value, default_value) if target_value.is_null() => {
            *target_value = default_value.clone();
        }
        _ => {}
    }
}

fn write_if_missing(path: &Path, content: &str) -> Result<(), String> {
    if !path.exists() {
        fs::write(path, content).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn ensure_json_object(value: &mut serde_json::Value) {
    if !value.is_object() {
        *value = serde_json::json!({});
    }
}

fn ensure_json_object_at<'a>(
    value: &'a mut serde_json::Value,
    key: &str,
) -> &'a mut serde_json::Value {
    ensure_json_object(value);
    if !value.get(key).map(|v| v.is_object()).unwrap_or(false) {
        value[key] = serde_json::json!({});
    }
    &mut value[key]
}

fn string_missing_or_empty(value: &serde_json::Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_none()
}

fn path_as_config_string(path: &Path) -> String {
    path.to_string_lossy()
        .replace("\\\\", "/")
        .replace("\\", "/")
}

fn openclaw_default_config(openclaw_base: Option<&Path>) -> serde_json::Value {
    let token_file = openclaw_base
        .map(|base| path_as_config_string(&base.join("telegram").join("bot-token.txt")))
        .unwrap_or_default();
    let workspace = path_as_config_string(&configured_openclaw_workspace_dir());

    serde_json::json!({
        "agents": {
            "defaults": {
                "bootstrapMaxChars": 4000,
                "bootstrapTotalMaxChars": 12000,
                "contextInjection": "continuation-skip",
                "memorySearch": {
                    "cache": { "enabled": true },
                    "enabled": true,
                    "experimental": { "sessionMemory": false },
                    "provider": "gemini",
                    "query": {
                        "hybrid": {
                            "enabled": true,
                            "mmr": { "enabled": true },
                            "temporalDecay": {
                                "enabled": true,
                                "halfLifeDays": 30
                            }
                        }
                    },
                    "sources": ["memory"]
                },
                "model": { "primary": "" },
                "models": {},
                "workspace": workspace
            }
        },
        "channels": {
            "telegram": {
                "allowFrom": ["*"],
                "dmPolicy": "open",
                "enabled": false,
                "groups": { "*": { "requireMention": false } },
                "streaming": { "mode": "off" },
                "tokenFile": token_file
            }
        },
        "gateway": {
            "auth": { "mode": "token", "token": "" },
            "bind": "lan",
            "controlUi": {
                "allowedOrigins": [
                    "http://localhost:18789",
                    "http://127.0.0.1:18789"
                ]
            },
            "mode": "local",
            "nodes": {}
        },
        "messages": {
            "groupChat": { "visibleReplies": "message_tool" }
        },
        "meta": {
            "lastTouchedAt": "",
            "lastTouchedVersion": ""
        },
        "models": {
            "mode": "merge",
            "providers": {}
        },
        "plugins": {
            "entries": {
                "memory-core": { "config": {} },
                "openai": { "config": {}, "enabled": true },
                "telegram": { "config": {}, "enabled": false }
            }
        },
        "skills": { "entries": {} },
        "talk": {
            "provider": "openai",
            "providers": { "openai": {} }
        },
        "tools": {
            "exec": {
                "ask": "off",
                "host": "auto",
                "security": "full"
            },
            "web": {
                "fetch": { "enabled": false },
                "search": {
                    "enabled": false,
                    "openaiCodex": {},
                    "provider": "brave"
                }
            }
        },
        "wizard": {
            "lastRunAt": "",
            "lastRunCommand": "doctor",
            "lastRunMode": "local",
            "lastRunVersion": ""
        }
    })
}

fn normalize_openclaw_channel_policy(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "open" => Some("open"),
        "allowlist" | "allow-list" | "allow_list" => Some("allowlist"),
        "pairing" | "pair" => Some("pairing"),
        "disabled" | "disable" | "off" => Some("pairing"),
        _ => None,
    }
}

fn repair_openclaw_config_value(config: &mut serde_json::Value, openclaw_base: Option<&Path>) {
    ensure_json_object(config);
    merge_missing_json(config, &openclaw_default_config(openclaw_base));

    let gateway = ensure_json_object_at(config, "gateway");
    if string_missing_or_empty(gateway, "mode") {
        gateway["mode"] = serde_json::json!("local");
    }
    let auth = ensure_json_object_at(gateway, "auth");
    let token_is_string = auth.get("token").map(|v| v.is_string()).unwrap_or(false);
    let has_token = auth
        .get("token")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some();
    if !token_is_string {
        if let Some(auth_obj) = auth.as_object_mut() {
            auth_obj.remove("token");
        }
    }
    if string_missing_or_empty(auth, "mode") {
        auth["mode"] = serde_json::json!(if has_token { "token" } else { "none" });
    } else if auth
        .get("mode")
        .and_then(|v| v.as_str())
        .map(|mode| mode.eq_ignore_ascii_case("token"))
        .unwrap_or(false)
        && !has_token
    {
        auth["mode"] = serde_json::json!("none");
    }
    if string_missing_or_empty(gateway, "bind") {
        gateway["bind"] = serde_json::json!("lan");
    }
    let control_ui = ensure_json_object_at(gateway, "controlUi");
    if !control_ui
        .get("allowedOrigins")
        .map(|v| v.is_array())
        .unwrap_or(false)
    {
        control_ui["allowedOrigins"] =
            serde_json::json!(["http://localhost:18789", "http://127.0.0.1:18789"]);
    }
    ensure_json_object_at(gateway, "nodes");

    let agents = ensure_json_object_at(config, "agents");
    let defaults = ensure_json_object_at(agents, "defaults");
    let memory_search = ensure_json_object_at(defaults, "memorySearch");
    if !memory_search
        .get("sources")
        .map(|v| v.is_array())
        .unwrap_or(false)
    {
        memory_search["sources"] = serde_json::json!(["memory"]);
    }
    ensure_json_object_at(defaults, "model");
    ensure_json_object_at(defaults, "models");

    let models = ensure_json_object_at(config, "models");
    if string_missing_or_empty(models, "mode") {
        models["mode"] = serde_json::json!("merge");
    }
    ensure_json_object_at(models, "providers");

    let telegram_enabled = {
        let channels = ensure_json_object_at(config, "channels");
        let telegram = ensure_json_object_at(channels, "telegram");
        for key in ["dmPolicy", "groupPolicy"] {
            if let Some(policy) = telegram
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(normalize_openclaw_channel_policy)
            {
                telegram[key] = serde_json::json!(policy);
            }
        }
        if !telegram
            .get("allowFrom")
            .map(|v| v.is_array())
            .unwrap_or(false)
        {
            if let Some(value) = telegram
                .get("allowFrom")
                .and_then(|v| v.as_str())
                .map(str::to_string)
            {
                telegram["allowFrom"] = serde_json::json!([value]);
            } else {
                telegram["allowFrom"] = serde_json::json!(["*"]);
            }
        }
        let has_bot_token = telegram
            .get("botToken")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some();
        let has_missing_token_file = telegram
            .get("tokenFile")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|path| !Path::new(path).exists())
            .unwrap_or(false);
        if has_bot_token && has_missing_token_file {
            if let Some(telegram_obj) = telegram.as_object_mut() {
                telegram_obj.remove("tokenFile");
            }
        }
        let groups = ensure_json_object_at(telegram, "groups");
        let wildcard_group = ensure_json_object_at(groups, "*");
        if !wildcard_group
            .get("requireMention")
            .map(|v| v.is_boolean())
            .unwrap_or(false)
        {
            wildcard_group["requireMention"] = serde_json::json!(false);
        }
        let streaming = ensure_json_object_at(telegram, "streaming");
        if string_missing_or_empty(streaming, "mode") {
            streaming["mode"] = serde_json::json!("off");
        }
        telegram
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };
    let plugins = ensure_json_object_at(config, "plugins");
    let entries = ensure_json_object_at(plugins, "entries");
    if telegram_enabled {
        let telegram_plugin = ensure_json_object_at(entries, "telegram");
        telegram_plugin["enabled"] = serde_json::json!(true);
        ensure_json_object_at(telegram_plugin, "config");
    }
    let skills = ensure_json_object_at(config, "skills");
    ensure_json_object_at(skills, "entries");
    let talk = ensure_json_object_at(config, "talk");
    ensure_json_object_at(talk, "providers");
    let tools = ensure_json_object_at(config, "tools");
    ensure_json_object_at(tools, "exec");
    ensure_json_object_at(tools, "web");
    ensure_json_object_at(config, "messages");
    ensure_json_object_at(config, "wizard");
}

fn repair_openclaw_config_file(path: &Path) -> Result<bool, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut config = if path.exists() {
        let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str::<serde_json::Value>(&raw)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.display(), e))?
    } else {
        serde_json::json!({})
    };
    write_openclaw_config_file(path, &mut config)
}

fn repair_openclaw_gateway_configs() -> Result<String, String> {
    let default_path = home_dir().join(".openclaw").join("openclaw.json");
    let configured_path = resolve_openclaw_config_path();
    let mut paths = vec![default_path];
    if !paths.iter().any(|p| p.eq(&configured_path)) {
        paths.push(configured_path);
    }

    let mut messages = Vec::new();
    for path in paths {
        let changed = repair_openclaw_config_file(&path)?;
        let mut auth_message = String::new();
        if let Some(base) = path.parent() {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&raw) {
                    let synced = sync_all_openclaw_auth_profiles(Some(&config), base)?;
                    if !synced.is_empty() {
                        auth_message = format!("; auth synced: {}", synced.join(", "));
                    }
                }
            } else {
                let synced = sync_all_openclaw_auth_profiles(None, base)?;
                if !synced.is_empty() {
                    auth_message = format!("; auth synced: {}", synced.join(", "));
                }
            }
        }
        messages.push(format!(
            "{}: {}{}",
            path.display(),
            if changed { "fixed" } else { "already ok" },
            auth_message
        ));
    }
    let synced_all = sync_openclaw_auth_profiles_across_known_bases(None, None)?;
    if !synced_all.is_empty() {
        messages.push(format!(
            "auth profiles synced across known .openclaw folders: {}",
            synced_all.join(", ")
        ));
    }
    Ok(format!("Gateway config repaired.\n{}", messages.join("\n")))
}

#[tauri::command]
fn check_tools() -> HashMap<String, bool> {
    // Use version-based detection for reliability
    let tools = [
        ("node", "node", "--version"),
        ("openclaw", "openclaw", "--version"),
        ("claude-code", "claude", "--version"),
        ("9router", "9router", "--version"),
        ("n8n", "n8n", "--version"),
        ("ngrok", "ngrok", "version"),
        ("git", "git", "--version"),
        ("python", "python", "--version"),
    ];
    tools
        .iter()
        .map(|(name, cmd, arg)| {
            let installed = run_cmd(cmd, &[arg])
                .ok()
                .and_then(|out| extract_version_number(&out))
                .is_some()
                || is_tool_installed(name);
            (name.to_string(), installed)
        })
        .collect()
}

#[tauri::command]
fn check_versions() -> HashMap<String, String> {
    let script = r#"
$tools = @(
    @('node','node --version'),
    @('openclaw','openclaw --version'),
    @('claude-code','claude --version'),
    @('9router','9router --version'),
    @('n8n','n8n --version'),
    @('ngrok','ngrok version'),
    @('git','git --version'),
    @('python','python --version')
)
$result = @{}
foreach ($t in $tools) {
    $name = $t[0]; $cmd = $t[1]
    try {
        $out = Invoke-Expression $cmd 2>&1 | Out-String
        $result[$name] = $out.Trim()
    } catch {
        $result[$name] = 'Not Installed'
    }
}
$result | ConvertTo-Json -Compress
"#;
    let output = run_ps(script).unwrap_or_default();
    let mut map: HashMap<String, String> = HashMap::new();
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&output) {
        if let Some(obj) = val.as_object() {
            for (k, v) in obj {
                let raw = v.as_str().unwrap_or("Not Installed").to_string();
                map.insert(k.clone(), clean_version_output(raw));
            }
        }
    }
    if map.is_empty() {
        let specs = [
            ("node", "node", vec!["--version"]),
            ("openclaw", "openclaw", vec!["--version"]),
            ("claude-code", "claude", vec!["--version"]),
            ("9router", "9router", vec!["--version"]),
            ("n8n", "n8n", vec!["--version"]),
            ("ngrok", "ngrok", vec!["version"]),
            ("git", "git", vec!["--version"]),
            ("python", "python", vec!["--version"]),
        ];
        specs
            .iter()
            .map(|(name, cmd, args)| {
                let v = if is_tool_installed(name) {
                    clean_version_output(run_cmd(cmd, args).unwrap_or_default())
                } else {
                    "Not Installed".into()
                };
                (name.to_string(), v)
            })
            .collect()
    } else {
        for name in [
            "node",
            "openclaw",
            "claude-code",
            "9router",
            "n8n",
            "ngrok",
            "git",
            "python",
        ] {
            let missing = map
                .get(name)
                .map(|v| v.trim().is_empty() || v == "Not Installed")
                .unwrap_or(true);
            if missing {
                if let Some(version) = tool_version(name) {
                    map.insert(name.to_string(), version);
                }
            }
        }
        map
    }
}

#[tauri::command]
fn app_statuses() -> HashMap<String, String> {
    let versions = check_versions();
    let is_installed = |name: &str| -> bool {
        versions
            .get(name)
            .map(|v| v != "Not Installed" && !v.is_empty())
            .unwrap_or(false)
    };
    let mut m = HashMap::new();

    let node_installed = is_installed("node");
    m.insert(
        "node".into(),
        if !node_installed {
            "Not Installed"
        } else {
            "Installed"
        }
        .into(),
    );

    let openclaw_installed = is_installed("openclaw");
    let openclaw_running = openclaw_installed
        && gateway_status_output()
            .map(|s| gateway_ready_from_status(&s))
            .unwrap_or(false);
    m.insert(
        "gateway".into(),
        if !openclaw_installed {
            "Not Installed"
        } else if openclaw_running {
            "Running"
        } else {
            "Stopped"
        }
        .into(),
    );
    m.insert(
        "openclaw".into(),
        if !openclaw_installed {
            "Not Installed"
        } else {
            "Installed"
        }
        .into(),
    );

    let detections = [
        ("n8n", "$p=Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { ($_.Name -match 'node|n8n') -and $_.CommandLine -match 'n8n' -and $_.CommandLine -notmatch 'openclaw' } | Select-Object -First 1; if($p){'true'}else{'false'}"),
        ("ngrok", "$p=Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.Name -ieq 'ngrok.exe' -or $_.CommandLine -match 'ngrok(\\.exe)?' } | Select-Object -First 1; if($p){'true'}else{'false'}"),
        ("claude-code", "$p=Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.Name -match 'claude' -or $_.CommandLine -match 'claude' } | Select-Object -First 1; if($p){'true'}else{'false'}"),
        ("9router", "$p=Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.Name -match '9router|node' -and $_.CommandLine -match '9router' } | Select-Object -First 1; if($p){'true'}else{'false'}"),
    ];

    for (key, script) in detections {
        let installed = is_installed(key);
        let running = installed
            && run_ps(script)
                .map(|s| s.to_lowercase().contains("true"))
                .unwrap_or(false);
        m.insert(
            key.into(),
            if !installed {
                "Not Installed"
            } else if running {
                "Running"
            } else {
                "Stopped"
            }
            .into(),
        );
    }
    m.insert(
        "git".into(),
        if is_installed("git") {
            "Installed"
        } else {
            "Not Installed"
        }
        .into(),
    );
    m.insert(
        "python".into(),
        if is_installed("python") {
            "Installed"
        } else {
            "Not Installed"
        }
        .into(),
    );
    m
}

#[tauri::command]
fn setup_openclaw_files(
    openclaw_path: String,
    workspace_path: String,
    openai_api_key: Option<String>,
    api_key_provider: Option<String>,
) -> Result<String, String> {
    let base = openclaw_base_dir(&openclaw_path);
    let workspace_root = PathBuf::from(&workspace_path);
    let ws = if workspace_root
        .file_name()
        .map(|n| n.eq_ignore_ascii_case("workspace"))
        .unwrap_or(false)
    {
        workspace_root
    } else {
        workspace_root.join("workspace")
    };
    let folders = vec![
        "",
        "agents",
        "agents/main",
        "agents/main/agent",
        "browser",
        "canvas",
        "completions",
        "credentials",
        "delivery-queue",
        "devices",
        "flows",
        "identity",
        "logs",
        "media",
        "memory",
        "npm",
        "plugins",
        "session-delivery-queue",
        "subagents",
        "tasks",
        "telegram",
    ];
    for folder in &folders {
        fs::create_dir_all(base.join(folder))
            .map_err(|e| format!("Failed to create folder: {}", e))?;
    }
    fs::create_dir_all(
        home_dir()
            .join(".openclaw")
            .join("agents")
            .join("main")
            .join("agent"),
    )
    .map_err(|e| format!("Failed to create default auth folder: {}", e))?;
    fs::create_dir_all(ws.join("memory"))
        .map_err(|e| format!("Failed to create workspace: {}", e))?;
    let ws_str = path_as_config_string(&ws);
    let config_path = base.join("openclaw.json");
    let mut defaults = openclaw_default_config(Some(&base));
    defaults["agents"]["defaults"]["workspace"] = serde_json::json!(ws_str);
    let mut config = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read openclaw.json: {}", e))?;
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if !config.is_object() {
        config = serde_json::json!({});
    }
    merge_missing_json(&mut config, &defaults);
    if !config["agents"].is_object() {
        config["agents"] = serde_json::json!({});
    }
    if !config["agents"]["defaults"].is_object() {
        config["agents"]["defaults"] = serde_json::json!({});
    }
    config["agents"]["defaults"]["workspace"] = serde_json::json!(ws_str);
    let api_key = openai_api_key.unwrap_or_default().trim().to_string();
    let provider_id = api_key_provider
        .as_deref()
        .and_then(provider_id_for_key)
        .unwrap_or("openai");
    if !api_key.is_empty() && provider_id != "unknown" {
        config["models"]["providers"][provider_id]["apiKey"] = serde_json::json!(api_key);
    }
    write_openclaw_config_file(&config_path, &mut config)
        .map_err(|e| format!("Failed to write openclaw.json: {}", e))?;
    write_if_missing(
        &base.join("gateway.cmd"),
        "@echo off\nopenclaw gateway start\n",
    )?;
    write_if_missing(&ws.join("AGENTS.md"), "# AGENTS.md - Your Workspace\n\nThis folder is home. Treat it that way.\n\n## Memory\n\n- **Daily notes:** `memory/YYYY-MM-DD.md`\n- **Long-term:** `MEMORY.md`\n")?;
    write_if_missing(&ws.join("SOUL.md"), "# SOUL.md - Who You Are\n\n## Core Truths\n\n**Be genuinely helpful, not performatively helpful.**\n**Have opinions.**\n**Be resourceful before asking.**\n")?;
    write_if_missing(
        &ws.join("USER.md"),
        "# USER.md - About Your Human\n\n- **Name:**\n- **Timezone:**\n- **Notes:**\n",
    )?;
    write_if_missing(
        &ws.join("MEMORY.md"),
        "# MEMORY.md\n\n_Your long-term memory. Update as you learn._\n",
    )?;
    write_if_missing(
        &ws.join("TOOLS.md"),
        "# TOOLS.md - Local Notes\n\n_Add environment-specific notes here._\n",
    )?;
    write_if_missing(
        &ws.join("HEARTBEAT.md"),
        "# HEARTBEAT.md\n\n# Keep this file empty to skip heartbeat API calls.\n",
    )?;
    persist_openclaw_setup_paths(&base, &ws)?;
    if !api_key.is_empty() && provider_id != "unknown" {
        persist_app_api_key(provider_id, &api_key)?;
        let _ = sync_openclaw_auth_profiles_across_known_bases(Some(&config), Some(&base))?;
    }
    Ok(format!(
        "Config: {}\nWorkspace: {}",
        base.display(),
        ws.display()
    ))
}

fn ensure_installed(tool: &str, display: &str) -> Result<(), String> {
    if tool_version(tool).is_some() || is_tool_installed(tool) {
        Ok(())
    } else {
        Err(format!("{} is not installed", display))
    }
}

fn generic_install_script(tool: &str, version: &str) -> Result<String, String> {
    validate_tool_version(version)?;
    Ok(match tool {
        "node" => winget_node_install_script(version),
        "openclaw" => npm_global_package_script("openclaw", "openclaw", version, false, false),
        "claude-code" | "claude" => npm_global_package_script("@anthropic-ai/claude-code", "claude", version, false, true),
        "9router" => npm_global_package_script("9router", "9router", version, false, false),
        "n8n" => npm_global_package_script("n8n", "n8n", version, false, false),
        "ngrok" => {
            if version.is_empty() {
                r#"
$dest = (Join-Path $env:LOCALAPPDATA 'Ngrok Tunnel')
if (-not (Test-Path $dest)) { New-Item -ItemType Directory -Path $dest -Force | Out-Null }
$zip = Join-Path $env:TEMP 'ngrok.zip'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri 'https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-windows-amd64.zip' -OutFile $zip
Expand-Archive -Path $zip -DestinationPath $dest -Force
Remove-Item $zip -Force -EA SilentlyContinue
$env:Path = $env:Path + ';' + $dest
ngrok version
"#.to_string()
            } else {
                format!("winget install --id ngrok.ngrok --exact --version {} --accept-package-agreements --accept-source-agreements --disable-interactivity; if($LASTEXITCODE -ne 0){{exit $LASTEXITCODE}}; ngrok version", version)
            }
        }
        "git" => make_download_script(
            "https://github.com/git-for-windows/git/releases/download/v2.49.0.windows.1/Git-2.49.0-64-bit.exe",
            "git-installer.exe",
            "$p=Start-Process -FilePath $dest -ArgumentList '/VERYSILENT /NORESTART /NOCANCEL /SP- /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS /COMPONENTS=\"icons,ext\\reg\\shellhere,assoc,assoc_sh\"' -Wait -PassThru; if($p.ExitCode -ne 0){throw \"Installer exit code $($p.ExitCode)\"}",
            "Test-Path 'C:\\Program Files\\Git\\cmd\\git.exe'",
            "Git installed successfully!"
        ),
        "python" => make_download_script(
            "https://www.python.org/ftp/python/3.13.3/python-3.13.3-amd64.exe",
            "python-installer.exe",
            "$p=Start-Process -FilePath $dest -ArgumentList '/quiet InstallAllUsers=1 PrependPath=1 Include_test=0' -Wait -PassThru -WindowStyle Hidden; if($p.ExitCode -ne 0){throw \"Installer exit code $($p.ExitCode)\"}",
            "(Get-Command python -All -EA SilentlyContinue | Where-Object { $_.Source -and $_.Source -notmatch 'WindowsApps' } | Select-Object -First 1) -or (Get-Command py -EA SilentlyContinue)",
            "Python installed successfully!"
        ),
        _ => return Err(format!("Unknown tool: {}", tool)),
    })
}

#[tauri::command]
fn run_action(action: String) -> Result<String, String> {
    let a = action.replace('_', "-");
    match a.as_str() {
        "gateway-run" => {
            ensure_installed("openclaw", "OpenClaw")?;
            let env = configured_openclaw_env();
            spawn_cmd_owned_env("openclaw", &["gateway", "run"], &env)
                .map(|pid| format!("started openclaw gateway run pid {}", pid))
        }
        "gateway-stop" => {
            ensure_installed("openclaw", "OpenClaw")?;
            stop_openclaw_gateway()
        }
        "gateway-restart" => {
            ensure_installed("openclaw", "OpenClaw")?;
            restart_openclaw_gateway()
        }
        "webui-run" => {
            ensure_installed("openclaw", "OpenClaw")?;
            let env = configured_openclaw_env();
            let gateway_running = gateway_status_output()
                .map(|status| gateway_ready_from_status(&status))
                .unwrap_or(false);
            if !gateway_running {
                let _ = spawn_cmd_owned_env("openclaw", &["gateway", "run"], &env)?;
                thread::sleep(Duration::from_secs(2));
            }
            open_openclaw_dashboard(&env)
        }
        "doctor" => {
            let repaired = repair_openclaw_gateway_configs()?;
            let env = configured_openclaw_env();
            run_ps_async_with_env("openclaw doctor --fix".to_string(), &env)
                .map(|_| format!("{}\ndoctor started", repaired))
        }
        "doctor-fix" => {
            let repaired = repair_openclaw_gateway_configs()?;
            let env = configured_openclaw_env();
            run_ps_with_env("openclaw doctor --fix", &env)
                .map(|out| format!("{}\n{}", repaired, out))
        }
        "fix-gateway" | "fix-openclaw-gateway" => repair_openclaw_gateway_configs(),
        "open-config" | "open_config" => run_ps_async("openclaw config".to_string()).map(|_| "config opened".into()),
        "n8n-run" => {
            ensure_installed("n8n", "n8n")?;
            spawn_cmd("n8n", &["start", "--open"], &[]).map(|pid| format!("started n8n pid {}", pid))
        }
        "n8n-stop" => {
            let cfg: AppConfig = load_settings();
            let port = cfg.ngrok.port_val();
            let port_str = if port.is_empty() { "5678".to_string() } else { port };
            validate_port(&port_str)?;
            run_ps_async(format!("taskkill /F /IM n8n.exe 2>$null; Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object {{ $_.CommandLine -match 'n8n' -and $_.CommandLine -notmatch 'openclaw' }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -EA SilentlyContinue }}; $conn=Get-NetTCPConnection -LocalPort {} -State Listen -EA SilentlyContinue; if($conn){{$conn | Select-Object -ExpandProperty OwningProcess | Select-Object -Unique | ForEach-Object {{ Stop-Process -Id $_ -Force -EA SilentlyContinue }}}}; 'stopped n8n'", port_str))
        },
        "ngrok-run" => {
            ensure_installed("ngrok", "ngrok")?;
            spawn_cmd("ngrok", &["http", "5678"], &[]).map(|pid| format!("started ngrok pid {}", pid))
        }
        "ngrok-stop" => run_ps_async("taskkill /F /IM ngrok.exe 2>$null; Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.Name -ieq 'ngrok.exe' -or $_.CommandLine -match 'ngrok(\\.exe)?' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -EA SilentlyContinue }; 'stopped ngrok'".to_string()),
        "n8n-ngrok-run" => {
            ensure_installed("n8n", "n8n")?;
            ensure_installed("ngrok", "ngrok")?;
            let cfg: AppConfig = load_settings();
            let port = cfg.ngrok.port_val();
            let port_str = if port.is_empty() { "5678".to_string() } else { port };
            let domain = cfg.ngrok.domain.clone();
            let token = cfg.ngrok.authtoken.clone();
            validate_port(&port_str)?;
            validate_ngrok_domain(&domain)?;
            validate_ngrok_token(&token)?;
            let token_arg = ps_single_quote(&token);
            let arg_list = quote_ps_arg(&format!("http {} --url={}", port_str, domain));
            let url = quote_ps_arg(&format!("https://{}", domain));
            let script = format!("taskkill /F /IM ngrok.exe 2>$null; ngrok config add-authtoken {}; Start-Job -ScriptBlock {{ $env:N8N_OPEN_BROWSER='false'; n8n start }} | Out-Null; Start-Sleep -Seconds 8; Start-Process 'ngrok' -ArgumentList {}; Start-Sleep -Seconds 3; Start-Process {}; 'started n8n + ngrok'", token_arg, arg_list, url);
            run_ps_async(script)
        }
        "n8n-ngrok-stop" => {
            let cfg: AppConfig = load_settings();
            let port = cfg.ngrok.port_val();
            let port_str = if port.is_empty() { "5678".to_string() } else { port };
            validate_port(&port_str)?;
            run_ps_async(format!("taskkill /F /IM ngrok.exe 2>$null; Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object {{ $_.CommandLine -match 'n8n' -and $_.CommandLine -notmatch 'openclaw' }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -EA SilentlyContinue }}; $conn=Get-NetTCPConnection -LocalPort {} -State Listen -EA SilentlyContinue; if($conn){{$conn | Select-Object -ExpandProperty OwningProcess | Select-Object -Unique | ForEach-Object {{ Stop-Process -Id $_ -Force -EA SilentlyContinue }}}}; 'stopped n8n + ngrok'", port_str))
        },
        "claude-run" | "claude-code" | "claude-code-run" => {
            ensure_installed("claude-code", "Claude Code")?;
            spawn_visible_terminal("claude")
        }
        "claude-stop" | "claude-code-stop" => run_ps_async("Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.Name -match 'claude' -or $_.CommandLine -match 'claude' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }; 'stopped claude'".to_string()),
        "9router-run" => {
            ensure_installed("9router", "9router")?;
            spawn_visible_terminal("9router")
        }
        "9router-stop" => run_ps_async("Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.Name -match '9router|node' -and $_.CommandLine -match '9router' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }; 'stopped 9router'".to_string()),
        "install-git" => {
            let cmd = generic_install_script("git", "")?;
            run_install_and_verify("git", "", cmd)
        }
        "install-python" => {
            let cmd = generic_install_script("python", "")?;
            run_install_and_verify("python", "", cmd)
        }
        "ngrok-view-account" => {
            let cfg: AppConfig = load_settings();
            Ok(format!("Authtoken: {}\nDomain: {}\nPort: {}",
                if cfg.ngrok.authtoken.is_empty() { "(not set)".into() } else { cfg.ngrok.authtoken },
                if cfg.ngrok.domain.is_empty() { "(not set)".into() } else { cfg.ngrok.domain },
                if cfg.ngrok.port.is_empty() { "(not set)".into() } else { cfg.ngrok.port }
            ))
        }
        "reset-ngrok-mapping" => {
            let _ = run_ps("taskkill /F /IM ngrok.exe 2>$null");
            let _ = run_ps("Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.CommandLine -match 'n8n' -and $_.CommandLine -notmatch 'openclaw' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -EA SilentlyContinue }");
            std::thread::sleep(std::time::Duration::from_secs(2));
            let _ = run_ps("Remove-Item -Force $env:LOCALAPPDATA\\ngrok\\ngrok.yml -EA SilentlyContinue");
            let _ = run_ps("Remove-Item -Force $env:USERPROFILE\\.ngrok2\\ngrok.yml -EA SilentlyContinue");
            let mut cfg: AppConfig = load_settings();
            cfg.ngrok = NgrokCfg::default();
            let _ = save_settings(cfg);
            Ok("reset mapping done".into())
        }
        _ if a.starts_with("map-authtoken-") => {
            let token = if action.starts_with("map-authtoken-") { &action[14..] } else { &a[14..] };
            validate_ngrok_token(token)?;
            run_ps(&format!("ngrok config add-authtoken {}", ps_single_quote(token))).map(|out| format!("authtoken mapped: {}", out))
        }
        "install-node" => {
            let cmd = generic_install_script("node", "")?;
            run_install_and_verify("node", "", cmd)
        }
        "install-openclaw" => {
            let cmd = generic_install_script("openclaw", "")?;
            run_install_and_verify("openclaw", "", cmd)
        }
        "install-claude" | "install-claude-code" => {
            let cmd = generic_install_script("claude-code", "")?;
            run_install_and_verify("claude-code", "", cmd)
        }
        "install-9router" => {
            let cmd = generic_install_script("9router", "")?;
            run_install_and_verify("9router", "", cmd)
        }
        "install-n8n" => {
            let cmd = generic_install_script("n8n", "")?;
            run_install_and_verify("n8n", "", cmd)
        }
        _ if a == "install-ngrok" || a.starts_with("install-ngrok-") => {
            let version = a.strip_prefix("install-ngrok-").unwrap_or("");
            let cmd = generic_install_script("ngrok", version)?;
            run_install_and_verify("ngrok", version, cmd)
        }
        "update-openclaw" => { ensure_installed("openclaw", "OpenClaw")?; run_update_and_verify("openclaw", &npm_global_package_script("openclaw", "openclaw", "", true, false)) },
        "update-n8n" => { ensure_installed("n8n", "n8n")?; run_update_and_verify("n8n", &npm_global_package_script("n8n", "n8n", "", true, false)) },
        "update-9router" => { ensure_installed("9router", "9router")?; run_update_and_verify("9router", &npm_global_package_script("9router", "9router", "", true, false)) },
        "update-claude" | "update-claude-code" => { ensure_installed("claude-code", "Claude Code")?; run_update_and_verify("claude-code", &npm_global_package_script("@anthropic-ai/claude-code", "claude", "", true, false)) },
        "update-node" => { ensure_installed("node", "Node.js")?; run_update_and_verify("node", &winget_node_install_script("")) },
        "update-ngrok" => {
            ensure_installed("ngrok", "ngrok")?;
            let script = r#"
$dest = (Join-Path $env:LOCALAPPDATA 'Ngrok Tunnel')
if (-not (Test-Path $dest)) { New-Item -ItemType Directory -Path $dest -Force | Out-Null }
$zip = Join-Path $env:TEMP 'ngrok.zip'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri 'https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-windows-amd64.zip' -OutFile $zip
Expand-Archive -Path $zip -DestinationPath $dest -Force
Remove-Item $zip -Force -EA SilentlyContinue
if (Test-Path (Join-Path $dest 'ngrok.exe')) { 'updated' } else { throw 'Update failed' }
"#;
            run_update_and_verify("ngrok", script)
        }
        "uninstall-node" => {
            let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
Get-Process node,npm,npx,corepack -EA SilentlyContinue | Stop-Process -Force

if (Get-Command winget -EA SilentlyContinue) {
    foreach ($id in @('OpenJS.NodeJS.LTS','OpenJS.NodeJS','OpenJS.NodeJS.Current')) {
        winget uninstall --id $id --exact --silent --accept-source-agreements --disable-interactivity | Out-Null
    }
    foreach ($name in @('Node.js','Node.js LTS')) {
        winget uninstall --name $name --silent --accept-source-agreements --disable-interactivity | Out-Null
    }
}

$roots = @(
    'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
    'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*'
)
$apps = foreach ($root in $roots) { Get-ItemProperty $root -EA SilentlyContinue }
$apps = $apps | Where-Object {
    $_.DisplayName -match '^Node\.js' -or
    (($_.DisplayName -match 'Node') -and ($_.Publisher -match 'OpenJS|Node'))
}
foreach ($app in $apps | Sort-Object DisplayName -Unique) {
    $uninstall = $app.QuietUninstallString
    if (-not $uninstall) { $uninstall = $app.UninstallString }
    if ($uninstall -match '\{[0-9A-Fa-f-]{36}\}') {
        Start-Process msiexec.exe -ArgumentList "/x $($Matches[0]) /qn /norestart" -WindowStyle Hidden -Wait
    } elseif ($uninstall) {
        Start-Process cmd.exe -ArgumentList @('/c', $uninstall + ' /quiet /norestart') -WindowStyle Hidden -Wait
    }
}

$nodeDirs = @("$env:ProgramFiles\nodejs", "${env:ProgramFiles(x86)}\nodejs") | Where-Object { $_ }
foreach ($dir in $nodeDirs) {
    if (Test-Path $dir) { Remove-Item -LiteralPath $dir -Recurse -Force -EA SilentlyContinue }
}

foreach ($scope in @('User','Machine')) {
    $path = [Environment]::GetEnvironmentVariable('Path', $scope)
    if ($path) {
        $clean = ($path -split ';' | Where-Object {
            $entry = $_.Trim().TrimEnd('\')
            $entry -and ($nodeDirs -notcontains $entry)
        }) -join ';'
        [Environment]::SetEnvironmentVariable('Path', $clean, $scope)
    }
}
"#;
            let uninstall_result = run_ps(script);
            thread::sleep(std::time::Duration::from_secs(1));
            if is_tool_installed("node") {
                match uninstall_result {
                    Ok(out) if !out.trim().is_empty() => Err(format!("Uninstall failed - Node.js is still installed: {}", out)),
                    Err(e) => Err(format!("Uninstall failed - Node.js is still installed: {}", e)),
                    _ => Err("Uninstall failed - Node.js is still installed".into()),
                }
            } else {
                Ok("Node.js uninstalled".into())
            }
        }
        "uninstall-openclaw" => {
            let _ = run_ps("openclaw gateway stop 2>&1");
            let _ = run_ps("Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.CommandLine -match 'openclaw' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -EA SilentlyContinue }");
            let _ = run_ps("npm uninstall -g openclaw 2>&1");
            if is_tool_installed("openclaw") { Err("Uninstall failed  - OpenClaw is still installed".into()) } else { Ok("OpenClaw uninstalled".into()) }
        }
        "uninstall-claude" | "uninstall-claude-code" => {
            let _ = run_ps("Get-Process claude* -EA SilentlyContinue | Stop-Process -Force");
            let _ = run_ps("npm uninstall -g @anthropic-ai/claude-code 2>&1");
            if is_tool_installed("claude-code") { Err("Uninstall failed  - Claude Code is still installed".into()) } else { Ok("Claude Code uninstalled".into()) }
        }
        "uninstall-9router" => {
            let _ = run_ps("Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { $_.CommandLine -match '9router' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -EA SilentlyContinue }");
            let _ = run_ps("npm uninstall -g 9router 2>&1");
            if is_tool_installed("9router") { Err("Uninstall failed  - 9router is still installed".into()) } else { Ok("9router uninstalled".into()) }
        }
        "uninstall-n8n" => {
            let _ = run_ps("Get-CimInstance Win32_Process -EA SilentlyContinue | Where-Object { ($_.Name -match 'node|n8n') -and $_.CommandLine -match 'n8n' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -EA SilentlyContinue }");
            let _ = run_ps("$conn=Get-NetTCPConnection -LocalPort 5678 -State Listen -EA SilentlyContinue; if($conn){$conn | Select-Object -ExpandProperty OwningProcess | Select-Object -Unique | ForEach-Object { Stop-Process -Id $_ -Force -EA SilentlyContinue }}");
            let _ = run_ps("npm uninstall -g n8n 2>&1");
            if is_tool_installed("n8n") { Err("Uninstall failed  - n8n is still installed".into()) } else { Ok("n8n uninstalled".into()) }
        }
        "uninstall-ngrok" => {
            let _ = run_ps("Get-Process ngrok -EA SilentlyContinue | Stop-Process -Force");
            let _ = run_ps(r#"Remove-Item (Join-Path $env:LOCALAPPDATA 'Ngrok Tunnel') -Recurse -Force -EA SilentlyContinue"#);
            let _ = run_ps("winget uninstall --id ngrok.ngrok --exact --silent --accept-source-agreements --disable-interactivity 2>&1");
            let ngrok_dir = format!("{}\\ngrok", std::env::var("LOCALAPPDATA").unwrap_or_default());
            let _ = std::fs::remove_dir_all(&ngrok_dir);
            let _ = run_ps(r#"$p=[Environment]::GetEnvironmentVariable('Path','User'); $p=($p -split ';' | Where-Object { $_ -ne (Join-Path $env:LOCALAPPDATA 'Ngrok Tunnel') }) -join ';'; [Environment]::SetEnvironmentVariable('Path',$p,'User')"#);
            if is_tool_installed("ngrok") { Err("Uninstall failed - ngrok is still installed".into()) } else { Ok("ngrok uninstalled".into()) }
        }
        "update-git" => { ensure_installed("git", "Git")?; run_update_and_verify("git", "winget upgrade --id Git.Git --exact --accept-package-agreements --accept-source-agreements --disable-interactivity; if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}; git --version") },
        "update-python" => { ensure_installed("python", "Python")?; run_update_and_verify("python", "winget upgrade --id Python.Python.3.13 --exact --accept-package-agreements --accept-source-agreements --disable-interactivity; if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}; $cmd=Get-Command python -All -EA SilentlyContinue | Where-Object { $_.Source -and $_.Source -notmatch 'WindowsApps' } | Select-Object -First 1; if($cmd){ & $cmd.Source --version } elseif(Get-Command py -EA SilentlyContinue){ py -3 --version } else { 'Python updated' }") },
        "uninstall-git" => {
            let _ = run_ps("Get-Process git* -EA SilentlyContinue | Stop-Process -Force");
            let _ = run_ps("if(Test-Path 'C:\\Program Files\\Git\\unins000.exe'){Start-Process 'C:\\Program Files\\Git\\unins000.exe' -ArgumentList '/VERYSILENT /NORESTART' -WindowStyle Hidden -Wait}");
            let _ = run_ps("winget uninstall --id Git.Git --exact --silent --accept-source-agreements --disable-interactivity 2>&1");
            if is_tool_installed("git") { Err("Uninstall failed — Git is still installed".into()) } else { Ok("Git uninstalled".into()) }
        }
        "uninstall-python" => {
            let _ = run_ps("Get-Process python* -EA SilentlyContinue | Stop-Process -Force");
            let _ = run_ps("winget uninstall --id Python.Python.3.13 --exact --silent --accept-source-agreements --disable-interactivity 2>&1; winget uninstall --id Python.Python.3.12 --exact --silent --accept-source-agreements --disable-interactivity 2>&1; winget uninstall --id Python.Python.3.11 --exact --silent --accept-source-agreements --disable-interactivity 2>&1");
            if is_tool_installed("python") { Err("Uninstall failed — Python is still installed".into()) } else { Ok("Python uninstalled".into()) }
        }
        _ if a.starts_with("install-") => {
            let rest = &a[8..];
            let known_tools = ["claude-code", "9router", "openclaw", "ngrok", "node", "n8n", "git", "python"];
            let mut tool_name = "";
            let mut version = "";
            for t in &known_tools {
                if rest.starts_with(t) {
                    tool_name = t;
                    let remainder = &rest[t.len()..];
                    version = remainder.strip_prefix('-').unwrap_or("");
                    break;
                }
            }
            if tool_name.is_empty() { return Err(format!("Unknown tool: {}", rest)); }
            let cmd = generic_install_script(tool_name, version)?;
            run_install_and_verify(tool_name, version, cmd)
        }
        _ => Err(format!("Unknown: {}", action)),
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct AppConfig {
    #[serde(default)]
    openclaw: OpenclawCfg,
    #[serde(default)]
    telegram: TgCfg,
    #[serde(default, alias = "google_console")]
    google: GoogleCfg,
    #[serde(default)]
    ngrok: NgrokCfg,
    #[serde(default)]
    n8n: N8nCfg,
}
#[derive(Serialize, Deserialize, Clone, Default)]
struct OpenclawCfg {
    #[serde(default)]
    model: serde_json::Value,
    #[serde(default, rename = "apiKeys")]
    api_keys: HashMap<String, String>,
    #[serde(default, rename = "customProviders")]
    custom_providers: Vec<serde_json::Value>,
    #[serde(default, alias = "installDir")]
    install_dir: String,
    #[serde(default, alias = "workspaceDir")]
    workspace_dir: String,
}
#[derive(Serialize, Deserialize, Clone, Default)]
struct TgCfg {
    #[serde(default, alias = "chatId")]
    group_id: String,
    #[serde(default, alias = "botId")]
    bot_id: String,
    #[serde(default, alias = "botToken", alias = "apiKey")]
    api_key: String,
    #[serde(default)]
    bots: Vec<TgBot>,
}
#[derive(Serialize, Deserialize, Clone, Default)]
struct TgBot {
    #[serde(default, rename = "botToken")]
    bot_token: String,
    #[serde(default, rename = "chatId")]
    chat_id: String,
}
#[derive(Serialize, Deserialize, Clone, Default)]
struct GoogleCfg {
    #[serde(default, rename = "clientId", alias = "customer_id")]
    client_id: String,
    #[serde(default, rename = "clientSecret", alias = "customer_secret_code")]
    client_secret: String,
    #[serde(default, rename = "apiKey")]
    api_key: String,
}
#[derive(Serialize, Deserialize, Clone, Default)]
struct NgrokCfg {
    #[serde(default)]
    authtoken: String,
    #[serde(default)]
    domain: String,
    #[serde(default)]
    port: String,
}
impl NgrokCfg {
    fn port_val(&self) -> String {
        self.port.clone()
    }
}
#[derive(Serialize, Deserialize, Clone, Default)]
struct N8nCfg {
    #[serde(default, alias = "apiKey")]
    api_key: String,
    #[serde(default, alias = "url", alias = "baseUrl")]
    base_url: String,
}

#[tauri::command]
fn load_settings() -> AppConfig {
    let p = config_file();
    if p.exists() {
        fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

#[tauri::command]
fn save_settings(mut cfg: AppConfig) -> Result<String, String> {
    let existing = load_settings();
    if cfg.openclaw.install_dir.trim().is_empty()
        && !existing.openclaw.install_dir.trim().is_empty()
    {
        cfg.openclaw.install_dir = existing.openclaw.install_dir;
    }
    if cfg.openclaw.workspace_dir.trim().is_empty()
        && !existing.openclaw.workspace_dir.trim().is_empty()
    {
        cfg.openclaw.workspace_dir = existing.openclaw.workspace_dir;
    }
    fs::create_dir_all(config_dir()).map_err(|e| e.to_string())?;
    fs::write(
        config_file(),
        serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    sync_to_openclaw(&cfg)?;
    Ok("saved".into())
}

fn persist_app_api_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    let key = api_key.trim();
    if provider_id.trim().is_empty() || key.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(config_dir()).map_err(|e| e.to_string())?;
    let mut cfg = fs::read_to_string(config_file())
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if !cfg.is_object() {
        cfg = serde_json::json!({});
    }
    cfg["openclaw"]["apiKeys"][provider_id] = serde_json::json!(key);
    fs::write(
        config_file(),
        serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn provider_id_for_key(key: &str) -> Option<&'static str> {
    let normalized = key.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "openai_api_key" | "openai" => Some("openai"),
        "anthropic_api_key" | "anthropic" | "claude" => Some("anthropic"),
        "google_api_key" | "google" | "gemini" => Some("google"),
        "groq_api_key" | "groq" => Some("groq"),
        "openrouter_api_key" | "openrouter" | "openrouter auto" => Some("openrouter"),
        "9router_api_key" | "9router" | "9router custom" => Some("9router"),
        "xai_api_key" | "xai" | "grok" => Some("xai"),
        "unknown" | "unknow" => Some("unknown"),
        _ if normalized.contains("gpt") || normalized.contains("openai") => Some("openai"),
        _ if normalized.contains("claude") || normalized.contains("anthropic") => Some("anthropic"),
        _ if normalized.contains("gemini") || normalized.contains("google") => Some("google"),
        _ if normalized.contains("groq") => Some("groq"),
        _ if normalized.contains("openrouter") => Some("openrouter"),
        _ if normalized.contains("grok") || normalized.contains("xai") => Some("xai"),
        _ => None,
    }
}

fn provider_id_from_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn numeric_json_value(value: &serde_json::Value) -> Option<serde_json::Value> {
    if value.is_number() {
        return Some(value.clone());
    }
    let raw = value.as_str()?.trim();
    if raw.is_empty() {
        return None;
    }
    raw.parse::<u64>()
        .ok()
        .map(|n| serde_json::Value::Number(serde_json::Number::from(n)))
}

fn string_json_value(value: &serde_json::Value) -> Option<serde_json::Value> {
    value
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| serde_json::json!(v))
        .or_else(|| {
            if value.is_number() || value.is_boolean() {
                Some(value.clone())
            } else {
                None
            }
        })
}

fn bool_json_value(value: &serde_json::Value) -> Option<serde_json::Value> {
    if let Some(value) = value.as_bool() {
        return Some(serde_json::json!(value));
    }
    match value.as_str()?.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Some(serde_json::json!(true)),
        "false" | "no" | "0" | "off" => Some(serde_json::json!(false)),
        _ => None,
    }
}

fn provider_value<'a>(
    provider: &'a serde_json::Value,
    keys: &[&str],
) -> Option<&'a serde_json::Value> {
    keys.iter().find_map(|key| provider.get(*key))
}

fn provider_string_value<'a>(provider: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    provider_value(provider, keys)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

fn provider_auth_id(provider_key: &str, provider: &serde_json::Value) -> Option<String> {
    if let Some(id) = provider_id_for_key(provider_key) {
        return Some(id.to_string());
    }
    if let Some(name) = provider_string_value(provider, &["name", "label", "provider"]) {
        if let Some(id) = provider_id_for_key(name) {
            return Some(id.to_string());
        }
    }

    let base_url = provider_string_value(provider, &["baseUrl", "base_url"]);
    let base_url_lower = base_url.unwrap_or("").to_ascii_lowercase();
    let compatibility = provider_string_value(
        provider,
        &[
            "compatibility",
            "endpointCompatibility",
            "endpoint_compatibility",
        ],
    )
    .unwrap_or("");
    if base_url_lower.contains("api.openai.com") {
        return Some("openai".to_string());
    }
    let looks_like_default_openai = compatibility.eq_ignore_ascii_case("openai");
    let key = provider_key.trim().to_ascii_lowercase();
    if key == "custom_provider" || key == "custom" {
        return Some("openai".to_string());
    }
    if looks_like_default_openai && key.is_empty() {
        return Some("openai".to_string());
    }
    if provider_string_value(provider, &["apiKey", "api_key", "key"]).is_some() && !key.is_empty() {
        return Some(key);
    }

    None
}

fn should_mirror_to_openai_auth(_provider_key: &str, provider: &serde_json::Value) -> bool {
    let base_url = provider_string_value(provider, &["baseUrl", "base_url"])
        .unwrap_or("")
        .to_ascii_lowercase();
    base_url.contains("api.openai.com")
}

fn upsert_agent_auth_profile(
    openclaw_base: &Path,
    provider_id: &str,
    api_key: &str,
) -> Result<bool, String> {
    let key = api_key.trim();
    if provider_id.trim().is_empty() || key.is_empty() {
        return Ok(false);
    }

    let auth_dir = openclaw_base.join("agents").join("main").join("agent");
    fs::create_dir_all(&auth_dir).map_err(|e| e.to_string())?;
    let auth_path = auth_dir.join("auth-profiles.json");
    let mut auth = if auth_path.exists() {
        let raw = fs::read_to_string(&auth_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    ensure_json_object(&mut auth);
    if !auth.get("version").map(|v| v.is_number()).unwrap_or(false) {
        auth["version"] = serde_json::json!(1);
    }

    let profiles = ensure_json_object_at(&mut auth, "profiles");
    let existing_id = profiles
        .as_object()
        .and_then(|map| {
            map.iter()
                .find(|(_, profile)| {
                    profile
                        .get("provider")
                        .and_then(|v| v.as_str())
                        .map(|v| v.eq_ignore_ascii_case(provider_id))
                        .unwrap_or(false)
                })
                .map(|(id, _)| id.clone())
        })
        .unwrap_or_else(|| format!("{}:manual", provider_id));

    let mut profile = profiles
        .get(&existing_id)
        .cloned()
        .filter(|v| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}));
    profile["type"] = serde_json::json!("api_key");
    profile["provider"] = serde_json::json!(provider_id);
    profile["key"] = serde_json::json!(key);
    profiles
        .as_object_mut()
        .ok_or_else(|| "Invalid auth profile store".to_string())?
        .insert(existing_id, profile);

    let before = if auth_path.exists() {
        fs::read_to_string(&auth_path).unwrap_or_default()
    } else {
        String::new()
    };
    let after = serde_json::to_string_pretty(&auth).map_err(|e| e.to_string())?;
    if before == after {
        return Ok(false);
    }
    fs::write(auth_path, after).map_err(|e| e.to_string())?;
    Ok(true)
}

fn record_synced_auth(
    synced: &mut Vec<String>,
    openclaw_base: &Path,
    provider_id: &str,
    api_key: &str,
) -> Result<(), String> {
    if upsert_agent_auth_profile(openclaw_base, provider_id, api_key)? {
        synced.push(provider_id.to_string());
    }
    Ok(())
}

fn sync_auth_profiles_from_openclaw_config(
    config: &serde_json::Value,
    openclaw_base: &Path,
) -> Result<Vec<String>, String> {
    let mut synced = Vec::new();
    let providers = match config
        .get("models")
        .and_then(|v| v.get("providers"))
        .and_then(|v| v.as_object())
    {
        Some(providers) => providers,
        None => return Ok(synced),
    };

    for (provider_key, provider) in providers {
        let api_key = match provider_string_value(provider, &["apiKey", "api_key", "key"]) {
            Some(api_key) => api_key,
            None => continue,
        };
        let provider_id = match provider_auth_id(provider_key, provider) {
            Some(provider_id) => provider_id,
            None => continue,
        };
        record_synced_auth(&mut synced, openclaw_base, &provider_id, api_key)?;
        if provider_id != "openai" && should_mirror_to_openai_auth(provider_key, provider) {
            record_synced_auth(&mut synced, openclaw_base, "openai", api_key)?;
        }
    }
    synced.sort();
    synced.dedup();
    Ok(synced)
}

fn sync_auth_profiles_from_app_settings(openclaw_base: &Path) -> Result<Vec<String>, String> {
    let raw = match fs::read_to_string(config_file()) {
        Ok(raw) => raw,
        Err(_) => return Ok(Vec::new()),
    };
    let cfg = match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(cfg) => cfg,
        Err(_) => return Ok(Vec::new()),
    };
    let mut synced = Vec::new();

    for api_keys in [
        cfg.get("openclaw").and_then(|v| v.get("apiKeys")),
        cfg.get("apiKeys"),
    ]
    .into_iter()
    .flatten()
    .filter_map(|v| v.as_object())
    {
        for (key, value) in api_keys {
            let api_key = match value.as_str().map(str::trim).filter(|v| !v.is_empty()) {
                Some(api_key) => api_key,
                None => continue,
            };
            if let Some(provider_id) = provider_id_for_key(key) {
                record_synced_auth(&mut synced, openclaw_base, provider_id, api_key)?;
            }
        }
    }

    for providers in [
        cfg.get("openclaw").and_then(|v| v.get("customProviders")),
        cfg.get("customProviders"),
    ]
    .into_iter()
    .flatten()
    .filter_map(|v| v.as_array())
    {
        for provider in providers {
            let api_key = match provider_string_value(provider, &["apiKey", "api_key", "key"]) {
                Some(api_key) => api_key,
                None => continue,
            };
            let name = provider_string_value(provider, &["name", "label", "provider"])
                .unwrap_or("custom_provider");
            let provider_key = provider_id_from_name(name);
            if let Some(provider_id) = provider_auth_id(&provider_key, provider) {
                record_synced_auth(&mut synced, openclaw_base, &provider_id, api_key)?;
                if provider_id != "openai" && should_mirror_to_openai_auth(&provider_key, provider)
                {
                    record_synced_auth(&mut synced, openclaw_base, "openai", api_key)?;
                }
            }
        }
    }

    for (provider_id, env_key) in [
        ("openai", "OPENAI_API_KEY"),
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("google", "GOOGLE_API_KEY"),
        ("groq", "GROQ_API_KEY"),
        ("openrouter", "OPENROUTER_API_KEY"),
        ("xai", "XAI_API_KEY"),
    ] {
        if let Ok(api_key) = std::env::var(env_key) {
            let trimmed = api_key.trim();
            if !trimmed.is_empty() {
                record_synced_auth(&mut synced, openclaw_base, provider_id, trimmed)?;
            }
        }
    }

    synced.sort();
    synced.dedup();
    Ok(synced)
}

fn sync_all_openclaw_auth_profiles(
    config: Option<&serde_json::Value>,
    openclaw_base: &Path,
) -> Result<Vec<String>, String> {
    let mut synced = Vec::new();
    if let Some(config) = config {
        synced.extend(sync_auth_profiles_from_openclaw_config(
            config,
            openclaw_base,
        )?);
    }
    synced.extend(sync_auth_profiles_from_app_settings(openclaw_base)?);
    synced.sort();
    synced.dedup();
    Ok(synced)
}

fn sync_openclaw_auth_profiles_across_known_bases(
    extra_config: Option<&serde_json::Value>,
    extra_base: Option<&Path>,
) -> Result<Vec<String>, String> {
    let _ = sync_openclaw_model_config_across_known_bases(extra_config, extra_base)?;
    let bases = known_openclaw_bases(extra_base);
    let mut configs = Vec::new();
    if let Some(config) = extra_config {
        configs.push(config.clone());
    }
    for base in &bases {
        let path = base.join("openclaw.json");
        if let Some(config) = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        {
            configs.push(config);
        }
    }

    let mut synced = Vec::new();
    for base in &bases {
        for config in &configs {
            synced.extend(sync_auth_profiles_from_openclaw_config(config, base)?);
        }
        synced.extend(sync_auth_profiles_from_app_settings(base)?);
    }
    synced.sort();
    synced.dedup();
    Ok(synced)
}

fn sync_configured_openclaw_auth_profiles() -> Result<Vec<String>, String> {
    let p = resolve_openclaw_config_path();
    let openclaw_base = p
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(configured_openclaw_dir);
    let config = fs::read_to_string(&p)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    sync_openclaw_auth_profiles_across_known_bases(config.as_ref(), Some(&openclaw_base))
}

fn normalize_context_window_values(config: &mut serde_json::Value) {
    if let Some(providers) = config
        .get_mut("models")
        .and_then(|v| v.get_mut("providers"))
        .and_then(|v| v.as_object_mut())
    {
        for provider in providers.values_mut() {
            if let Some(value) = provider.get("contextWindow").and_then(numeric_json_value) {
                provider["contextWindow"] = value;
            }
        }
    }
}

fn provider_has_model_entries(provider: &serde_json::Value) -> bool {
    provider
        .get("models")
        .and_then(|v| v.as_array())
        .map(|models| !models.is_empty())
        .unwrap_or(false)
}

fn provider_first_model_id(provider: &serde_json::Value) -> Option<String> {
    provider
        .get("models")
        .and_then(|v| v.as_array())
        .and_then(|models| models.first())
        .and_then(|model| model.get("id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn provider_model_ref(provider_id: &str, provider: &serde_json::Value) -> Option<String> {
    provider_first_model_id(provider)
        .or_else(|| {
            provider_string_value(provider, &["modelId", "model_id", "model"]).map(str::to_string)
        })
        .map(|model_id| format!("{}/{}", provider_id, model_id))
}

fn sync_agent_default_model_refs(config: &mut serde_json::Value) {
    let mut refs = Vec::new();
    if let Some(primary) = config
        .get("agents")
        .and_then(|v| v.get("defaults"))
        .and_then(|v| v.get("model"))
        .and_then(|v| v.get("primary"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        refs.push(primary.to_string());
    }
    if let Some(providers) = config
        .get("models")
        .and_then(|v| v.get("providers"))
        .and_then(|v| v.as_object())
    {
        for (provider_id, provider) in providers {
            if let Some(model_ref) = provider_model_ref(provider_id, provider) {
                refs.push(model_ref);
            }
        }
    }
    refs.sort();
    refs.dedup();

    let agents = ensure_json_object_at(config, "agents");
    let defaults = ensure_json_object_at(agents, "defaults");
    let models = ensure_json_object_at(defaults, "models");
    for model_ref in refs {
        if !models
            .get(&model_ref)
            .map(|v| v.is_object())
            .unwrap_or(false)
        {
            models[model_ref.as_str()] = serde_json::json!({});
        }
    }
}

fn model_ref_priority(model_ref: &str) -> u8 {
    let lower = model_ref.trim().to_ascii_lowercase();
    if lower.starts_with("openai/") {
        3
    } else {
        1
    }
}

fn prefer_model_ref(current: &mut Option<String>, candidate: String) {
    let should_replace = current
        .as_ref()
        .map(|existing| model_ref_priority(&candidate) < model_ref_priority(existing))
        .unwrap_or(true);
    if should_replace {
        *current = Some(candidate);
    }
}

fn primary_model_needs_repair(current: &str, replacement: &str) -> bool {
    let current = current.trim();
    if current.is_empty() || !current.contains('/') {
        return true;
    }
    current.to_ascii_lowercase().starts_with("openai/")
        && model_ref_priority(replacement) < model_ref_priority(current)
}

fn model_ref_exists_in_config(config: &serde_json::Value, model_ref: &str) -> bool {
    let Some((provider_id, model_id)) = model_ref.trim().split_once('/') else {
        return false;
    };
    let Some(provider) = config
        .get("models")
        .and_then(|v| v.get("providers"))
        .and_then(|v| v.get(provider_id))
    else {
        return false;
    };
    let Some(models) = provider.get("models").and_then(|v| v.as_array()) else {
        return true;
    };
    models
        .iter()
        .any(|model| model.get("id").and_then(|v| v.as_str()) == Some(model_id))
}

fn canonical_provider_id(provider_id: &str, provider: &serde_json::Value) -> String {
    let provider_key = provider_id_from_name(provider_id);
    let provider_name = provider_string_value(provider, &["name", "label", "provider"])
        .unwrap_or("")
        .to_ascii_lowercase();

    if let Some(id) = provider_id_for_key(&provider_key) {
        return id.to_string();
    }
    if !provider_key.is_empty() && provider_key != "custom" && provider_key != "custom_provider" {
        return provider_key;
    }
    if let Some(id) = provider_id_for_key(&provider_name) {
        return id.to_string();
    }
    let normalized_name = provider_id_from_name(&provider_name);
    if !normalized_name.is_empty() {
        return normalized_name;
    }
    provider_key
}

fn provider_default_base_url(provider_key: &str) -> Option<&'static str> {
    match provider_key {
        "openai" => Some(OPENAI_DEFAULT_BASE_URL),
        "anthropic" => Some(ANTHROPIC_DEFAULT_BASE_URL),
        "google" => Some(GOOGLE_DEFAULT_BASE_URL),
        "groq" => Some(GROQ_DEFAULT_BASE_URL),
        "openrouter" => Some(OPENROUTER_DEFAULT_BASE_URL),
        "xai" => Some(XAI_DEFAULT_BASE_URL),
        "9router" => Some(NINE_ROUTER_DEFAULT_BASE_URL),
        _ => None,
    }
}

fn provider_default_api(provider_key: &str, base_url: &str) -> Option<&'static str> {
    match provider_key {
        "anthropic" => Some("anthropic-messages"),
        "google" => Some("google-generative-ai"),
        "groq" | "openrouter" | "xai" | "9router" => Some("openai-completions"),
        "openai" => {
            if base_url.is_empty() || base_url.contains("api.openai.com") {
                None
            } else {
                Some("openai-completions")
            }
        }
        _ => None,
    }
}

fn provider_default_models(provider_key: &str) -> Option<serde_json::Value> {
    let models = match provider_key {
        "openai" => serde_json::json!([
            {
                "api": "openai-completions",
                "id": "gpt-5.5",
                "name": "GPT-5.5",
                "reasoning": true,
                "input": ["text", "image"],
                "maxTokens": DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
                "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            },
            {
                "api": "openai-completions",
                "id": "gpt-5.4-mini",
                "name": "GPT-5.4 Mini",
                "reasoning": true,
                "input": ["text", "image"],
                "maxTokens": DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
                "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            }
        ]),
        "anthropic" => serde_json::json!([
            {
                "api": "anthropic-messages",
                "id": "claude-opus-4-6",
                "name": "Claude Opus 4.6",
                "reasoning": false,
                "input": ["text", "image"],
                "maxTokens": DEFAULT_ANTHROPIC_MAX_TOKENS,
                "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            }
        ]),
        "google" => serde_json::json!([
            {
                "api": "google-generative-ai",
                "id": "gemini-3.1-pro-preview",
                "name": "Gemini 3.1 Pro Preview",
                "reasoning": true,
                "input": ["text", "image"],
                "maxTokens": DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
                "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            },
            {
                "api": "google-generative-ai",
                "id": "gemini-3-flash-preview",
                "name": "Gemini 3 Flash Preview",
                "reasoning": false,
                "input": ["text", "image"],
                "maxTokens": DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
                "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            }
        ]),
        "groq" => serde_json::json!([
            {
                "api": "openai-completions",
                "id": "llama-3.3-70b-versatile",
                "name": "Llama 3.3 70B Versatile",
                "reasoning": false,
                "input": ["text"],
                "maxTokens": DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
                "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            }
        ]),
        "openrouter" => serde_json::json!([
            {
                "api": "openai-completions",
                "id": "auto",
                "name": "OpenRouter Auto",
                "reasoning": false,
                "input": ["text", "image"],
                "maxTokens": DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
                "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            }
        ]),
        "xai" => serde_json::json!([
            {
                "api": "openai-completions",
                "id": "grok-4",
                "name": "Grok 4",
                "reasoning": true,
                "input": ["text", "image"],
                "maxTokens": DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
                "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            }
        ]),
        "9router" => serde_json::json!([
            {
                "api": "openai-completions",
                "id": "gpt5.5",
                "name": "gpt5.5",
                "reasoning": false,
                "input": ["text", "image"],
                "maxTokens": DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
                "cost": {
                    "input": 0,
                    "output": 0,
                    "cacheRead": 0,
                    "cacheWrite": 0
                },
                "contextWindow": DEFAULT_CONTEXT_WINDOW
            }
        ]),
        _ => return None,
    };
    Some(models)
}

fn canonicalize_provider_map(config: &mut serde_json::Value) {
    let Some(providers) = config
        .get_mut("models")
        .and_then(|v| v.get_mut("providers"))
        .and_then(|v| v.as_object_mut())
    else {
        return;
    };

    let original = std::mem::take(providers);
    let mut canonicalized = serde_json::Map::new();
    for (provider_id, provider) in original {
        if !provider.is_object() {
            continue;
        }
        let canonical_id = canonical_provider_id(&provider_id, &provider);
        if let Some(existing) = canonicalized.get_mut(&canonical_id) {
            let mut merged = provider;
            merge_missing_json(&mut merged, existing);
            *existing = merged;
        } else {
            canonicalized.insert(canonical_id, provider);
        }
    }
    *providers = canonicalized;
}

fn provider_can_default_base_url(provider_id: &str, _provider: &serde_json::Value) -> bool {
    let provider_key = provider_id.trim().to_ascii_lowercase();
    provider_default_base_url(&provider_key).is_some() || provider_key == "9router"
}

fn default_max_tokens_for_api(api: &str) -> u64 {
    match api {
        "anthropic-messages" => DEFAULT_ANTHROPIC_MAX_TOKENS,
        "openai-codex-responses" => DEFAULT_CODEX_MAX_TOKENS,
        _ => DEFAULT_OPENAI_COMPAT_MAX_TOKENS,
    }
}

fn default_custom_model_entry(provider: &serde_json::Value, model_id: String) -> serde_json::Value {
    let context_window = provider
        .get("contextWindow")
        .and_then(numeric_json_value)
        .unwrap_or_else(|| serde_json::json!(DEFAULT_CONTEXT_WINDOW));
    let model_name = provider_string_value(provider, &["alias"])
        .map(str::to_string)
        .unwrap_or_else(|| model_id.clone());
    let api = provider_string_value(provider, &["api"]).unwrap_or("openai-completions");
    let max_tokens = provider
        .get("maxTokens")
        .or_else(|| provider.get("max_tokens"))
        .and_then(numeric_json_value)
        .unwrap_or_else(|| serde_json::json!(default_max_tokens_for_api(api)));
    let reasoning = provider
        .get("reasoning")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    serde_json::json!({
        "api": api,
        "id": model_id,
        "name": model_name,
        "reasoning": reasoning,
        "input": ["text", "image"],
        "maxTokens": max_tokens,
        "cost": {
            "input": 0,
            "output": 0,
            "cacheRead": 0,
            "cacheWrite": 0
        },
        "contextWindow": context_window
    })
}

fn normalize_provider_model_entries(provider: &mut serde_json::Value) {
    let provider_api = provider_string_value(provider, &["api"])
        .unwrap_or("openai-completions")
        .to_string();
    let default_max_tokens = provider
        .get("maxTokens")
        .or_else(|| provider.get("max_tokens"))
        .and_then(numeric_json_value)
        .unwrap_or_else(|| serde_json::json!(default_max_tokens_for_api(&provider_api)));
    let provider_context_window = provider
        .get("contextWindow")
        .or_else(|| provider.get("context_window"))
        .and_then(numeric_json_value)
        .unwrap_or_else(|| serde_json::json!(DEFAULT_CONTEXT_WINDOW));
    let default_reasoning = provider
        .get("reasoning")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let Some(models) = provider.get_mut("models").and_then(|v| v.as_array_mut()) else {
        return;
    };

    for model in models {
        ensure_json_object(model);
        if string_missing_or_empty(model, "api") {
            model["api"] = serde_json::json!(provider_api.clone());
        }
        if !model.get("input").map(|v| v.is_array()).unwrap_or(false) {
            model["input"] = serde_json::json!(["text", "image"]);
        }
        if !model.get("cost").map(|v| v.is_object()).unwrap_or(false) {
            model["cost"] = serde_json::json!({
                "input": 0,
                "output": 0,
                "cacheRead": 0,
                "cacheWrite": 0
            });
        }
        if let Some(value) = model.get("contextWindow").and_then(numeric_json_value) {
            model["contextWindow"] = value;
        } else {
            model["contextWindow"] = provider_context_window.clone();
        }
        if let Some(value) = model.get("maxTokens").and_then(numeric_json_value) {
            model["maxTokens"] = value;
        } else {
            model["maxTokens"] = default_max_tokens.clone();
        }
        if !model
            .get("reasoning")
            .map(|v| v.is_boolean())
            .unwrap_or(false)
        {
            model["reasoning"] = serde_json::json!(default_reasoning);
        }
        if string_missing_or_empty(model, "name") {
            let model_id = model
                .get("id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_default();
            if !model_id.is_empty() {
                model["name"] = serde_json::json!(model_id);
            }
        }
    }
}

fn ensure_provider_runtime_defaults(provider_id: &str, provider: &mut serde_json::Value) {
    ensure_json_object(provider);
    let provider_key = provider_id.trim().to_ascii_lowercase();
    if string_missing_or_empty(provider, "baseUrl") && string_missing_or_empty(provider, "base_url")
    {
        if let Some(default_base_url) = provider_default_base_url(&provider_key) {
            provider["baseUrl"] = serde_json::json!(default_base_url);
        }
    }

    let effective_base_url = provider_string_value(provider, &["baseUrl", "base_url"])
        .unwrap_or("")
        .to_ascii_lowercase();
    let effective_key = provider_key.as_str();
    if string_missing_or_empty(provider, "api") {
        if let Some(default_api) = provider_default_api(effective_key, &effective_base_url) {
            provider["api"] = serde_json::json!(default_api);
        }
    }

    if !provider_has_model_entries(provider) {
        if provider_string_value(provider, &["modelId", "model_id", "model"]).is_none() {
            if let Some(default_models) = provider_default_models(effective_key) {
                provider["models"] = default_models;
            } else if !effective_base_url.is_empty() {
                provider["models"] = serde_json::json!([default_custom_model_entry(
                    provider,
                    "gpt-5.5".to_string()
                )]);
            }
        }
    }
    normalize_provider_model_entries(provider);
}

fn normalize_openclaw_model_config(config: &mut serde_json::Value) {
    canonicalize_provider_map(config);
    normalize_context_window_values(config);
    let mut generated_primary = None;
    if let Some(providers) = config
        .get_mut("models")
        .and_then(|v| v.get_mut("providers"))
        .and_then(|v| v.as_object_mut())
    {
        for (provider_id, provider) in providers.iter_mut() {
            ensure_provider_runtime_defaults(provider_id, provider);
            let base_url = provider_string_value(provider, &["baseUrl", "base_url"])
                .unwrap_or("")
                .to_ascii_lowercase();
            let compatibility = provider_string_value(
                provider,
                &[
                    "compatibility",
                    "endpointCompatibility",
                    "endpoint_compatibility",
                ],
            )
            .unwrap_or("")
            .to_ascii_lowercase();
            let provider_key = provider_id.to_ascii_lowercase();
            let native_openai = provider_key == "openai"
                && (base_url.is_empty() || base_url.contains("api.openai.com"));
            let looks_openai_compatible = provider_key == "9router"
                || provider_key == "custom_provider"
                || compatibility == "openai"
                || (!native_openai
                    && (base_url.contains("/v1")
                        || base_url.contains("localhost")
                        || base_url.contains("127.0.0.1")));
            if looks_openai_compatible && string_missing_or_empty(provider, "api") {
                provider["api"] = serde_json::json!("openai-completions");
            }
            let models_missing = !provider_has_model_entries(provider);
            if models_missing {
                if let Some(model_id) =
                    provider_string_value(provider, &["modelId", "model_id", "model"])
                        .map(str::to_string)
                {
                    provider["models"] =
                        serde_json::json!([default_custom_model_entry(provider, model_id.clone())]);
                    prefer_model_ref(
                        &mut generated_primary,
                        format!("{}/{}", provider_id, model_id),
                    );
                }
            } else if let Some(model_ref) = provider_model_ref(provider_id, provider) {
                prefer_model_ref(&mut generated_primary, model_ref);
            }
            if let Some(obj) = provider.as_object_mut() {
                for key in [
                    "alias",
                    "modelId",
                    "model_id",
                    "model",
                    "name",
                    "contextWindow",
                    "context_window",
                    "maxTokens",
                    "max_tokens",
                    "reasoning",
                    "input",
                    "compatibility",
                    "endpointCompatibility",
                    "endpoint_compatibility",
                ] {
                    obj.remove(key);
                }
            }
        }
    }

    if let Some(primary) = generated_primary {
        let current = config
            .get("agents")
            .and_then(|v| v.get("defaults"))
            .and_then(|v| v.get("model"))
            .and_then(|v| v.get("primary"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if primary_model_needs_repair(current, &primary)
            || !model_ref_exists_in_config(config, current)
        {
            config["agents"]["defaults"]["model"]["primary"] = serde_json::json!(primary);
        }
    }
    sync_agent_default_model_refs(config);
}

fn model_providers_from_config(
    config: &serde_json::Value,
) -> Option<&serde_json::Map<String, serde_json::Value>> {
    config
        .get("models")
        .and_then(|v| v.get("providers"))
        .and_then(|v| v.as_object())
}

fn preferred_primary_from_config(config: &serde_json::Value) -> Option<String> {
    let configured = config
        .get("agents")
        .and_then(|v| v.get("defaults"))
        .and_then(|v| v.get("model"))
        .and_then(|v| v.get("primary"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    let mut generated = None;
    if let Some(providers) = model_providers_from_config(config) {
        for (provider_id, provider) in providers {
            if let Some(model_ref) = provider_model_ref(provider_id, provider) {
                prefer_model_ref(&mut generated, model_ref);
            }
        }
    }

    match (configured, generated) {
        (Some(current), Some(replacement))
            if primary_model_needs_repair(&current, &replacement) =>
        {
            Some(replacement)
        }
        (Some(current), _) if current.contains('/') => Some(current),
        (_, Some(replacement)) => Some(replacement),
        (Some(current), _) => Some(current),
        _ => None,
    }
}

fn openclaw_model_config_from_app_settings() -> Option<serde_json::Value> {
    let raw = fs::read_to_string(config_file()).ok()?;
    let cfg = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    let mut config = serde_json::json!({
        "agents": { "defaults": {} },
        "models": { "mode": "merge", "providers": {} }
    });
    let mut found = false;

    for api_keys in [
        cfg.get("openclaw").and_then(|v| v.get("apiKeys")),
        cfg.get("apiKeys"),
    ]
    .into_iter()
    .flatten()
    .filter_map(|v| v.as_object())
    {
        for (key, value) in api_keys {
            let Some(api_key) = value.as_str().map(str::trim).filter(|v| !v.is_empty()) else {
                continue;
            };
            let Some(provider_id) = provider_id_for_key(key) else {
                continue;
            };
            let provider = &mut config["models"]["providers"][provider_id];
            provider["apiKey"] = serde_json::json!(api_key);
            ensure_provider_runtime_defaults(provider_id, provider);
            found = true;
        }
    }

    for providers in [
        cfg.get("openclaw").and_then(|v| v.get("customProviders")),
        cfg.get("customProviders"),
    ]
    .into_iter()
    .flatten()
    .filter_map(|v| v.as_array())
    {
        for provider_config in providers {
            let name = provider_string_value(provider_config, &["name", "label", "provider"])
                .unwrap_or("")
                .trim();
            if name.is_empty() {
                continue;
            }
            let provider_id = provider_id_from_name(name);
            if provider_string_value(provider_config, &["baseUrl", "base_url"]).is_none()
                && !provider_can_default_base_url(&provider_id, provider_config)
            {
                continue;
            }
            let provider = &mut config["models"]["providers"][provider_id.as_str()];
            provider["name"] = serde_json::json!(name);
            if let Some(api) = provider_string_value(provider_config, &["api", "apiType"]) {
                provider["api"] = serde_json::json!(api);
            }
            if let Some(base_url) = provider_string_value(provider_config, &["baseUrl", "base_url"])
            {
                provider["baseUrl"] = serde_json::json!(base_url);
            }
            if let Some(api_key) =
                provider_string_value(provider_config, &["apiKey", "api_key", "key"])
            {
                provider["apiKey"] = serde_json::json!(api_key);
            }
            if let Some(value) = provider_value(provider_config, &["modelId", "model_id", "model"])
                .and_then(string_json_value)
            {
                provider["modelId"] = value;
            }
            if let Some(value) =
                provider_value(provider_config, &["contextWindow", "context_window"])
                    .and_then(numeric_json_value)
            {
                provider["contextWindow"] = value;
            }
            if let Some(value) = provider_value(provider_config, &["maxTokens", "max_tokens"])
                .and_then(numeric_json_value)
            {
                provider["maxTokens"] = value;
            }
            if let Some(value) =
                provider_value(provider_config, &["reasoning"]).and_then(bool_json_value)
            {
                provider["reasoning"] = value;
            }
            if let Some(value) =
                provider_value(provider_config, &["alias"]).and_then(string_json_value)
            {
                provider["alias"] = value;
            }
            ensure_provider_runtime_defaults(&provider_id, provider);
            found = true;
        }
    }

    if !found {
        return None;
    }
    normalize_openclaw_model_config(&mut config);
    Some(config)
}

fn sync_agent_models_file(
    openclaw_base: &Path,
    aggregate_providers: &serde_json::Map<String, serde_json::Value>,
) -> Result<bool, String> {
    if aggregate_providers.is_empty() {
        return Ok(false);
    }
    let agent_dir = openclaw_base.join("agents").join("main").join("agent");
    fs::create_dir_all(&agent_dir).map_err(|e| e.to_string())?;
    let path = agent_dir.join("models.json");
    let mut config = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    ensure_json_object(&mut config);
    if config.get("providers").is_none() {
        if let Some(providers) = config
            .get("models")
            .and_then(|v| v.get("providers"))
            .cloned()
        {
            config["providers"] = providers;
        }
    }
    let before = if path.exists() {
        fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };
    {
        let providers = ensure_json_object_at(&mut config, "providers");
        for (provider_id, provider) in aggregate_providers {
            providers[provider_id.as_str()] = provider.clone();
        }
    }
    let mut wrapper = serde_json::json!({
        "models": {
            "mode": "merge",
            "providers": config.get("providers").cloned().unwrap_or_else(|| serde_json::json!({}))
        }
    });
    normalize_openclaw_model_config(&mut wrapper);
    config["providers"] = wrapper["models"]["providers"].clone();
    let after = format!(
        "{}\n",
        serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?
    );
    if before == after && path.exists() {
        return Ok(false);
    }
    if path.exists() {
        backup_existing_file(&path)?;
    }
    fs::write(path, after).map_err(|e| e.to_string())?;
    Ok(true)
}

fn sync_openclaw_model_config_across_known_bases(
    extra_config: Option<&serde_json::Value>,
    extra_base: Option<&Path>,
) -> Result<Option<String>, String> {
    let bases = known_openclaw_bases(extra_base);
    let mut normalized_configs = Vec::new();
    if let Some(config) = extra_config {
        let mut value = config.clone();
        normalize_openclaw_model_config(&mut value);
        normalized_configs.push(value);
    }
    if let Some(mut config) = openclaw_model_config_from_app_settings() {
        normalize_openclaw_model_config(&mut config);
        normalized_configs.push(config);
    }
    for base in &bases {
        let path = base.join("openclaw.json");
        if let Some(mut config) = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        {
            normalize_openclaw_model_config(&mut config);
            normalized_configs.push(config);
        }
    }

    let mut aggregate_providers = serde_json::Map::new();
    let mut primary = None;
    for config in &normalized_configs {
        if let Some(candidate) = preferred_primary_from_config(config) {
            let should_replace = primary
                .as_ref()
                .map(|current: &String| {
                    current.starts_with("openai/") && !candidate.starts_with("openai/")
                })
                .unwrap_or(true);
            if should_replace {
                primary = Some(candidate);
            }
        }
        if let Some(providers) = model_providers_from_config(config) {
            for (provider_id, provider) in providers {
                if provider.is_object() {
                    aggregate_providers.insert(provider_id.clone(), provider.clone());
                }
            }
        }
    }

    if aggregate_providers.is_empty() {
        return Ok(primary);
    }

    for base in &bases {
        let path = base.join("openclaw.json");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut config = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        if !config.is_object() {
            config = serde_json::json!({});
        }
        for (provider_id, provider) in &aggregate_providers {
            config["models"]["providers"][provider_id] = provider.clone();
        }
        if let Some(primary) = &primary {
            let current = config
                .get("agents")
                .and_then(|v| v.get("defaults"))
                .and_then(|v| v.get("model"))
                .and_then(|v| v.get("primary"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .unwrap_or("");
            if current.is_empty() || current.starts_with("openai/") {
                config["agents"]["defaults"]["model"]["primary"] = serde_json::json!(primary);
            }
        }
        write_openclaw_config_file(&path, &mut config)?;
        let _ = sync_agent_models_file(base, &aggregate_providers)?;
    }

    Ok(primary)
}

fn sync_to_openclaw(cfg: &AppConfig) -> Result<(), String> {
    let p = resolve_openclaw_config_path();
    let mut oc: serde_json::Value = if p.exists() {
        fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let openclaw_base = p
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(configured_openclaw_dir);
    if !cfg.openclaw.workspace_dir.trim().is_empty() {
        let workspace = path_as_config_string(&PathBuf::from(cfg.openclaw.workspace_dir.trim()));
        oc["agents"]["defaults"]["workspace"] = serde_json::json!(workspace);
    }
    let model = &cfg.openclaw.model;
    let primary = if model.is_array() {
        model
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else {
        model.as_str().unwrap_or("")
    };
    if !primary.is_empty() {
        oc["agents"]["defaults"]["model"]["primary"] = serde_json::json!(primary);
    }
    for (env_key, val) in &cfg.openclaw.api_keys {
        let v = val.trim();
        if v.is_empty() {
            continue;
        }
        if let Some(pid) = provider_id_for_key(env_key) {
            oc["models"]["providers"][pid]["apiKey"] = serde_json::json!(v);
        }
    }
    for provider in &cfg.openclaw.custom_providers {
        let name = provider
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if name.is_empty() {
            continue;
        }
        let id = provider_id_from_name(name);
        if provider_string_value(provider, &["baseUrl", "base_url"]).is_none()
            && !provider_can_default_base_url(&id, provider)
        {
            continue;
        }
        oc["models"]["providers"][id.as_str()]["name"] = serde_json::json!(name);
        if let Some(api) = provider_string_value(provider, &["api", "apiType"]) {
            oc["models"]["providers"][id.as_str()]["api"] = serde_json::json!(api);
        }
        if let Some(base_url) = provider
            .get("baseUrl")
            .or_else(|| provider.get("base_url"))
            .and_then(|v| v.as_str())
            .filter(|v| !v.trim().is_empty())
        {
            oc["models"]["providers"][id.as_str()]["baseUrl"] = serde_json::json!(base_url.trim());
        }
        if let Some(api_key) = provider
            .get("apiKey")
            .or_else(|| provider.get("api_key"))
            .and_then(|v| v.as_str())
            .filter(|v| !v.trim().is_empty())
        {
            oc["models"]["providers"][id.as_str()]["apiKey"] = serde_json::json!(api_key.trim());
        }
        if let Some(value) =
            provider_value(provider, &["modelId", "model_id", "model"]).and_then(string_json_value)
        {
            oc["models"]["providers"][id.as_str()]["modelId"] = value;
        }
        if let Some(value) = provider_value(provider, &["contextWindow", "context_window"])
            .and_then(numeric_json_value)
        {
            oc["models"]["providers"][id.as_str()]["contextWindow"] = value;
        }
        if let Some(value) =
            provider_value(provider, &["maxTokens", "max_tokens"]).and_then(numeric_json_value)
        {
            oc["models"]["providers"][id.as_str()]["maxTokens"] = value;
        }
        if let Some(value) = provider_value(provider, &["reasoning"]).and_then(bool_json_value) {
            oc["models"]["providers"][id.as_str()]["reasoning"] = value;
        }
        if let Some(value) = provider_value(provider, &["alias"]).and_then(string_json_value) {
            oc["models"]["providers"][id.as_str()]["alias"] = value;
        }
    }
    if !cfg.telegram.bots.is_empty() {
        let first = &cfg.telegram.bots[0];
        if !first.bot_token.is_empty() {
            oc["channels"]["telegram"]["botToken"] = serde_json::json!(first.bot_token.trim());
            oc["channels"]["telegram"]["enabled"] = serde_json::json!(true);
        }
        if !first.chat_id.is_empty() {
            oc["channels"]["telegram"]["defaultTo"] = serde_json::json!(first.chat_id.trim());
        }
    } else {
        if !cfg.telegram.api_key.is_empty() {
            oc["channels"]["telegram"]["botToken"] = serde_json::json!(cfg.telegram.api_key.trim());
            oc["channels"]["telegram"]["enabled"] = serde_json::json!(true);
        }
        if !cfg.telegram.group_id.is_empty() {
            oc["channels"]["telegram"]["defaultTo"] =
                serde_json::json!(cfg.telegram.group_id.trim());
        }
    }
    if !cfg.google.client_id.is_empty()
        || !cfg.google.client_secret.is_empty()
        || !cfg.google.api_key.is_empty()
    {
        oc["integrations"]["google"]["clientId"] = serde_json::json!(cfg.google.client_id.trim());
        oc["integrations"]["google"]["clientSecret"] =
            serde_json::json!(cfg.google.client_secret.trim());
        oc["integrations"]["google"]["apiKey"] = serde_json::json!(cfg.google.api_key.trim());
    }
    if !cfg.n8n.base_url.is_empty() || !cfg.n8n.api_key.is_empty() {
        oc["integrations"]["n8n"]["baseUrl"] = serde_json::json!(cfg.n8n.base_url.trim());
        oc["integrations"]["n8n"]["apiKey"] = serde_json::json!(cfg.n8n.api_key.trim());
    }
    sync_openclaw_auth_profiles_across_known_bases(Some(&oc), Some(&openclaw_base))?;
    write_openclaw_config_file(&p, &mut oc)?;
    Ok(())
}

#[tauri::command]
fn list_logs() -> Vec<String> {
    let d = log_dir();
    if !d.exists() {
        return vec![];
    }
    fs::read_dir(d)
        .ok()
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "log").unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[tauri::command]
fn read_log(name: String) -> String {
    let p = log_dir().join(Path::new(&name).file_name().unwrap_or_default());
    fs::read_to_string(p)
        .ok()
        .map(|s| {
            let chars: Vec<char> = s.chars().collect();
            if chars.len() > 50000 {
                chars[chars.len() - 50000..].iter().collect()
            } else {
                s
            }
        })
        .unwrap_or_default()
}

#[tauri::command]
fn run_terminal_cmd(cmd: String) -> Result<String, String> {
    validate_terminal_cmd(&cmd)?;
    let handle = thread::spawn(move || run_ps(&cmd));
    match handle.join() {
        Ok(result) => result,
        Err(_) => Err("terminal command worker crashed".to_string()),
    }
}

#[tauri::command]
fn open_logs_folder() {
    let d = log_dir();
    let _ = fs::create_dir_all(&d);
    let _ = opener::open(d.to_string_lossy().to_string());
}

#[tauri::command]
fn backup_app_data(app: String, destination_dir: String) -> Result<String, String> {
    let app_key = app.trim().to_lowercase();
    let dest = PathBuf::from(destination_dir);
    if !dest.exists() || !dest.is_dir() {
        return Err("Backup folder does not exist".into());
    }
    let sources = backup_sources(&app_key)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let zip_name = format!("{}_backup_{}.zip", safe_zip_name(&app_key), timestamp);
    let zip_path = dest.join(zip_name);
    let file = File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    let manifest = serde_json::json!({
        "app": app_key,
        "created_at_unix": timestamp,
        "format": BACKUP_FORMAT,
        "sources": sources.iter().map(|s| serde_json::json!({
            "folder": s.label,
            "original_path": s.path.to_string_lossy().to_string()
        })).collect::<Vec<_>>()
    });
    zip.start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&manifest)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    let mut count = 0u64;
    for source in &sources {
        count += copy_source_to_zip(&mut zip, &app_key, source, options)?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    if count == 0 {
        let _ = fs::remove_file(&zip_path);
        return Err("No data folder found for the selected app".into());
    }
    Ok(format!("Backup completed: {}", zip_path.display()))
}

#[tauri::command]
fn restore_app_data(app: String, zip_file: String) -> Result<String, String> {
    let app_key = app.trim().to_lowercase();
    let file = File::open(PathBuf::from(zip_file)).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let allowed_sources = validate_backup_manifest(&mut archive, &app_key)?;
    let prefix = format!("{}/", app_key);
    let mut restored = 0u64;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().replace('\\', "/");
        if name.ends_with("manifest.json") {
            continue;
        }
        let relative = if name.starts_with(&prefix) {
            PathBuf::from(name.trim_start_matches(&prefix))
        } else {
            PathBuf::from(&name)
        };
        if !is_safe_restore_path(&relative) {
            return Err("Backup contains an unsafe path".into());
        }
        let mut parts = relative.components();
        let first = parts
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("");
        if !allowed_sources
            .iter()
            .any(|label| label.eq_ignore_ascii_case(first))
        {
            continue;
        }
        let remainder: PathBuf = parts.collect();
        let target_root = restore_target(&app_key, first)
            .ok_or_else(|| format!("Unknown restore target: {}", first))?;
        let target = join_restore_target(&target_root, remainder);

        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = File::create(&target).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        restored += 1;
    }

    if restored == 0 {
        return Err("No matching backup content found for the selected app".into());
    }
    Ok(format!("Restore completed: {} files", restored))
}

#[tauri::command]
fn load_openclaw_config() -> Result<serde_json::Value, String> {
    let p = resolve_openclaw_config_path();
    if !p.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = fs::read_to_string(&p).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_openclaw_config(mut config: serde_json::Value) -> Result<String, String> {
    let requested_base = requested_openclaw_base(&config);
    let requested_workspace = requested_workspace_root(&config);
    let p = requested_base
        .as_ref()
        .map(|base| base.join("openclaw.json"))
        .unwrap_or_else(resolve_openclaw_config_path);
    write_openclaw_config_file(&p, &mut config)?;
    let openclaw_base = requested_base
        .clone()
        .or_else(|| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(configured_openclaw_dir);
    let synced_auth =
        sync_openclaw_auth_profiles_across_known_bases(Some(&config), Some(&openclaw_base))?;
    if requested_base.is_some() || requested_workspace.is_some() {
        let base = requested_base
            .or_else(|| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(configured_openclaw_dir);
        let workspace = requested_workspace.unwrap_or_else(configured_openclaw_workspace_dir);
        persist_openclaw_setup_paths(&base, &workspace)?;
    }
    if synced_auth.is_empty() {
        Ok("saved".into())
    } else {
        Ok(format!("saved; auth synced: {}", synced_auth.join(", ")))
    }
}

#[tauri::command]
async fn setup_workspace(openclaw_dir: String, workspace_dir: String) -> Result<String, String> {
    setup_openclaw_files(openclaw_dir, workspace_dir, None, None)
}

#[tauri::command]
fn add_to_path(tool: String) -> Result<String, String> {
    let tool_key = match tool.trim().to_lowercase().as_str() {
        "node" => "node",
        "openclaw" => "openclaw",
        "claude-code" | "claude" => "claude-code",
        "9router" => "9router",
        "n8n" => "n8n",
        "ngrok" => "ngrok",
        "git" => "git",
        "python" => "python",
        _ => return Err("Unsupported tool for PATH".into()),
    };
    let script = format!(
        r#"
$tool = {}
$paths = @()
switch ($tool) {{
    'node' {{
        $paths = @(
            'C:\Program Files\nodejs',
            "${{env:ProgramFiles(x86)}}\nodejs",
            (Join-Path $env:LOCALAPPDATA 'Programs\nodejs')
        )
        $cmd = Get-Command node -EA SilentlyContinue
        if ($cmd -and $cmd.Source) {{ $paths += (Split-Path $cmd.Source) }}
    }}
    'openclaw' {{ $paths = @((Join-Path $env:APPDATA 'npm')) }}
    'claude-code' {{ $paths = @((Join-Path $env:APPDATA 'npm')) }}
    '9router' {{ $paths = @((Join-Path $env:APPDATA 'npm')) }}
    'n8n' {{ $paths = @((Join-Path $env:APPDATA 'npm')) }}
    'ngrok' {{ $paths = @((Join-Path $env:LOCALAPPDATA 'Ngrok Tunnel'), (Join-Path $env:LOCALAPPDATA 'ngrok')) }}
    'git' {{ $paths = @('C:\Program Files\Git\cmd') }}
    'python' {{
        $cmd = Get-Command python -All -EA SilentlyContinue | Where-Object {{ $_.Source -and $_.Source -notmatch 'WindowsApps' }} | Select-Object -First 1
        if ($cmd) {{ $paths += (Split-Path $cmd.Source) }}
        $roots = @($env:ProgramFiles, ${{env:ProgramFiles(x86)}}, (Join-Path $env:LOCALAPPDATA 'Programs\Python')) | Where-Object {{ $_ }}
        foreach ($root in $roots) {{
            Get-ChildItem -LiteralPath $root -Directory -Filter 'Python*' -EA SilentlyContinue | ForEach-Object {{
                if (Test-Path -LiteralPath (Join-Path $_.FullName 'python.exe')) {{
                    $paths += $_.FullName
                    if (Test-Path -LiteralPath (Join-Path $_.FullName 'Scripts')) {{ $paths += (Join-Path $_.FullName 'Scripts') }}
                }}
            }}
        }}
    }}
}}
$paths = @($paths | Where-Object {{ $_ -and (Test-Path -LiteralPath $_) }} | ForEach-Object {{ (Resolve-Path -LiteralPath $_).Path }} | Select-Object -Unique)
if ($paths.Count -eq 0) {{ throw "No valid PATH target found for $tool" }}
$userPath = [Environment]::GetEnvironmentVariable('Path','User')
if ($null -eq $userPath) {{ $userPath = '' }}
$normalized = @($userPath -split ';' | Where-Object {{ $_.Trim() }} | ForEach-Object {{ [Environment]::ExpandEnvironmentVariables($_).Trim().TrimEnd('\').ToLowerInvariant() }})
$added = @()
foreach ($p in $paths) {{
    $norm = [Environment]::ExpandEnvironmentVariables($p).Trim().TrimEnd('\').ToLowerInvariant()
    if ($p -and -not ($normalized -contains $norm)) {{
        if ([string]::IsNullOrWhiteSpace($userPath)) {{ $userPath = $p }} else {{ $userPath = $userPath.TrimEnd(';') + ';' + $p }}
        $added += $p
        $normalized += $norm
    }}
}}
if ($added.Count -gt 0) {{
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
    if (-not $key) {{ throw 'Cannot open HKCU:\Environment for writing' }}
    $key.SetValue('Path', $userPath, [Microsoft.Win32.RegistryValueKind]::ExpandString)
    $key.Close()
    [Environment]::SetEnvironmentVariable('Path', $userPath, 'User')
    $verify = (Get-ItemProperty -Path 'HKCU:\Environment' -Name 'Path' -EA Stop).Path
    $missing = @()
    foreach ($p in $added) {{
        $norm = [Environment]::ExpandEnvironmentVariables($p).Trim().TrimEnd('\').ToLowerInvariant()
        $ok = @($verify -split ';' | ForEach-Object {{ [Environment]::ExpandEnvironmentVariables($_).Trim().TrimEnd('\').ToLowerInvariant() }}) -contains $norm
        if (-not $ok) {{ $missing += $p }}
    }}
    if ($missing.Count -gt 0) {{ throw ('Path write verification failed: ' + ($missing -join ', ')) }}
    try {{
        $typeDefinition = @"
using System;
using System.Runtime.InteropServices;
public class EnvBroadcast {{
    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(IntPtr hWnd, int Msg, UIntPtr wParam, string lParam, int flags, int timeout, out UIntPtr result);
}}
"@
        Add-Type -TypeDefinition $typeDefinition -EA SilentlyContinue
        $broadcastResult = [UIntPtr]::Zero
        [EnvBroadcast]::SendMessageTimeout([IntPtr]0xffff, 0x1A, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$broadcastResult) | Out-Null
    }} catch {{ }}
    'Added to User Path: ' + ($added -join ', ')
}} else {{
    'Already in User Path: ' + ($paths -join ', ')
}}
"#,
        ps_single_quote(tool_key)
    );
    run_ps(&script)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    verify_ownership_notice();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            gateway_status,
            check_tools,
            check_versions,
            app_statuses,
            run_action,
            load_settings,
            save_settings,
            list_logs,
            read_log,
            open_logs_folder,
            run_terminal_cmd,
            load_openclaw_config,
            save_openclaw_config,
            setup_workspace,
            setup_openclaw_files,
            add_to_path,
            backup_app_data,
            restore_app_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_provider_local_base_url_keeps_user_provider_id() {
        let mut config = serde_json::json!({
            "agents": {
                "defaults": {
                    "model": { "primary": "abc/cx/gpt-5.5" }
                }
            },
            "models": {
                "mode": "merge",
                "providers": {
                    "abc": {
                        "name": "abc",
                        "baseUrl": "http://localhost:20128/v1",
                        "apiKey": "sk-test",
                        "modelId": "cx/gpt-5.5",
                        "contextWindow": 500000
                    }
                }
            }
        });

        repair_openclaw_config_value(&mut config, None);
        normalize_openclaw_model_config(&mut config);

        let providers = config["models"]["providers"].as_object().unwrap();
        assert!(providers.contains_key("abc"));
        assert!(!providers.contains_key("9router"));
        assert_eq!(
            config["agents"]["defaults"]["model"]["primary"],
            serde_json::json!("abc/cx/gpt-5.5")
        );
        assert_eq!(
            config["models"]["providers"]["abc"]["models"][0]["api"],
            serde_json::json!("openai-completions")
        );
        assert_eq!(
            config["models"]["providers"]["abc"]["models"][0]["maxTokens"],
            serde_json::json!(DEFAULT_OPENAI_COMPAT_MAX_TOKENS)
        );
        assert!(config["models"]["providers"]["abc"]
            .get("modelId")
            .is_none());
        assert!(config["models"]["providers"]["abc"]
            .get("contextWindow")
            .is_none());
    }

    #[test]
    fn custom_provider_user_model_fields_land_on_model_entry() {
        let mut config = serde_json::json!({
            "agents": {
                "defaults": {}
            },
            "models": {
                "mode": "merge",
                "providers": {
                    "custom_proxy": {
                        "name": "custom_proxy",
                        "api": "openai-responses",
                        "baseUrl": "https://api.example.com/v1",
                        "apiKey": "sk-test",
                        "modelId": "llama-3.1-8b",
                        "contextWindow": "131072",
                        "maxTokens": "16384",
                        "reasoning": true
                    }
                }
            }
        });

        repair_openclaw_config_value(&mut config, None);
        normalize_openclaw_model_config(&mut config);

        let provider = &config["models"]["providers"]["custom_proxy"];
        let model = &provider["models"][0];
        assert_eq!(provider["api"], serde_json::json!("openai-responses"));
        assert_eq!(model["id"], serde_json::json!("llama-3.1-8b"));
        assert_eq!(model["api"], serde_json::json!("openai-responses"));
        assert_eq!(model["contextWindow"], serde_json::json!(131072));
        assert_eq!(model["maxTokens"], serde_json::json!(16384));
        assert_eq!(model["reasoning"], serde_json::json!(true));
        assert!(provider.get("maxTokens").is_none());
        assert!(provider.get("contextWindow").is_none());
    }

    #[test]
    fn setup_provider_choice_maps_known_wizard_options() {
        assert_eq!(provider_id_for_key("OpenAI"), Some("openai"));
        assert_eq!(provider_id_for_key("Anthropic"), Some("anthropic"));
        assert_eq!(provider_id_for_key("Unknown"), Some("unknown"));
    }

    #[test]
    fn backup_manifest_accepts_current_app_sources() {
        let manifest = serde_json::json!({
            "app": "openclaw",
            "format": BACKUP_FORMAT,
            "sources": [
                { "folder": ".openclaw", "original_path": "C:\\Users\\demo\\.openclaw" },
                { "folder": "workspace", "original_path": "C:\\Users\\demo\\workspace" }
            ]
        });

        let labels = validate_backup_manifest_value(&manifest, "openclaw").unwrap();

        assert_eq!(
            labels,
            vec![".openclaw".to_string(), "workspace".to_string()]
        );
    }

    #[test]
    fn backup_manifest_rejects_wrong_app() {
        let manifest = serde_json::json!({
            "app": "n8n",
            "format": BACKUP_FORMAT,
            "sources": [
                { "folder": "home_dot_n8n", "original_path": "C:\\Users\\demo\\.n8n" }
            ]
        });

        let err = validate_backup_manifest_value(&manifest, "openclaw").unwrap_err();

        assert!(err.contains("Backup is for"));
    }

    #[test]
    fn backup_manifest_rejects_unknown_source() {
        let manifest = serde_json::json!({
            "app": "openclaw",
            "format": BACKUP_FORMAT,
            "sources": [
                { "folder": ".ssh", "original_path": "C:\\Users\\demo\\.ssh" }
            ]
        });

        let err = validate_backup_manifest_value(&manifest, "openclaw").unwrap_err();

        assert!(err.contains("unknown source"));
    }

    #[test]
    fn append_query_param_adds_gateway_token_to_existing_query() {
        let url = append_query_param(
            "http://127.0.0.1:18789/chat?session=main",
            "token",
            "abc123",
        );

        assert_eq!(url, "http://127.0.0.1:18789/chat?session=main&token=abc123");
    }

    #[test]
    fn append_query_param_does_not_duplicate_existing_token() {
        let url = append_query_param(
            "http://127.0.0.1:18789/chat?session=main&token=old",
            "token",
            "new",
        );

        assert_eq!(url, "http://127.0.0.1:18789/chat?session=main&token=old");
    }

    #[test]
    fn append_query_param_percent_encodes_value() {
        let url = append_query_param("http://127.0.0.1:18789/", "token", "a b+c");

        assert_eq!(url, "http://127.0.0.1:18789/?token=a%20b%2Bc");
    }

    #[test]
    fn repair_config_normalizes_telegram_policy_case() {
        let mut config = serde_json::json!({
            "channels": {
                "telegram": {
                    "dmPolicy": "Open",
                    "groupPolicy": "Allowlist"
                }
            }
        });

        repair_openclaw_config_value(&mut config, None);

        assert_eq!(config["channels"]["telegram"]["dmPolicy"], "open");
        assert_eq!(config["channels"]["telegram"]["groupPolicy"], "allowlist");
    }

    #[test]
    fn repair_config_enables_telegram_plugin_and_drops_missing_token_file() {
        let missing_token_file =
            std::env::temp_dir().join("agents-control-center-test-missing-telegram-token.txt");
        let mut config = serde_json::json!({
            "channels": {
                "telegram": {
                    "enabled": true,
                    "botToken": "123:abc",
                    "tokenFile": path_as_config_string(&missing_token_file)
                }
            },
            "plugins": {
                "entries": {
                    "telegram": {
                        "enabled": false
                    }
                }
            }
        });

        repair_openclaw_config_value(&mut config, None);

        assert!(config["channels"]["telegram"].get("tokenFile").is_none());
        assert_eq!(
            config["plugins"]["entries"]["telegram"]["enabled"],
            serde_json::json!(true)
        );
        assert!(config["plugins"]["entries"]["telegram"]["config"].is_object());
        assert_eq!(
            config["channels"]["telegram"]["groups"]["*"]["requireMention"],
            serde_json::json!(false)
        );
        assert_eq!(
            config["channels"]["telegram"]["streaming"]["mode"],
            serde_json::json!("off")
        );
    }
}
