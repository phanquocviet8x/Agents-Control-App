// Copyright (c) 2026 Vu. All rights reserved.
// Proprietary source. See OWNERSHIP.md at the repository root.

import { startTransition, useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import HelpTabComponent from './HelpTab.jsx';
import qrImage from './assets/qr.png';
import telegramGroupQr from './assets/telegram-group-qr.png';
import { LANGUAGES, LANGUAGE_STORAGE_KEY, thanksLines, translate, translateState } from './i18n.js';
import {
  Play, Globe, Terminal as TerminalIcon, Settings,
  HelpCircle, KeyRound, ScrollText, Heart, Download,
  Trash2, ChevronDown, FolderOpen, X, Minus, Plus, RefreshCw, Send, Copy, BookOpen
} from 'lucide-react';
import './App.css';

const tabs = [
  { key: 'run', label: 'Run', icon: Play },
  { key: 'terminal', label: 'Terminal', icon: TerminalIcon },
  { key: 'setup', label: 'Install', icon: Settings },
  { key: 'help', label: 'Setup', icon: HelpCircle },
  { key: 'api', label: 'API Keys', icon: KeyRound },
  { key: 'logs', label: 'Logs', icon: ScrollText },
  { key: 'thanks', label: 'Thanks', icon: Heart },
];

const setupTools = [
  { key: 'node', name: 'Node.js', versions: ['v24.15.0', 'v24.14.0', 'v22.15.0', 'v22.14.0', 'v20.19.0'] },
  { key: 'openclaw', name: 'OpenClaw', versions: ['2026.4.23', '2026.4.22', '2026.4.21', '2026.4.20', '2026.4.19'] },
  { key: 'claude-code', name: 'Claude Code', versions: ['1.0.20', '1.0.19', '1.0.18', '1.0.17', '1.0.16'] },
  { key: '9router', name: '9router', versions: ['0.4.12', '0.4.11', '0.4.10', '0.4.9', '0.4.8'] },
  { key: 'n8n', name: 'n8n', versions: ['2.18.5', '2.18.4', '2.18.3', '2.18.2', '2.18.1'] },
  { key: 'ngrok', name: 'ngrok', versions: [] },
  { key: 'git', name: 'Git', versions: [] },
  { key: 'python', name: 'Python', versions: [] },
];

const runApps = [
  { key: 'node', name: 'Node.js' },
  { key: 'gateway', name: 'OpenClaw Gateway' },
  { key: 'openclaw', name: 'OpenClaw' },
  { key: 'n8n', name: 'n8n' },
  { key: 'ngrok', name: 'ngrok' },
  { key: 'claude-code', name: 'Claude Code' },
  { key: '9router', name: '9router' },
  { key: 'git', name: 'Git' },
  { key: 'python', name: 'Python' },
];

const models = [
  'Claude Opus 4.6', 'Claude Sonnet 4.6', 'GPT-5.5', 'GPT-5.4', 'o3', 'o4-mini',
  'Gemini 2.5 Pro', 'Gemini 3 Pro', 'Gemini 2.5 Flash', 'Gemini 3 Flash',
  'Llama 4 Scout', 'Llama 4 Maverick', 'DeepSeek R1', 'DeepSeek V3',
  'Grok 3', 'Grok 3 Mini', 'OpenRouter Auto', '9router Custom'
];
const apiTabs = ['AI Model', 'Custom Provider', 'Telegram', 'Google API', 'Ngrok', 'N8N'];
const customProviderTemplate = { name: '', baseUrl: '', apiKey: '', api: 'openai-completions', modelId: '', contextWindow: '', maxTokens: '8192', reasoning: false };
const providerApiOptions = ['openai-completions', 'openai-responses', 'anthropic-messages', 'google-generative-ai'];
const telegramGroupUrl = 'https://t.me/+ZW5oLBnKR39lMmE1';
const backupApps = [
  { key: 'openclaw', name: 'OpenClaw' },
  { key: 'claude-code', name: 'Claude Code' },
  { key: 'n8n', name: 'N8N' },
  { key: 'ngrok', name: 'Ngrok' },
];

const commandRiskRules = [
  { level: 'danger', pattern: /\b(remove-item|rm|del|erase|rmdir|format|reg\s+delete|taskkill)\b/i, message: 'This command can delete files, registry entries, or stop processes.' },
  { level: 'warning', pattern: /\b(winget|npm)\s+(uninstall|remove)|set-executionpolicy|stop-process\b/i, message: 'This command can change installed software or system policy.' },
];
function getCommandRisk(command) { return commandRiskRules.find(rule => rule.pattern.test(command || '')) || null; }
function confirmRiskyCommand(command) { const risk = getCommandRisk(command); return !risk || window.confirm(`${risk.message}\n\nRun anyway?`); }

function App() {
  const [activeTab, setActiveTab] = useState('run');
  const [language, setLanguage] = useState(() => {
    try { return localStorage.getItem(LANGUAGE_STORAGE_KEY) || 'en'; } catch { return 'en'; }
  });
  const [languageModal, setLanguageModal] = useState(false);
  const [telegramModal, setTelegramModal] = useState(false);
  const [appStates, setAppStates] = useState({});
  const [setupInstalled, setSetupInstalled] = useState({});
  const [setupVersions, setSetupVersions] = useState({});
  const [statusChecking, setStatusChecking] = useState(false);
  const [toasts, setToasts] = useState([]);
  const [terminalAutoCmd, setTerminalAutoCmd] = useState('');
  const [terminalAutoSeq, setTerminalAutoSeq] = useState(0);
  const [errorAnalysis, setErrorAnalysis] = useState(null);
  const t = useCallback((text) => translate(language, text), [language]);
  const setAppLanguage = (next) => {
    setLanguageModal(false);
    if (next === language) return;
    try { localStorage.setItem(LANGUAGE_STORAGE_KEY, next); } catch { /* ignore */ }
    window.setTimeout(() => {
      startTransition(() => setLanguage(next));
    }, 0);
  };
  const toast = useCallback((text) => { const id = Date.now() + Math.random(); setToasts(t => [...t, { id, text }]); setTimeout(() => setToasts(t => t.filter(x => x.id !== id)), 3200); }, []);
  const loadAppStatus = useCallback(async () => {
    const res = await invoke('app_statuses');
    const next = typeof res === 'object' ? res : {};
    setAppStates(next);
    return next;
  }, []);
  const loadSetupStatus = useCallback(async () => {
    const [installedRes, versionsRes] = await Promise.all([invoke('check_tools'), invoke('check_versions')]);
    const installed = typeof installedRes === 'object' ? installedRes : {};
    const versions = typeof versionsRes === 'object' ? versionsRes : {};
    setSetupInstalled(installed);
    setSetupVersions(versions);
    return { installed, versions };
  }, []);
  const refreshStatusWithModal = useCallback(async ({ includeSetup = false } = {}) => {
    setStatusChecking(true);
    try {
      if (includeSetup) {
        await Promise.all([loadAppStatus(), loadSetupStatus()]);
      } else {
        await loadAppStatus();
      }
    } catch (e) {
      toast(String(e));
    } finally {
      setStatusChecking(false);
    }
  }, [loadAppStatus, loadSetupStatus, toast]);
  const refreshSetupStatusWithModal = useCallback(async () => {
    setStatusChecking(true);
    try {
      await loadSetupStatus();
    } catch (e) {
      toast(String(e));
    } finally {
      setStatusChecking(false);
    }
  }, [loadSetupStatus, toast]);
  const runAction = useCallback(async (action) => { try { const res = await invoke('run_action', { action }); toast(String(res || action)); return res; } catch (e) { toast(String(e)); throw e; } }, [toast]);
  const markActionState = useCallback((action) => {
    const nextState = action.includes('stop') ? 'Stopped' : 'Running';
    const patch = {};
    if (action.startsWith('gateway-') || action === 'webui-run') patch.gateway = nextState;
    else if (action.startsWith('n8n-ngrok-')) {
      patch.n8n = nextState;
      patch.ngrok = nextState;
    } else if (action.startsWith('n8n-')) patch.n8n = nextState;
    else if (action.startsWith('ngrok-')) patch.ngrok = nextState;
    else if (action.startsWith('claude-code-')) patch['claude-code'] = nextState;
    else if (action.startsWith('9router-')) patch['9router'] = nextState;
    if (Object.keys(patch).length) setAppStates(current => ({ ...current, ...patch }));
  }, []);
  useEffect(() => { const id = setTimeout(() => refreshStatusWithModal({ includeSetup: true }), 0); return () => clearTimeout(id); }, [refreshStatusWithModal]);
  const openTerminalWithCommand = (command) => {
    const cmd = (command || '').trim();
    if (!cmd) return;
    setTerminalAutoCmd(cmd);
    setTerminalAutoSeq(v => v + 1);
    setActiveTab('terminal');
  };
  const clearTerminalAutoRun = useCallback(() => {
    setTerminalAutoCmd('');
    setTerminalAutoSeq(0);
  }, []);

  const analyzeTerminalError = async ({ command, output }) => {
    const text = String(output || '');
    const cmd = String(command || '');
    const lower = (cmd + '\\n' + text).toLowerCase();
    const hasError = /error|exception|not recognized|not found|cannot find|access is denied|eacces|eaddrinuse|address already in use|econnrefused|401|403|404|err_ngrok|failed/i.test(text);
    if (!hasError) return;

    setErrorAnalysis({ phase: 'loading' });
    const started = Date.now();
    const waitMin = async () => {
      const left = 5000 - (Date.now() - started);
      if (left > 0) await new Promise(r => setTimeout(r, left));
    };

    const make = (title, cause, detail, suggestion, confidence = 'High', source = 'Local analysis') => ({
      phase: 'done', title, cause, detail, suggestion, confidence, source
    });

    let result = null;
    if (lower.includes('commandnotfoundexception') || lower.includes('not recognized as the name') || (lower.includes('the term') && lower.includes('is not recognized'))) {
      const tool = (cmd.match(/\b(openclaw|n8n|ngrok|claude|9router|git|python|node)\b/i)?.[1] || 'app');
      const nameMap = { openclaw: 'OpenClaw', n8n: 'n8n', ngrok: 'ngrok', claude: 'Claude Code', '9router': '9router', git: 'Git', python: 'Python', node: 'Node.js' };
      const appName = nameMap[String(tool).toLowerCase()] || tool;
      result = make(
        `${appName} is not ready to run`,
        `${appName} is not installed or not in PATH.`,
        `PowerShell returned CommandNotFoundException, which means Windows cannot find the ${tool} command.`,
        `Go to the Install tab to install ${appName}. If it is already installed, restart the app or check PATH.`
      );
    } else if (lower.includes('access is denied') || lower.includes('eacces') || lower.includes('permission denied')) {
      result = make('Permission denied', 'The command is blocked by Windows or by system permissions.', 'The terminal returned a permission error such as Access is denied / EACCES.', 'Try running the app as Administrator or check the related file/folder permissions.');
    } else if (lower.includes('eaddrinuse') || lower.includes('address already in use') || lower.includes('only one usage of each socket address')) {
      result = make('Port already in use', 'Another application is already using the required port.', 'The error shows that the address/port is already being used by another process.', 'Stop the process that is using the port or change the configured port, then try again.');
    } else if (lower.includes('econnrefused') || lower.includes('connection refused')) {
      result = make('Cannot connect to service', 'The target service is not running or the port/address is wrong.', 'ECONNREFUSED means a connection was attempted but the target side refused it.', 'Check whether the service is running and whether the port is correct, then try again.');
    } else if (lower.includes('err_ngrok')) {
      const code = (text.match(/ERR_NGROK_[0-9]+/i) || ['ERR_NGROK'])[0];
      result = make(`Ngrok Error ${code}`, 'Ngrok returned a configuration or account error.', `The terminal detected error code ${code}.`, 'Check the authtoken, domain, and port in API Keys &gt; Ngrok. If the code is still unclear, use Search Error.', 'Medium');
    } else if (lower.includes('401') || lower.includes('unauthorized')) {
      result = make('Missing or invalid authentication', 'The API key/token is invalid or has not been configured.', 'The terminal returned 401 Unauthorized.', 'Check the API key/token again in the API Keys tab.');
    } else if (lower.includes('403') || lower.includes('forbidden')) {
      result = make('Access forbidden', 'The account/token exists but does not have enough permissions.', 'The terminal returned 403 Forbidden.', 'Check the account permissions, token scope, or access rights to the resource.');
    } else if (lower.includes('404') || lower.includes('not found')) {
      result = make('Resource not found', 'The path, endpoint, package, or file does not exist.', 'The terminal returned 404 / Not Found.', 'Check the URL, package name, file path, or related configuration again.', 'Medium');
    } else if (lower.includes('cannot find module') || lower.includes('module not found')) {
      result = make('Missing package/module', 'The application is missing a required dependency.', 'The terminal reported Cannot find module / Module not found.', 'Reinstall the dependency or reinstall the related tool in the Setup tab.');
    }

    if (!result) {
      result = make(
        'Unknown error - more research needed',
        'The app could not confidently identify this error using local analysis.',
        'This may be a tool-specific or uncommon error.',
        'Local analysis was prioritized first. If needed, click Search Error to open a browser search for the error details.',
        'Low',
        'Local analysis; web/browser may be needed for further verification'
      );
    }

    await waitMin();
    setErrorAnalysis(result);
  };
  return <div className="app-shell">
    <Titlebar />
    <div className="app-body">
      <aside className="sidebar">
        <div className="nav-list">{tabs.map(({ key, label, icon: Icon }) => <button key={key} className={activeTab === key ? 'active' : ''} onClick={() => setActiveTab(key)}><Icon size={18} />{t(label)}</button>)}</div>
        <div className="sidebar-tools"><button className="sidebar-language-btn" onClick={() => setLanguageModal(true)}><Globe size={16} />{t('Language')}</button><button className="sidebar-language-btn" onClick={() => setTelegramModal(true)}><Send size={16} />{t('Telegram Group')}</button></div>
        <div className="sidebar-copyright" aria-label="Dự Án Vì Cộng Đồng AI Agents Việt Nam"><span>Dự Án</span><span>Vì Cộng Đồng AI Agents</span><span>Việt Nam</span></div>
      </aside>
      <main className="content">
        <div className="content-scroll">
          {activeTab === 'run' && <RunTab appStates={appStates} refreshAppStatus={() => refreshStatusWithModal()} runAction={runAction} markActionState={markActionState} toast={toast} setActiveTab={setActiveTab} openTerminalWithCommand={openTerminalWithCommand} language={language} t={t} />}
          {activeTab === 'terminal' && <TerminalTab toast={toast} autoRunCommand={terminalAutoCmd} autoRunSeq={terminalAutoSeq} onAutoRunConsumed={clearTerminalAutoRun} onAnalyzeError={analyzeTerminalError} t={t} />}
          {activeTab === 'setup' && <SetupTab installed={setupInstalled} versions={setupVersions} refreshSetupStatus={refreshSetupStatusWithModal} loadSetupStatus={loadSetupStatus} toast={toast} setActiveTab={setActiveTab} openTerminalWithCommand={openTerminalWithCommand} language={language} t={t} />}
          {activeTab === 'help' && <HelpTabComponent runAction={runAction} Panel={Panel} language={language} />}
          {activeTab === 'api' && <ApiTab toast={toast} t={t} />}
          {activeTab === 'logs' && <LogsTab toast={toast} t={t} />}
          {activeTab === 'thanks' && <ThanksTab language={language} t={t} />}
        </div>
      </main>
    </div>
    <div className="toasts">{toasts.map(t => <div className="toast" key={t.id}>{t.text}</div>)}</div>
    {statusChecking && <StatusCheckingModal t={t} />}
    {errorAnalysis && <ErrorAnalysisModal data={errorAnalysis} onClose={() => setErrorAnalysis(null)} t={t} />}
    {languageModal && <LanguageModal language={language} onSelect={setAppLanguage} onClose={() => setLanguageModal(false)} t={t} />}
    {telegramModal && <TelegramGroupModal onClose={() => setTelegramModal(false)} t={t} />}
  </div>;
}
function Titlebar() { const win = getCurrentWindow(); const safeWindowAction = async (fn) => { try { await fn(); } catch (e) { console.error(e); } }; return <div className="titlebar"><div className="titlebar-brand" data-tauri-drag-region><Globe size={22} /> <span className="brand-name">Agents Setup Center</span><span className="brand-version">v2026.0.2</span></div><div className="window-controls"><button type="button" className="window-btn" onClick={() => safeWindowAction(() => win.minimize())}><Minus size={14} /></button><button type="button" className="window-btn close" onClick={() => safeWindowAction(() => win.close())}><X size={14} /></button></div></div>; }
function Panel({ title, children }) { return <section className="panel"><h2 className="panel-title">{title}</h2><div className="panel-body">{children}</div></section>; }
function AppStateBadge({ state, language }) { const s = state || '-'; return <strong className={'app-state ' + s.toLowerCase().replaceAll(' ', '-')}>{translateState(language, s)}</strong>; }
function StatusCheckingModal({ t }) { return <div className="modal-backdrop"><div className="modal confirm-modal busy-modal"><div className="install-spinner"><svg viewBox="0 0 50 50" width="64" height="64"><circle cx="25" cy="25" r="20" fill="none" stroke="rgba(108,99,255,0.15)" strokeWidth="5" /><circle cx="25" cy="25" r="20" fill="none" stroke="#00d4aa" strokeWidth="5" strokeLinecap="round" strokeDasharray="90 150" className="spinner-arc" /></svg></div><p className="busy-title">{t('Please Wait')}</p><p className="busy-subtitle">{t('Checking status...')}</p></div></div>; }
function BusyModal({ phase, text, result, onOk, onCheckError, t, autoCloseMs = 1100 }) {
  useEffect(() => {
    if (phase !== 'done' || !autoCloseMs || !onOk) return undefined;
    const id = window.setTimeout(onOk, autoCloseMs);
    return () => window.clearTimeout(id);
  }, [autoCloseMs, onOk, phase]);
  return <div className="modal-backdrop"><div className="modal confirm-modal busy-modal">{phase === 'running' ? <><div className="install-spinner"><svg viewBox="0 0 50 50" width="64" height="64"><circle cx="25" cy="25" r="20" fill="none" stroke="rgba(108,99,255,0.15)" strokeWidth="5" /><circle cx="25" cy="25" r="20" fill="none" stroke="#00d4aa" strokeWidth="5" strokeLinecap="round" strokeDasharray="90 150" className="spinner-arc" /></svg></div><p className="busy-text">{text}</p></> : <><p className="busy-icon">{phase === 'done' ? 'OK' : 'ERROR'}</p><p className="busy-result">{result}</p><div className="ok-center"><button className="btn primary" onClick={onOk}>{t('OK')}</button>{phase === 'error' && onCheckError && <button className="btn danger" style={{marginLeft:10}} onClick={onCheckError}>{t('Check Error')}</button>}</div></>}</div></div>;
}
function ErrorAnalysisModal({ data, onClose, t }) {
  if (data?.phase === 'loading') {
    return <div className="modal-backdrop"><div className="modal confirm-modal busy-modal"><div className="install-spinner"><svg viewBox="0 0 50 50" width="64" height="64"><circle cx="25" cy="25" r="20" fill="none" stroke="rgba(108,99,255,0.15)" strokeWidth="5" /><circle cx="25" cy="25" r="20" fill="none" stroke="#00d4aa" strokeWidth="5" strokeLinecap="round" strokeDasharray="90 150" className="spinner-arc" /></svg></div><p className="busy-text">{t('Analyzing Error...')}</p></div></div>;
  }
  const q = encodeURIComponent(`${data?.title || ''} ${data?.detail || ''}`.trim());
  return <div className="modal-backdrop"><div className="modal confirm-modal busy-modal"><p className="busy-icon">?</p><h3 style={{marginTop:0}}>{data?.title || t('Error Analysis')}</h3><p className="busy-result"><b>{t('Cause')}:</b> {data?.cause}</p><p className="busy-result"><b>{t('Detail')}:</b> {data?.detail}</p><p className="busy-result"><b>{t('Suggestion')}:</b> {data?.suggestion}</p><p className="busy-result"><b>{t('Confidence')}:</b> {data?.confidence} - <b>{t('Source')}:</b> {data?.source}</p><div className="ok-center"><button className="btn primary" onClick={onClose}>{t('OK')}</button>{data?.confidence !== 'High' && <button className="btn secondary" style={{marginLeft:10}} onClick={() => shellOpen(`https://www.google.com/search?q=${q}`).catch(console.error)}>{t('Search Error')}</button>}</div></div></div>;
}

function LanguageModal({ language, onSelect, onClose, t }) {
  return <div className="modal-backdrop"><div className="modal confirm-modal language-modal"><h3>{t('Choose Language')}</h3><div className="language-options">{LANGUAGES.map(item => <button key={item.key} className={'language-option ' + (language === item.key ? 'active' : '')} onClick={() => onSelect(item.key)}><span>{t(item.label)}</span><small>{item.key === 'en' ? t('Use the current English interface.') : t('Translate the interface to Vietnamese.')}</small></button>)}</div><div className="ok-center"><button className="btn secondary" onClick={onClose}>{t('Cancel')}</button></div></div></div>;
}

function TelegramGroupModal({ onClose, t }) {
  const [copied, setCopied] = useState(false);
  const copyLink = async () => {
    try {
      await navigator.clipboard.writeText(telegramGroupUrl);
    } catch {
      const el = document.createElement('textarea');
      el.value = telegramGroupUrl;
      el.style.position = 'fixed';
      el.style.opacity = '0';
      document.body.appendChild(el);
      el.select();
      document.execCommand('copy');
      document.body.removeChild(el);
    }
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  };
  const joinGroup = async () => {
    try { await shellOpen(telegramGroupUrl); } catch (e) { console.error(e); }
  };
  return <div className="modal-backdrop"><div className="modal confirm-modal telegram-group-modal"><h3>{t('Telegram Group')}</h3><img className="telegram-group-qr" src={telegramGroupQr} alt={t('Telegram Group QR')} /><div className="telegram-copy-row"><input readOnly value={telegramGroupUrl} aria-label={t('Telegram Group Link')} /><button className="telegram-copy-btn" onClick={copyLink} title={t('Copy Link')}><Copy size={16} /></button></div>{copied && <p className="telegram-copy-status">{t('Copied!')}</p>}<div className="ok-center telegram-group-actions"><button className="btn primary" onClick={joinGroup}><Send size={15} />{t('Join Group')}</button><button className="btn secondary" onClick={onClose}>{t('Close')}</button></div></div></div>;
}

function RunTab({ appStates, refreshAppStatus, runAction, markActionState, toast, openTerminalWithCommand, language, t }) {
  const [n8nMode, setN8nMode] = useState('only');
  const [busy, setBusy] = useState(null);
  const doRefresh = async () => { await refreshAppStatus(); };
  const getDiagCommand = (action, result = '') => {
    const key = (action || '').toLowerCase();
    if (key.includes('gateway') || key.includes('webui') || key.includes('openclaw')) return 'openclaw --version; openclaw gateway status';
    if (key.includes('n8n-ngrok')) return 'n8n --version; ngrok version';
    if (key.includes('n8n')) return 'n8n --version';
    if (key.includes('ngrok')) return 'ngrok version';
    if (key.includes('claude')) return 'claude --version';
    if (key.includes('9router')) return '9router --version';
    const msg = String(result || '').toLowerCase();
    if (msg.includes('openclaw')) return 'openclaw --version; openclaw gateway status';
    if (msg.includes('n8n') && msg.includes('ngrok')) return 'n8n --version; ngrok version';
    if (msg.includes('n8n')) return 'n8n --version';
    if (msg.includes('ngrok')) return 'ngrok version';
    if (msg.includes('claude')) return 'claude --version';
    if (msg.includes('9router')) return '9router --version';
    return 'node --version';
  };
  const doRun = async (action) => { setBusy({ phase: 'running', text: action.includes('stop') ? t('Stopping Process...') : t('Starting...'), result: '' }); try { await runAction(action); markActionState?.(action); setBusy({ phase: 'done', result: action.includes('stop') ? t('Process Stopped!') : t('Started Successfully!') }); window.setTimeout(() => setBusy(current => current?.phase === 'done' ? null : current), 900); } catch (e) { setBusy({ phase: 'error', result: String(e).substring(0, 200), action }); } };
  return <>
    {busy && <BusyModal phase={busy.phase} text={busy.text} result={busy.result} t={t} onOk={() => setBusy(null)} onCheckError={() => { const cmd = getDiagCommand(busy.action, busy.result); setBusy(null); openTerminalWithCommand?.(cmd); }} />}
    <div className="page-grid run-grid">
      <Panel title={<span className="run-panel-title">OpenClaw</span>}><div className="button-row run-actions"><button className="btn secondary" onClick={() => doRun('gateway-run')}>{t('Run')}</button><button className="btn secondary" onClick={() => doRun('gateway-stop')}>{t('Stop')}</button><button className="btn secondary" onClick={() => doRun('gateway-restart')}>{t('Restart')}</button><button className="btn secondary" onClick={() => doRun('webui-run')}>{t('Open Dashboard')}</button></div></Panel>
      <Panel title={<span className="run-panel-title">N8N</span>}><div className="subtabs" style={{ justifyContent: 'center', marginBottom: 4 }}><button className={n8nMode === 'only' ? 'active' : ''} onClick={() => setN8nMode('only')}>{t('N8N Only')}</button><button className={n8nMode === 'ngrok' ? 'active' : ''} onClick={() => setN8nMode('ngrok')}>{t('N8N with Ngrok')}</button></div>{n8nMode === 'only' && <div className="button-row run-actions"><button className="btn secondary" onClick={() => doRun('n8n-run')}>{t('Run')}</button><button className="btn secondary" onClick={() => doRun('n8n-stop')}>{t('Stop')}</button></div>}{n8nMode === 'ngrok' && <div className="button-row run-actions"><button className="btn secondary" onClick={async () => { try { const cfg = await invoke('load_settings'); if (!cfg?.ngrok?.authtoken || !cfg?.ngrok?.domain || !cfg?.ngrok?.port) { toast(t('Configure Ngrok (authtoken, domain, port) in API Keys tab first')); return; } doRun('n8n-ngrok-run'); } catch (e) { toast(String(e)); } }}>{t('Run')}</button><button className="btn secondary" onClick={() => doRun('n8n-ngrok-stop')}>{t('Stop')}</button></div>}</Panel>
      <Panel title={<span className="run-panel-title">Claude Code</span>}><div className="button-row run-actions"><button className="btn secondary" onClick={() => doRun('claude-code-run')}>{t('Run')}</button><button className="btn secondary" onClick={() => doRun('claude-code-stop')}>{t('Stop')}</button></div></Panel>
      <Panel title={<span className="run-panel-title">9router</span>}><div className="button-row run-actions"><button className="btn secondary" onClick={() => doRun('9router-run')}>{t('Run')}</button><button className="btn secondary" onClick={() => doRun('9router-stop')}>{t('Stop')}</button></div></Panel>
    </div>
    <div style={{ marginTop: 10 }}><Panel title={<span className="versions-header">{t('App Activity Status')}<button className="btn secondary btn-sm" onClick={doRefresh}>{t('Refresh')}</button></span>}><div className="installed-list">{runApps.map(app => <div className="installed-item" key={app.key}><span>{app.name}</span><AppStateBadge state={appStates[app.key]} language={language} /></div>)}</div></Panel></div>
  </>;
}

function TerminalTab({ toast, autoRunCommand, autoRunSeq, onAutoRunConsumed, onAnalyzeError, t }) {
  const [lines, setLines] = useState([]);
  const [cmd, setCmd] = useState('');
  const [busy, setBusy] = useState(false);
  const endRef = useRef(null);
  const handledAutoSeqRef = useRef(0);
  useEffect(() => endRef.current?.scrollIntoView({ behavior: 'smooth' }), [lines]);
  const runCommand = useCallback(async (command) => {
    const current = (command || '').trim();
    if (!current) return;
    if (!confirmRiskyCommand(current)) return;
    setBusy(true);
    setLines(l => [...l, 'PS> ' + current]);
    try {
      const out = await invoke('run_terminal_cmd', { cmd: current });
      const text = String(out || '');
      setLines(l => [...l, text]);
      onAnalyzeError?.({ command: current, output: text });
    }
    catch (err) {
      const text = 'ERROR: ' + err;
      toast(String(err));
      setLines(l => [...l, text]);
      onAnalyzeError?.({ command: current, output: text });
    }
    finally { setBusy(false); }
  }, [onAnalyzeError, toast]);
  useEffect(() => {
    if (!autoRunSeq) return;
    if (handledAutoSeqRef.current === autoRunSeq) return;
    const current = (autoRunCommand || '').trim();
    if (!current || busy) return;
    handledAutoSeqRef.current = autoRunSeq;
    const id = setTimeout(() => { onAutoRunConsumed?.(); runCommand(current); }, 0);
    return () => clearTimeout(id);
  }, [autoRunCommand, autoRunSeq, busy, onAutoRunConsumed, runCommand]);
  const submit = async (e) => { e.preventDefault(); if (!cmd.trim() || busy) return; const current = cmd; setCmd(''); await runCommand(current); };
  return <div className="terminal-page"><div className="terminal-screen">{lines.map((l, i) => <div className="terminal-line" key={i}>{l}</div>)}<div ref={endRef} /></div><form className="terminal-input-row" onSubmit={submit}><span className="terminal-prompt">PS&gt;</span><input value={cmd} onChange={e => setCmd(e.target.value)} autoFocus disabled={busy} placeholder={busy ? t('Command is running...') : t('Type a command for any app...')} /></form></div>;
}

function SetupTab({ installed = {}, versions = {}, refreshSetupStatus, toast, openTerminalWithCommand, language, t }) {
  const [selected, setSelected] = useState(Object.fromEntries(setupTools.map(tool => [tool.key, tool.versions[0]])));
  const [openDrop, setOpenDrop] = useState(null);
  const [busy, setBusy] = useState({});
  const [confirm, setConfirm] = useState(null);
  const [openclawSetup, setOpenclawSetup] = useState(null);
  const [installing, setInstalling] = useState(null);
  const [installResult, setInstallResult] = useState('');
  const [lastErrorCmd, setLastErrorCmd] = useState('');
  const [guideOpen, setGuideOpen] = useState(false);
  const [backupFlow, setBackupFlow] = useState(null);

  const runSetup = async (tool, kind) => {
    setBusy(b => ({ ...b, [tool.key]: true }));
    setInstalling({ phase: 'running', kind });
    setInstallResult('');
    setLastErrorCmd('');
    try {
      const installVersion = selected[tool.key] || '';
      const action = kind === 'install' ? `install-${tool.key}${installVersion ? `-${installVersion}` : ''}` : kind === 'update' ? `update-${tool.key}` : `uninstall-${tool.key}`;
      const res = await invoke('run_action', { action });
      if (kind === 'install' && tool.key === 'openclaw' && openclawSetup) {
        await invoke('setup_openclaw_files', { openclawPath: '', workspacePath: openclawSetup.workspacePath, openaiApiKey: openclawSetup.openaiApiKey || '', apiKeyProvider: openclawSetup.apiProvider || 'openai' });
      }
      setInstallResult(String(res || (kind === 'uninstall' ? t('Uninstalled!') : t('Installed Successfully!'))));
      setInstalling({ phase: 'done', kind });
    } catch (e) {
      setInstallResult(String(e).substring(0, 240));
      setLastErrorCmd(tool.key);
      setInstalling({ phase: 'error', kind });
    } finally {
      setBusy(b => ({ ...b, [tool.key]: false }));
      setConfirm(null);
      setOpenclawSetup(null);
    }
  };

  const startOpenclawInstall = (tool) => setOpenclawSetup({ step: 1, workspacePath: '', apiProvider: 'openai', openaiApiKey: '', tool });
  const handleOpenclawBrowse = async () => { const dir = await dialogOpen({ directory: true, title: t('Choose Workspace folder') }); if (dir) setOpenclawSetup(s => ({ ...s, workspacePath: dir })); };
  const handleOpenclawConfirm = () => { if (!openclawSetup.workspacePath) { toast(t('Please select workspace folder')); return; } runSetup(openclawSetup.tool, 'install'); };
  const handleAddPath = async (tool) => {
    setBusy(b => ({ ...b, [tool.key]: true }));
    setInstalling({ phase: 'running', kind: 'add-path' });
    setInstallResult('');
    setLastErrorCmd(tool.key);
    try {
      const res = await invoke('add_to_path', { tool: tool.key });
      setInstallResult(String(res || t('Done')));
      setInstalling({ phase: 'done', kind: 'add-path' });
    } catch (e) {
      setInstallResult(String(e).substring(0, 240));
      setInstalling({ phase: 'error', kind: 'add-path' });
    } finally {
      setBusy(b => ({ ...b, [tool.key]: false }));
    }
  };
  const chooseBackupFolder = async () => { const dir = await dialogOpen({ directory: true, title: t('Choose backup folder') }); if (dir) setBackupFlow(s => ({ ...s, path: dir })); };
  const chooseRestoreZip = async () => { const file = await dialogOpen({ multiple: false, directory: false, title: t('Choose backup zip file'), filters: [{ name: 'ZIP', extensions: ['zip'] }] }); if (file) setBackupFlow(s => ({ ...s, path: Array.isArray(file) ? file[0] : file })); };

  const runBackupRestore = async (forceRestore = false) => {
    if (!backupFlow?.app?.key || !backupFlow?.path) return;
    if (backupFlow.mode === 'restore' && !forceRestore) {
      setBackupFlow(s => ({ ...s, confirm: true }));
      return;
    }
    const mode = backupFlow.mode;
    const appKey = backupFlow.app.key;
    const path = backupFlow.path;
    setBackupFlow(s => ({ ...s, phase: 'running', confirm: false }));
    try {
      const res = mode === 'backup'
        ? await invoke('backup_app_data', { app: appKey, destinationDir: path })
        : await invoke('restore_app_data', { app: appKey, zipFile: path });
      setBackupFlow(s => ({ ...s, phase: 'done', result: String(res || t('Done')) }));
    } catch (e) {
      setBackupFlow(s => ({ ...s, phase: 'error', result: String(e) }));
    }
  };

  const busyText = installing?.kind === 'add-path' ? t('Adding PATH...')
    : installing?.kind === 'uninstall' ? t('Uninstalling, please wait...')
    : installing?.kind === 'update' ? t('Updating, please wait...')
    : t('Installing, please wait...');
  const errorCommands = { openclaw: 'openclaw --version; openclaw gateway status', n8n: 'n8n --version', ngrok: 'ngrok version', 'claude-code': 'claude --version', '9router': '9router --version', git: 'git --version', python: 'python --version', node: 'node --version' };

  return <>
    <div className="setup-grid">{setupTools.map(tool => {
      const isInstalled = !!installed[tool.key];
      const status = isInstalled ? 'Installed' : 'Not Installed';
      const version = versions[tool.key] && versions[tool.key] !== 'Not Installed' ? versions[tool.key] : t('Not Installed');
      return <div className="tool-card" key={tool.key}><div className="tool-card-head"><div><h3>{tool.name}</h3><div className="tool-status-line"><span className={'app-state ' + status.toLowerCase().replaceAll(' ', '-')}>{t(status)}</span><small>{version}</small></div></div>{busy[tool.key] ? <span className="progress-ring" /> : null}</div><div className="tool-main-row"><div className="install-stack"><div style={{display:'flex',gap:6}}><button className="btn primary" onClick={() => tool.key === 'openclaw' ? startOpenclawInstall(tool) : setConfirm({ tool, kind: 'install', text: `${t('Install')} ${tool.name}${selected[tool.key] ? ' ' + selected[tool.key] : ''}?` })}><Download size={15} />{t('Install')}</button><button className="btn" style={{background:'#2196F3',color:'#fff',border:'none'}} onClick={() => handleAddPath(tool)}><Plus size={15} />{t('Add PATH')}</button></div>{tool.versions.length > 0 && <div className="dropdown-wrap"><button className="btn secondary" onClick={() => setOpenDrop(openDrop === tool.key ? null : tool.key)}>{selected[tool.key]} <ChevronDown size={14} /></button>{openDrop === tool.key && <div className="dropdown-menu">{tool.versions.map(v => <button key={v} onClick={() => { setSelected(s => ({ ...s, [tool.key]: v })); setOpenDrop(null); }}>{v}</button>)}</div>}</div>}</div><div className="tool-card-actions"><button className="btn warning" onClick={() => setConfirm({ tool, kind: 'update', text: `${t('Update')} ${tool.name}?` })}><RefreshCw size={15} />{t('Update')}</button><button className="btn danger" onClick={() => setConfirm({ tool, kind: 'uninstall', text: `${t('Uninstall')} ${tool.name}?` })}><Trash2 size={15} />{t('Uninstall')}</button></div></div></div>;
    })}</div>
    {confirm && <div className="modal-backdrop"><div className="modal confirm-modal"><h3>{confirm.text}</h3><div className="button-row"><button className="btn primary" onClick={() => runSetup(confirm.tool, confirm.kind)}>{t('Yes')}</button><button className="btn secondary" onClick={() => setConfirm(null)}>{t('No')}</button></div></div></div>}
{openclawSetup && <div className="modal-backdrop"><div className="modal confirm-modal" style={{ minWidth: 420 }}><h3>{t('Install OpenClaw')} {selected.openclaw}</h3><p style={{ color: '#8888aa', fontSize: '0.85rem', marginBottom: 16 }}>{t('OpenClaw will use the default config folder.')}</p><div style={{ marginBottom: 12 }}><label style={{ fontSize: '0.85rem', color: '#aaa' }}>{t('Workspace Folder')}</label><div style={{ display: 'flex', gap: 8, marginTop: 4 }}><input readOnly value={openclawSetup.workspacePath} placeholder={t('Select folder...')} style={{ flex: 1, padding: '8px 12px', borderRadius: 8, border: '1px solid rgba(108,99,255,0.3)', background: 'rgba(0,0,0,0.3)', color: '#e0e0f0', fontSize: '0.85rem' }} /><button className="btn secondary" onClick={handleOpenclawBrowse}>{t('Browse')}</button></div></div><div style={{ marginBottom: 12 }}><label style={{ fontSize: '0.85rem', color: '#aaa' }}>{t('API Provider')}</label><select value={openclawSetup.apiProvider || 'openai'} onChange={e => setOpenclawSetup(s => ({ ...s, apiProvider: e.target.value, openaiApiKey: e.target.value === 'unknown' ? '' : s.openaiApiKey }))} style={{ width: '100%', marginTop: 4, padding: '8px 12px', borderRadius: 8, border: '1px solid rgba(108,99,255,0.3)', background: 'rgba(0,0,0,0.3)', color: '#e0e0f0', fontSize: '0.85rem' }}><option value="openai">OpenAI</option><option value="anthropic">Anthropic</option><option value="unknown">Unknown</option></select></div><div style={{ marginBottom: 20 }}><label style={{ fontSize: '0.85rem', color: '#aaa' }}>{t('API Key')} ({t('optional')})</label><input type="password" disabled={(openclawSetup.apiProvider || 'openai') === 'unknown'} value={openclawSetup.openaiApiKey || ''} onChange={e => setOpenclawSetup(s => ({ ...s, openaiApiKey: e.target.value }))} placeholder={(openclawSetup.apiProvider || 'openai') === 'anthropic' ? 'sk-ant-...' : 'sk-...'} style={{ width: '100%', marginTop: 4, padding: '8px 12px', borderRadius: 8, border: '1px solid rgba(108,99,255,0.3)', background: 'rgba(0,0,0,0.3)', color: '#e0e0f0', fontSize: '0.85rem', opacity: (openclawSetup.apiProvider || 'openai') === 'unknown' ? 0.55 : 1 }} /></div><div className="button-row"><button className="btn primary" onClick={handleOpenclawConfirm} disabled={!openclawSetup.workspacePath}>{t('Install')}</button><button className="btn secondary" onClick={() => setOpenclawSetup(null)}>{t('Cancel')}</button></div></div></div>}
    {installing && <BusyModal phase={installing.phase} text={busyText} result={installResult} t={t} onOk={() => setInstalling(null)} onCheckError={() => { const tool = (lastErrorCmd || '').toLowerCase(); const cmd = errorCommands[tool] || 'node --version'; setInstalling(null); openTerminalWithCommand?.(cmd); }} />}
    {guideOpen && <GuideModalV2 language={language} onClose={() => setGuideOpen(false)} t={t} />}
    {backupFlow && <BackupRestoreModal flow={backupFlow} setFlow={setBackupFlow} chooseBackupFolder={chooseBackupFolder} chooseRestoreZip={chooseRestoreZip} runBackupRestore={runBackupRestore} t={t} />}
    <div className="setup-bottom-actions"><button className="btn secondary" onClick={() => setBackupFlow({ step: 'mode', mode: '', app: null, path: '', phase: 'idle', result: '' })}><FolderOpen size={15} />{t('Backup/Restore')}</button><button className="btn secondary" onClick={refreshSetupStatus}><RefreshCw size={15} />{t('Refresh')}</button><button className="btn secondary" onClick={() => setGuideOpen(true)}><BookOpen size={15} />{t('Guide')}</button></div>
  </>;
}

function BackupRestoreModal({ flow, setFlow, chooseBackupFolder, chooseRestoreZip, runBackupRestore, t }) {
  if (flow.phase === 'running') return <BusyModal phase="running" text={t('Please Wait...')} result="" t={t} onOk={() => {}} />;
  if (flow.phase === 'done' || flow.phase === 'error') return <BusyModal phase={flow.phase === 'done' ? 'done' : 'error'} text="" result={flow.result} t={t} onOk={() => setFlow(null)} />;
  if (flow.confirm) {
    return <div className="modal-backdrop"><div className="modal confirm-modal"><h3>{t('Are you sure you want to restore?')}</h3><p className="restore-warning">{t('This process will replace old files for the selected app.')}</p><div className="button-row"><button className="btn primary" onClick={() => runBackupRestore(true)}>{t('OK')}</button><button className="btn secondary" onClick={() => setFlow(s => ({ ...s, confirm: false }))}>{t('Cancel')}</button></div></div></div>;
  }
  if (flow.step === 'mode') {
    return <div className="modal-backdrop"><div className="modal confirm-modal backup-modal"><h3>{t('Backup/Restore')}</h3><div className="backup-mode-grid"><button className="backup-choice" onClick={() => setFlow(s => ({ ...s, step: 'app', mode: 'backup' }))}><Download size={22} /><strong>{t('Backup')}</strong></button><button className="backup-choice" onClick={() => setFlow(s => ({ ...s, step: 'app', mode: 'restore' }))}><RefreshCw size={22} /><strong>{t('Restore')}</strong></button></div><div className="ok-center"><button className="btn secondary" onClick={() => setFlow(null)}>{t('Cancel')}</button></div></div></div>;
  }
  if (flow.step === 'app') {
    return <div className="modal-backdrop"><div className="modal confirm-modal backup-modal"><h3>{flow.mode === 'backup' ? t('Choose app to backup') : t('Choose app to restore')}</h3><div className="backup-app-grid">{backupApps.map(app => <button key={app.key} className="backup-choice" onClick={() => setFlow(s => ({ ...s, step: 'path', app }))}><FolderOpen size={20} /><strong>{app.name}</strong></button>)}</div><div className="ok-center"><button className="btn secondary" onClick={() => setFlow(s => ({ ...s, step: 'mode', app: null }))}>{t('Back')}</button></div></div></div>;
  }
  return <div className="modal-backdrop"><div className="modal confirm-modal backup-modal"><h3>{flow.mode === 'backup' ? `${t('Backup')} ${flow.app?.name || ''}` : `${t('Restore')} ${flow.app?.name || ''}`}</h3><div className="backup-path-block"><label>{flow.mode === 'backup' ? t('Folder to save backup file') : t('Backup zip file')}</label><div className="backup-path-row"><input readOnly value={flow.path || ''} placeholder={flow.mode === 'backup' ? t('Select folder...') : t('Select .zip file...')} /><button className="btn secondary" onClick={flow.mode === 'backup' ? chooseBackupFolder : chooseRestoreZip}>{flow.mode === 'backup' ? t('Choose folder') : t('Choose file')}</button></div></div><div className="button-row"><button className="btn primary" disabled={!flow.path} onClick={runBackupRestore}>{flow.mode === 'backup' ? t('Backup') : t('Restore')}</button><button className="btn secondary" onClick={() => setFlow(s => ({ ...s, step: 'app', path: '' }))}>{t('Back')}</button><button className="btn secondary" onClick={() => setFlow(null)}>{t('Cancel')}</button></div></div></div>;
}

// eslint-disable-next-line no-unused-vars
function InstallGuideModal({ onClose, t }) {
  const toolRows = [
    ['Node.js', 'Phụ trợ nền tảng cho npm, n8n, OpenClaw, Claude Code và 9router. Cài Node trước các tool npm. Nếu node/npm không nhận lệnh thì bấm Add PATH hoặc cài lại Node.js LTS.'],
    ['Git', 'Cần cho clone source, một số package npm và workflow dev. Nếu git không nhận lệnh thì Add PATH hoặc cài Git for Windows.'],
    ['OpenClaw', 'Cần Node.js/npm. Sau khi cài nên chọn đúng thư mục .openclaw và workspace, rồi chạy Doctor nếu gateway không chạy.'],
    ['Claude Code', 'Cần Node.js 18+ và tài khoản Claude/Anthropic để đăng nhập. Nếu cài lỗi quyền npm, tránh chạy sudo; trên Windows nên cài lại Node và mở terminal mới.'],
    ['9router', 'Cần Node.js/npm. Nếu lệnh 9router không nhận sau khi cài, bấm Add PATH cho npm global.'],
    ['n8n', 'Cần Node.js. Nếu không mở được http://localhost:5678, kiểm tra port 5678, dừng process node/n8n cũ rồi chạy lại.'],
    ['ngrok', 'Cần tài khoản ngrok và authtoken. Sau khi cài phải Map Authtoken trong API Keys > Ngrok trước khi chạy tunnel. Domain riêng cần đúng domain đã reserve trên ngrok.'],
    ['Python', 'Một số script/dev tool có thể cần Python. Nếu python mở Microsoft Store hoặc không nhận lệnh, Add PATH hoặc cài Python bản chính thức và bật Prepend PATH.'],
  ];
  const errorRows = [
    ['Command not found / is not recognized', 'Tool chưa cài hoặc PATH chưa cập nhật. Bấm Add PATH, đóng mở lại app, hoặc restart Windows.'],
    ['npm install failed / permission denied', 'Kiểm tra Node.js/npm, mạng internet, quyền ghi thư mục npm global. Có thể chạy Terminal bằng quyền Administrator nếu máy nội bộ cho phép.'],
    ['Port already in use / EADDRINUSE', 'Port đang bị app khác dùng. Với n8n thường là 5678. Dừng n8n/node cũ trong Run tab hoặc đổi port cấu hình.'],
    ['ngrok authentication failed', 'Chưa Map Authtoken, token sai, token đã bị rotate, hoặc tài khoản ngrok không có quyền dùng domain đó.'],
    ['ngrok domain/tunnel error', 'Kiểm tra domain không có https://, không có dấu /, port đúng, domain đã được reserve trong dashboard ngrok.'],
    ['OpenClaw gateway stopped', 'Kiểm tra OpenClaw đã cài, workspace/config đúng, chạy Doctor, sau đó Restart Gateway.'],
    ['API unauthorized 401/403', 'API key/token sai, thiếu quyền, hết hạn, hoặc chưa lưu trong API Keys. Nhập lại key rồi Save.'],
    ['App không mở trình duyệt/link', 'Kiểm tra app đã build bản mới có shell permission. Nếu Windows chặn default app, mở link thủ công từ ô copy.'],
  ];
  return <div className="modal-backdrop"><div className="modal install-guide-modal"><div className="install-guide-head"><h3>{t('Installation Guide')}</h3><button className="close-x" onClick={onClose}>x</button></div><div className="install-guide-body"><section><h4>{t('Recommended install order')}</h4><ol><li>Cài Node.js trước.</li><li>Cài Git để phục vụ dev/clone source.</li><li>Cài OpenClaw, Claude Code, 9router, n8n bằng npm.</li><li>Cài ngrok, sau đó vào API Keys &gt; Ngrok để Map Authtoken, Domain và Port.</li><li>Chạy thử từng tool ở Run tab; nếu lỗi thì dùng Terminal để kiểm tra version/status.</li></ol></section><section><h4>{t('Required helpers by app')}</h4><div className="guide-table">{toolRows.map(([name, detail]) => <div className="guide-row" key={name}><strong>{name}</strong><span>{detail}</span></div>)}</div></section><section><h4>{t('Common fixes')}</h4><div className="guide-table">{errorRows.map(([name, detail]) => <div className="guide-row" key={name}><strong>{name}</strong><span>{detail}</span></div>)}</div></section><section><h4>{t('When to use Add PATH / API Key')}</h4><ul><li>Dùng Add PATH khi cài xong nhưng Terminal báo không nhận lệnh như node, npm, git, n8n, ngrok, claude, 9router.</li><li>Dùng API Keys khi lỗi 401/403, model/provider không hoạt động, Telegram bot không gửi được, ngrok thiếu authtoken/domain/port.</li><li>Sau khi Add PATH hoặc cài tool mới, nên đóng mở lại app để Windows nạp PATH mới.</li></ul></section></div><div className="ok-center"><button className="btn primary" onClick={onClose}>{t('OK')}</button></div></div></div>;
}

function GuideModalV2({ language, onClose, t }) {
  const [section, setSection] = useState('install');
  const vi = language === 'vi';
  const copy = vi ? {
    title: 'Hướng Dẫn',
    installTab: 'Hướng Dẫn Cài Đặt',
    errorTab: 'Bảng Tra Cứu Lỗi',
    installTitle: 'Hướng Dẫn Cài Đặt Theo Ứng Dụng',
    errorTitle: 'Bảng Tra Cứu Lỗi Và Cách Fix',
    close: 'Đóng',
    tools: [
      ['Node.js', 'Cài trước các tool dùng npm như OpenClaw, Claude Code, 9router và n8n. Nếu node/npm không nhận lệnh, bấm Add PATH hoặc cài lại Node.js LTS.'],
      ['Git', 'Cần cho clone source, workflow dev và một số package. Nếu git không nhận lệnh, bấm Add PATH hoặc cài Git for Windows.'],
      ['OpenClaw', 'Cần Node.js/npm. Sau khi cài, chọn workspace, nhập API key nếu có, rồi dùng Doctor hoặc Fix Gateway nếu gateway không chạy.'],
      ['Claude Code', 'Cần Node.js 18+ và tài khoản Claude/Anthropic. Nếu npm lỗi quyền, mở app bằng quyền phù hợp hoặc sửa npm global path.'],
      ['9router', 'Cần Node.js/npm. Nếu cài xong nhưng không nhận lệnh 9router, bấm Add PATH rồi đóng mở lại app.'],
      ['n8n', 'Cần Node.js. Nếu không mở được http://localhost:5678, kiểm tra port 5678 và dừng process node/n8n cũ trước khi chạy lại.'],
      ['ngrok', 'Cần tài khoản ngrok và authtoken. Vào API Keys > Ngrok để nhập authtoken, domain, port; domain không kèm https:// hoặc dấu /.'],
      ['Python', 'Một số script/dev tool cần Python. Nếu python mở Microsoft Store hoặc không nhận lệnh, cài Python chính thức và bật Add to PATH.'],
    ],
    errors: [
      ['Command not found / is not recognized', 'Tool chưa cài hoặc PATH chưa cập nhật. Bấm Add PATH, đóng mở lại app, hoặc restart Windows nếu PATH vẫn chưa nhận.'],
      ['npm install failed / permission denied', 'Kiểm tra Node.js/npm, mạng internet và quyền ghi thư mục npm global. Có thể mở app bằng Administrator nếu máy cho phép.'],
      ['Port already in use / EADDRINUSE', 'Port đang bị app khác dùng. Dừng app tương ứng trong Run tab hoặc đổi port cấu hình. Với n8n thường là port 5678.'],
      ['OpenClaw gateway stopped', 'Kiểm tra OpenClaw đã cài, cấu hình openclaw.json đúng, chạy Fix Gateway hoặc Doctor rồi Restart Gateway.'],
      ['Dashboard không mở', 'Chạy gateway trước. Nếu có token gateway, app sẽ tự gắn token vào URL dashboard. Nếu vẫn lỗi, bấm Refresh rồi mở lại.'],
      ['Telegram bot không phản hồi', 'Kiểm tra botToken, defaultTo, dmPolicy=open, groups.*.requireMention=false và plugins.entries.telegram.enabled=true.'],
      ['ngrok authentication/domain error', 'Kiểm tra authtoken, domain đã reserve, domain không có https://, port đúng và tài khoản có quyền dùng domain đó.'],
      ['API 401/403', 'API key/token sai, hết hạn, thiếu quyền hoặc chưa Save. Nhập lại key trong API Keys rồi Save.'],
    ],
  } : {
    title: 'Guide',
    installTab: 'Installation Guide',
    errorTab: 'Error Lookup',
    installTitle: 'Per-App Installation Guide',
    errorTitle: 'Error Lookup And Fixes',
    close: 'Close',
    tools: [
      ['Node.js', 'Install before npm-based tools such as OpenClaw, Claude Code, 9router, and n8n. If node/npm is not recognized, use Add PATH or reinstall Node.js LTS.'],
      ['Git', 'Required for cloning source, dev workflows, and some packages. If git is not recognized, use Add PATH or install Git for Windows.'],
      ['OpenClaw', 'Requires Node.js/npm. After install, choose a workspace, add an API key when needed, then use Doctor or Fix Gateway if the gateway does not start.'],
      ['Claude Code', 'Requires Node.js 18+ and a Claude/Anthropic account. If npm has permission errors, run with the right Windows permissions or repair npm global path.'],
      ['9router', 'Requires Node.js/npm. If the 9router command is missing after install, use Add PATH and reopen the app.'],
      ['n8n', 'Requires Node.js. If http://localhost:5678 does not open, check port 5678 and stop old node/n8n processes before starting again.'],
      ['ngrok', 'Requires an ngrok account and authtoken. Configure authtoken, domain, and port in API Keys > Ngrok; domain should not include https:// or trailing /.'],
      ['Python', 'Some scripts/dev tools may require Python. If python opens Microsoft Store or is not recognized, install official Python and enable Add to PATH.'],
    ],
    errors: [
      ['Command not found / is not recognized', 'The tool is not installed or PATH is stale. Use Add PATH, reopen the app, or restart Windows if needed.'],
      ['npm install failed / permission denied', 'Check Node.js/npm, internet access, and write permission for the npm global folder. Use Administrator only when appropriate.'],
      ['Port already in use / EADDRINUSE', 'Another app owns the port. Stop the matching app in Run tab or change the configured port. n8n commonly uses 5678.'],
      ['OpenClaw gateway stopped', 'Check OpenClaw installation and openclaw.json, then run Fix Gateway or Doctor and restart the gateway.'],
      ['Dashboard does not open', 'Start the gateway first. If gateway token auth is enabled, the app appends the token to the dashboard URL automatically.'],
      ['Telegram bot does not respond', 'Check botToken, defaultTo, dmPolicy=open, groups.*.requireMention=false, and plugins.entries.telegram.enabled=true.'],
      ['ngrok authentication/domain error', 'Check authtoken, reserved domain ownership, no https:// in the domain, correct port, and account permission.'],
      ['API 401/403', 'The API key/token is wrong, expired, missing permissions, or not saved. Re-enter it in API Keys and Save.'],
    ],
  };
  const rows = section === 'install' ? copy.tools : copy.errors;
  return <div className="modal-backdrop"><div className="modal install-guide-modal"><div className="install-guide-head"><h3>{copy.title}</h3><button className="close-x" onClick={onClose}>x</button></div><div className="guide-tabs"><button className={section === 'install' ? 'active' : ''} onClick={() => setSection('install')}>{copy.installTab}</button><button className={section === 'error' ? 'active' : ''} onClick={() => setSection('error')}>{copy.errorTab}</button></div><div className="install-guide-body"><section><h4>{section === 'install' ? copy.installTitle : copy.errorTitle}</h4><div className="guide-table">{rows.map(([name, detail]) => <div className="guide-row" key={name}><strong>{name}</strong><span>{detail}</span></div>)}</div></section></div><div className="ok-center"><button className="btn primary" onClick={onClose}>{copy.close || t('OK')}</button></div></div></div>;
}

function emptySettings() { return { openclaw: { model: [models[0]], apiKeys: {}, customProviders: [] }, model: [models[0]], keys: [''], customProviders: [{ ...customProviderTemplate }], telegram: { botToken: '', chatId: '' }, google: { clientId: '', clientSecret: '', apiKey: '' }, ngrok: { authtoken: '', domain: '', port: '' }, n8n: { url: 'http://localhost:5678', apiKey: '' } }; }
function normalizeLoadedSettings(raw = {}) {
  const base = emptySettings();
  const openclaw = { ...base.openclaw, ...(raw.openclaw || {}) };
  const selected = Array.isArray(openclaw.model) ? openclaw.model : (raw.model ? [raw.model].flat() : base.model);
  const firstBot = raw.telegram?.bots?.[0] || {};
  const customProviders = (raw.customProviders || openclaw.customProviders || base.customProviders).map(p => ({ ...customProviderTemplate, ...p, baseUrl: p.baseUrl || p.base_url || '', apiKey: p.apiKey || p.api_key || '', api: p.api || p.apiType || 'openai-completions', modelId: p.modelId || p.model_id || p.model || '', contextWindow: p.contextWindow || p.context_window || '', maxTokens: p.maxTokens || p.max_tokens || '8192', reasoning: Boolean(p.reasoning) }));
  return {
    ...base,
    ...raw,
    openclaw: { ...openclaw, model: selected, apiKeys: openclaw.apiKeys || {}, customProviders: openclaw.customProviders || [] },
    model: selected,
    customProviders,
    telegram: {
      ...base.telegram,
      ...(raw.telegram || {}),
      botToken: raw.telegram?.botToken || raw.telegram?.api_key || firstBot.botToken || '',
      chatId: raw.telegram?.chatId || raw.telegram?.group_id || firstBot.chatId || '',
    },
    google: {
      ...base.google,
      ...(raw.google || {}),
      clientId: raw.google?.clientId || raw.google?.client_id || raw.google_console?.customer_id || '',
      clientSecret: raw.google?.clientSecret || raw.google?.client_secret || raw.google_console?.customer_secret_code || '',
      apiKey: raw.google?.apiKey || raw.google?.api_key || raw.google_console?.api_key || '',
    },
    ngrok: { ...base.ngrok, ...(raw.ngrok || {}) },
    n8n: {
      ...base.n8n,
      ...(raw.n8n || {}),
      url: raw.n8n?.url || raw.n8n?.base_url || base.n8n.url,
      apiKey: raw.n8n?.apiKey || raw.n8n?.api_key || '',
    },
  };
}
function ApiTab({ toast, t }) {
  const [subtab, setSubtab] = useState('AI Model'); const [settings, setSettings] = useState(emptySettings()); const [modelModal, setModelModal] = useState(false); const [draftModels, setDraftModels] = useState([]);
  const [resetNgrokConfirm, setResetNgrokConfirm] = useState(false);
  useEffect(() => { let active = true; invoke('load_settings').then(res => { if (active) setSettings(normalizeLoadedSettings(res || {})); }).catch(e => toast(String(e))); return () => { active = false; }; }, [toast]);
  const selectedModels = Array.isArray(settings.model) ? settings.model : [settings.model].filter(Boolean);
  const apiKeys = settings.openclaw?.apiKeys || {};
  const save = async () => { try { const cfg = normalizeLoadedSettings({ ...settings, openclaw: { ...(settings.openclaw || {}), customProviders: settings.customProviders || [] } }); await invoke('save_settings', { cfg }); toast(t('Settings saved')); } catch (e) { toast(String(e)); } };
  const resetNgrokMapping = async () => {
    try {
      await invoke('run_action', { action: 'reset-ngrok-mapping' });
      toast(t('Reset mapping done!'));
      setResetNgrokConfirm(false);
    } catch (e) {
      toast(String(e));
    }
  };
  const setField = (section, field, value) => setSettings(s => ({ ...s, [section]: { ...s[section], [field]: value } }));
  const openModelPicker = () => { setDraftModels(selectedModels); setModelModal(true); };
  const applyModels = () => { const next = draftModels.length ? draftModels : [models[0]]; setSettings(s => ({ ...s, model: next, openclaw: { ...(s.openclaw || {}), model: next, apiKeys: { ...(s.openclaw?.apiKeys || {}) } }, keys: next.map((_, i) => s.keys?.[i] || '') })); setModelModal(false); };
  const setApiKey = (model, value) => setSettings(s => ({ ...s, openclaw: { ...(s.openclaw || {}), model: selectedModels, apiKeys: { ...(s.openclaw?.apiKeys || {}), [model]: value } } }));
  const setCustomProvider = (index, patch) => setSettings(s => ({ ...s, customProviders: s.customProviders.map((item, n) => n === index ? { ...item, ...patch } : item) }));
  return <div>
    <div className="subtabs">{apiTabs.map(tab => <button key={tab} className={subtab === tab ? 'active' : ''} onClick={() => setSubtab(tab)}>{t(tab)}</button>)}</div>
    {subtab === 'AI Model' && <Panel title={t('AI Model')}><button className="btn secondary" onClick={openModelPicker}>{selectedModels.length} {t('models selected')}</button><div className="form-grid">{selectedModels.map((m, i) => <label key={m}>{m} {t('API Key')}<input type="password" value={apiKeys[m] || ''} onChange={e => setApiKey(m, e.target.value)} placeholder={`${t('API Key')} ${i + 1}`} /></label>)}</div></Panel>}
    {subtab === 'Custom Provider' && <Panel title={t('Custom Provider')}><div className="form-grid">{settings.customProviders.map((p, i) => <div className="panel" key={i}><input placeholder={t('Provider ID')} value={p.name} onChange={e => setCustomProvider(i, { name: e.target.value })} /><input placeholder={t('Base URL')} value={p.baseUrl} onChange={e => setCustomProvider(i, { baseUrl: e.target.value })} /><input placeholder={t('API Key')} type="password" value={p.apiKey} onChange={e => setCustomProvider(i, { apiKey: e.target.value })} /><select value={p.api || 'openai-completions'} onChange={e => setCustomProvider(i, { api: e.target.value })}>{providerApiOptions.map(option => <option key={option} value={option}>{option}</option>)}</select><input placeholder={t('Model ID')} value={p.modelId || ''} onChange={e => setCustomProvider(i, { modelId: e.target.value })} /><input placeholder={t('Context Window')} value={p.contextWindow || ''} onChange={e => setCustomProvider(i, { contextWindow: e.target.value.replace(/[^0-9]/g, '') })} /><input placeholder={t('Max Tokens')} value={p.maxTokens || ''} onChange={e => setCustomProvider(i, { maxTokens: e.target.value.replace(/[^0-9]/g, '') })} /><label className="checkbox-line"><input type="checkbox" checked={Boolean(p.reasoning)} onChange={e => setCustomProvider(i, { reasoning: e.target.checked })} />{t('Reasoning')}</label><button className="btn danger" onClick={() => setSettings(s => ({ ...s, customProviders: s.customProviders.filter((_, n) => n !== i) }))}>{t('Remove')}</button></div>)}</div><button className="btn secondary" onClick={() => setSettings(s => ({ ...s, customProviders: [...s.customProviders, { ...customProviderTemplate }] }))}><Plus size={14} />{t('Add Provider')}</button></Panel>}
    {subtab === 'Telegram' && <TelegramTab settings={settings} setSettings={setSettings} t={t} />}
    {subtab === 'Google API' && <SimpleForm data={settings.google} fields={['clientId', 'clientSecret', 'apiKey']} onChange={(f, v) => setField('google', f, v)} t={t} />}
    {subtab === 'Ngrok' && <Panel title={t('Settings')}><div className="form-grid"><label>{t('authtoken')}<input value={settings.ngrok.authtoken || ''} onChange={e => { let v = e.target.value.trim(); if (v.includes('add-authtoken ')) v = v.split('add-authtoken ').pop().trim(); setField('ngrok', 'authtoken', v); }} /></label><label>{t('domain')}<input value={settings.ngrok.domain || ''} onChange={e => { let v = e.target.value.trim().replace('https://', '').replace('http://', '').replace(/\d+\/?$/, '').replace(/\/$/, ''); setField('ngrok', 'domain', v); }} /></label><label>{t('port')}<input value={settings.ngrok.port || ''} placeholder={t('default port is 5678')} onChange={e => setField('ngrok', 'port', e.target.value)} /></label></div><div className="button-row" style={{ marginTop: 12 }}><button className="btn danger" onClick={() => setResetNgrokConfirm(true)}>{t('Reset Mapping')}</button><button className="btn warning" onClick={async () => { if (!settings.ngrok.authtoken) { toast(t('Enter authtoken first')); return; } try { await invoke('run_action', { action: 'map-authtoken-' + settings.ngrok.authtoken }); toast(t('Authtoken mapped!')); } catch (e) { toast(String(e)); } }}>{t('Map Authtoken')}</button><button className="btn warning" onClick={async () => { if (!settings.ngrok.domain) { toast(t('Enter domain first')); return; } await save(); toast(t('Domain saved!')); }}>{t('Map Domain')}</button></div></Panel>}
    {subtab === 'N8N' && <SimpleForm data={settings.n8n} fields={['url', 'apiKey']} onChange={(f, v) => setField('n8n', f, v)} t={t} />}
    <div className="button-row">{subtab === 'Telegram' && <button className="btn warning" onClick={() => { const bots = Array.isArray(settings.telegram?.bots) ? settings.telegram.bots : [{ botToken: settings.telegram?.botToken || '', chatId: settings.telegram?.chatId || '' }]; if (bots.length >= 5) { toast(t('Maximum 5 bots')); return; } setSettings(s => ({ ...s, telegram: { ...s.telegram, bots: [...bots, { botToken: '', chatId: '' }] } })); }}>{t('Add Bot')}</button>}<button className="btn primary" onClick={save}>{t('Save')}</button></div>
    {modelModal && <div className="modal-backdrop"><div className="modal"><h3>{t('Choose models')}</h3><div className="model-grid checkbox-grid">{models.map(m => <label key={m} className="radio-option"><input type="checkbox" checked={draftModels.includes(m)} onChange={e => setDraftModels(d => e.target.checked ? [...d, m] : d.filter(x => x !== m))} /><span>{m}</span></label>)}</div><div className="button-row" style={{ marginTop: 14 }}><button className="btn primary" onClick={applyModels}>{t('Save')}</button><button className="btn secondary" onClick={() => setModelModal(false)}>{t('Cancel')}</button></div></div></div>}
    {resetNgrokConfirm && <div className="modal-backdrop"><div className="modal confirm-modal"><h3>{t('Reset Mapping')}</h3><p className="busy-result">{t('Reset Mapping Warning')}</p><div className="button-row"><button className="btn primary" onClick={resetNgrokMapping}>{t('OK')}</button><button className="btn secondary" onClick={() => setResetNgrokConfirm(false)}>{t('Cancel')}</button></div></div></div>}
  </div>;
}
function SimpleForm({ data, fields, onChange, t }) { return <Panel title={t('Settings')}><div className="form-grid">{fields.map(f => <label key={f}>{t(f)}<input value={data[f] || ''} onChange={e => onChange(f, e.target.value)} /></label>)}</div></Panel>; }
function LogsTab({ toast, t }) { const [selected, setSelected] = useState('gateway.log'); const [content, setContent] = useState(t('Select a log file to view output.')); const logs = ['gateway.log', 'webui.log', 'n8n.log', 'ngrok.log', 'claude-code.log', '9router.log']; const open = async (name) => { setSelected(name); try { const res = await invoke('read_log', { name }); setContent(String(res || '')); } catch (e) { setContent(String(e)); } }; const openFolder = async () => { try { await invoke('open_logs_folder'); } catch (e) { toast(String(e)); } }; return <div className="logs-page"><div className="log-list"><button className="btn secondary" onClick={openFolder}><FolderOpen size={15} />{t('Open Folder')}</button>{logs.map(l => <button key={l} className={selected === l ? 'active' : ''} onClick={() => open(l)}>{l}</button>)}</div><pre className="log-view">{content}</pre></div>; }
function ThanksTab({ language, t }) {
  return <div className="thanks-page"><div className="thanks-card"><div className="thanks-head"><Heart size={34} /><h1>{t('Thanks')}</h1></div><img className="thanks-qr" src={qrImage} alt="Donate QR" /><div className="thanks-message">{thanksLines(language).map((line, index) => <p key={index}>{line}</p>)}</div></div></div>;
}
function TelegramTab({ settings, setSettings, t }) {
  const bots = Array.isArray(settings.telegram?.bots) ? settings.telegram.bots : [{ botToken: settings.telegram?.botToken || '', chatId: settings.telegram?.chatId || '' }];
  const setBots = (newBots) => setSettings(s => ({ ...s, telegram: { ...s.telegram, bots: newBots } }));
  const updateBot = (i, field, value) => setBots(bots.map((b, n) => n === i ? { ...b, [field]: value } : b));
  const removeBot = (i) => setBots(bots.filter((_, n) => n !== i));
  return <Panel title={<span>{t('Settings')} <span style={{ fontSize: '0.75rem', color: '#94a3b8', fontWeight: 400, marginLeft: 8 }}>{t('Up to 5 bots can be added')}</span></span>}><div className="form-grid">{bots.map((bot, i) => <div key={i} className="panel" style={{ padding: 12, marginBottom: 8 }}><label>{t('API Bot Token')} {bots.length > 1 ? `#${i+1}` : ''}<input value={bot.botToken || ''} onChange={e => updateBot(i, 'botToken', e.target.value)} placeholder={t('Bot token from @BotFather')} /></label><label>{t('Group Chat ID')} {bots.length > 1 ? `#${i+1}` : ''}<input value={bot.chatId || ''} onChange={e => updateBot(i, 'chatId', e.target.value)} placeholder="-100xxxxxxxxxx" /></label>{bots.length > 1 && <button className="btn danger" style={{ marginTop: 6, padding: '4px 10px', fontSize: 11 }} onClick={() => removeBot(i)}>{t('Remove')}</button>}</div>)}</div></Panel>;
}

export default App;


