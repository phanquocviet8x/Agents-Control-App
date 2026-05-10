const fs=require('fs');
const path=require('path');
const app=String.raw`import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  Play, Square, RotateCcw, Globe, Terminal as TerminalIcon, Settings,
  HelpCircle, KeyRound, ScrollText, Heart, CheckCircle2, Download,
  Trash2, ChevronDown, FolderOpen, X, Minus, Maximize2, Plus
} from 'lucide-react';
import './App.css';

const tabs = [
  { key: 'run', label: 'Run', icon: Play },
  { key: 'terminal', label: 'Terminal', icon: TerminalIcon },
  { key: 'setup', label: 'Setup', icon: Settings },
  { key: 'help', label: 'Help', icon: HelpCircle },
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
  { key: 'ngrok', name: 'ngrok', versions: ['3.20.0', '3.19.1', '3.19.0', '3.18.4', '3.18.3'] },
];

const models = [
  'Claude Opus 4.6', 'Claude Sonnet 4.6', 'GPT-5.5', 'GPT-5.4', 'o3', 'o4-mini',
  'Gemini 2.5 Pro', 'Gemini 3 Pro', 'Gemini 2.5 Flash', 'Gemini 3 Flash',
  'Llama 4 Scout', 'Llama 4 Maverick', 'DeepSeek R1', 'DeepSeek V3',
  'Grok 3', 'Grok 3 Mini', 'OpenRouter Auto', '9router Custom'
];

const apiTabs = ['AI Model', 'Custom Provider', 'Telegram', 'Google API', 'Ngrok', 'N8N'];

function App() {
  const [activeTab, setActiveTab] = useState('run');
  const [status, setStatus] = useState({ gateway: false, n8n: false, ngrok: false, claude: false, router: false });
  const [toasts, setToasts] = useState([]);

  const toast = (text) => {
    const id = Date.now() + Math.random();
    setToasts((t) => [...t, { id, text }]);
    setTimeout(() => setToasts((t) => t.filter((x) => x.id !== id)), 3200);
  };

  const runAction = async (action) => {
    try {
      const res = await invoke('run_action', { action });
      toast(String(res || action));
      if (action.includes('gateway')) setStatus((s) => ({ ...s, gateway: action.includes('run') || action.includes('restart') }));
      if (action.includes('n8n')) setStatus((s) => ({ ...s, n8n: action.includes('run') }));
      if (action.includes('ngrok')) setStatus((s) => ({ ...s, ngrok: action.includes('run') }));
      if (action.includes('claude')) setStatus((s) => ({ ...s, claude: action.includes('run') }));
      if (action.includes('9router')) setStatus((s) => ({ ...s, router: action.includes('run') }));
    } catch (e) { toast(String(e)); }
  };

  return (
    <div className="app-shell">
      <Titlebar />
      <div className="app-body">
        <aside className="sidebar">
          <div className="nav-list">
            {tabs.map(({ key, label, icon: Icon }) => (
              <button key={key} className={activeTab === key ? 'active' : ''} onClick={() => setActiveTab(key)}><Icon size={18} />{label}</button>
            ))}
          </div>
        </aside>
        <main className="content"><div className="content-scroll">
          {activeTab === 'run' && <RunTab status={status} runAction={runAction} />}
          {activeTab === 'terminal' && <TerminalTab toast={toast} />}
          {activeTab === 'setup' && <SetupTab toast={toast} />}
          {activeTab === 'help' && <HelpTab runAction={runAction} />}
          {activeTab === 'api' && <ApiTab toast={toast} />}
          {activeTab === 'logs' && <LogsTab toast={toast} />}
          {activeTab === 'thanks' && <ThanksTab />}
        </div></main>
      </div>
      <div className="toasts">{toasts.map((t) => <div className="toast" key={t.id}>{t.text}</div>)}</div>
    </div>
  );
}

function Titlebar() {
  const win = getCurrentWindow();
  return <div className="titlebar" data-tauri-drag-region>
    <div className="titlebar-brand"><Globe size={18} /> OpenClaw Control Center</div>
    <div className="window-controls">
      <button className="window-btn" onClick={() => win.minimize()}><Minus size={14} /></button>
      <button className="window-btn" onClick={() => win.toggleMaximize()}><Maximize2 size={14} /></button>
      <button className="window-btn" onClick={() => win.close()}><X size={14} /></button>
    </div>
  </div>;
}

function Panel({ title, children }) {
  return <section className="panel"><h2 className="panel-title">{title}</h2><div className="panel-body">{children}</div></section>;
}
function StatusDot({ on }) { return <span className={'status-dot ' + (on ? 'running' : '')} />; }

function RunTab({ status, runAction }) {
  return <div className="page-grid">
    <Panel title="OpenClaw"><p><StatusDot on={status.gateway} /> {status.gateway ? 'Running' : 'Not started'}</p><div className="button-row">
      <button className="btn primary" onClick={() => runAction('gateway-run')}>Run</button><button className="btn danger" onClick={() => runAction('gateway-stop')}>Stop</button><button className="btn warning" onClick={() => runAction('gateway-restart')}>Restart</button><button className="btn primary" onClick={() => runAction('webui-run')}>WebUI</button><button className="btn danger" onClick={() => runAction('webui-stop')}>WebUI Stop</button>
    </div></Panel>
    <Panel title="N8N & Ngrok"><p><StatusDot on={status.n8n} /> N8N · <StatusDot on={status.ngrok} /> Ngrok</p><div className="button-row"><button className="btn primary" onClick={() => runAction('n8n-run')}>N8N Run</button><button className="btn danger" onClick={() => runAction('n8n-stop')}>N8N Stop</button><button className="btn primary" onClick={() => runAction('ngrok-run')}>Ngrok Run</button><button className="btn danger" onClick={() => runAction('ngrok-stop')}>Ngrok Stop</button></div></Panel>
    <Panel title="Claude Code"><p><StatusDot on={status.claude} /> {status.claude ? 'Running' : 'Not started'}</p><div className="button-row"><button className="btn primary" onClick={() => runAction('claude-code-run')}>Run</button><button className="btn danger" onClick={() => runAction('claude-code-stop')}>Stop</button></div></Panel>
    <Panel title="9router"><p><StatusDot on={status.router} /> {status.router ? 'Running' : 'Not started'}</p><div className="button-row"><button className="btn primary" onClick={() => runAction('9router-run')}>Run</button><button className="btn danger" onClick={() => runAction('9router-stop')}>Stop</button></div></Panel>
  </div>;
}

function TerminalTab({ toast }) {
  const [lines, setLines] = useState(['OpenClaw PowerShell terminal ready.']);
  const [cmd, setCmd] = useState('');
  const endRef = useRef(null);
  useEffect(() => endRef.current?.scrollIntoView({ behavior: 'smooth' }), [lines]);
  const submit = async (e) => { e.preventDefault(); if (!cmd.trim()) return; const current = cmd; setCmd(''); setLines((l) => [...l, 'PS> ' + current]); try { const out = await invoke('run_terminal_cmd', { cmd: current }); setLines((l) => [...l, String(out || '')]); } catch (err) { toast(String(err)); setLines((l) => [...l, 'ERROR: ' + err]); } };
  return <div className="terminal-page"><div className="terminal-screen">{lines.map((l, i) => <div className="terminal-line" key={i}>{l}</div>)}<div ref={endRef} /></div><form className="terminal-input-row" onSubmit={submit}><span className="terminal-prompt">PS&gt;</span><input value={cmd} onChange={(e) => setCmd(e.target.value)} autoFocus /></form></div>;
}

function SetupTab({ toast }) {
  const [installed, setInstalled] = useState({});
  const [versions, setVersions] = useState({});
  const [selected, setSelected] = useState(Object.fromEntries(setupTools.map(t => [t.key, t.versions[0]])));
  const [openDrop, setOpenDrop] = useState(null);
  const [busy, setBusy] = useState({});
  const [confirm, setConfirm] = useState(null);
  const runSetup = async (tool, kind) => { setBusy((b) => ({ ...b, [tool.key]: true })); try { const action = kind === 'install' ? 'install-tool' : 'uninstall-tool'; const res = await invoke('run_action', { action: action + '-' + tool.key + '-' + (selected[tool.key] || '') }); toast(String(res || kind + ' ' + tool.name)); setInstalled((s) => ({ ...s, [tool.key]: kind === 'install' })); } catch (e) { toast(String(e)); } finally { setBusy((b) => ({ ...b, [tool.key]: false })); setConfirm(null); } };
  const installConfirmText = (tool) => 'Install ' + tool.name + ' ' + selected[tool.key] + '?';
  const uninstallConfirmText = (tool) => 'Uninstall ' + tool.name + '?';
  const checkTools = async () => { try { const res = await invoke('check_tools'); setInstalled(typeof res === 'object' ? res : {}); } catch (e) { toast(String(e)); } };
  const checkVersions = async () => { try { const res = await invoke('check_versions'); setVersions(typeof res === 'object' ? res : {}); } catch (e) { toast(String(e)); } };
  useEffect(() => { checkTools(); }, []);
  return <><div className="setup-grid">{setupTools.map((tool) => <div className="tool-card" key={tool.key}><div className="tool-card-head"><h3>{tool.name}</h3>{busy[tool.key] ? <span className="progress-ring" /> : installed[tool.key] ? <CheckCircle2 className="check-done" /> : null}</div><div className="tool-card-actions"><button className="btn primary" onClick={() => setConfirm({ tool, kind: 'install', text: installConfirmText(tool) })}><Download size={15} />Install</button><button className="btn danger" onClick={() => setConfirm({ tool, kind: 'uninstall', text: uninstallConfirmText(tool) })}><Trash2 size={15} />Uninstall</button><div className="dropdown-wrap"><button className="btn secondary" onClick={() => setOpenDrop(openDrop === tool.key ? null : tool.key)}>{selected[tool.key]} <ChevronDown size={14} /></button>{openDrop === tool.key && <div className="dropdown-menu">{tool.versions.map(v => <button key={v} onClick={() => { setSelected((s) => ({ ...s, [tool.key]: v })); setOpenDrop(null); }}>{v}</button>)}</div>}</div></div></div>)}</div><div className="page-grid"><Panel title="Installed Apps"><div className="installed-list">{setupTools.map(t => <div className="installed-item" key={t.key}><span>{t.name}</span><strong className={installed[t.key] ? 'installed' : 'not-installed'}>{installed[t.key] ? 'Installed' : 'Not Installed'}</strong></div>)}</div></Panel><Panel title="Versions"><button className="btn secondary" onClick={checkVersions}>Check Status</button><div className="installed-list">{setupTools.map(t => <div className="installed-item" key={t.key}><span>{t.name}</span><strong>{versions[t.key] || 'Unknown'}</strong></div>)}</div></Panel></div>{confirm && <div className="modal-backdrop"><div className="modal confirm-modal"><h3>{confirm.text}</h3><div className="button-row"><button className="btn primary" onClick={() => runSetup(confirm.tool, confirm.kind)}>Yes</button><button className="btn secondary" onClick={() => setConfirm(null)}>No</button></div></div></div>}</>;
}

function HelpTab({ runAction }) {
  const apps = ['OpenClaw', 'N8N', 'Ngrok', 'Claude Code'];
  const key = (name) => name.toLowerCase().replaceAll(' ', '-');
  return <><div className="page-grid">{apps.map(app => <Panel title={app} key={app}><div className="button-row"><button className="btn secondary" onClick={() => runAction(key(app) + '-status')}>Status</button><button className="btn warning" onClick={() => runAction(key(app) + '-doctor')}>Doctor</button><button className="btn primary" onClick={() => runAction(key(app) + '-config')}>Config</button></div></Panel>)}</div><Panel title="Getting Started"><ol><li>Install Node.js first.</li><li>Install OpenClaw, Claude Code, 9router, n8n, and ngrok from Setup.</li><li>Add API keys in the API Keys tab.</li><li>Start services from the Run tab.</li></ol></Panel><Panel title="Usage Guide"><p>Use Run for service control, Terminal for commands, Logs for diagnostics, and Help for quick doctor/config actions.</p></Panel></>;
}

function emptySettings() { return { model: models[0], keys: ['', '', '', '', '', ''], customProviders: [{ name: '', baseUrl: '', apiKey: '' }], telegram: { botToken: '', chatId: '' }, google: { clientId: '', clientSecret: '', apiKey: '' }, ngrok: { authtoken: '', domain: '' }, n8n: { url: 'http://localhost:5678', apiKey: '' } }; }
function ApiTab({ toast }) {
  const [subtab, setSubtab] = useState('AI Model');
  const [settings, setSettings] = useState(emptySettings());
  const [modelModal, setModelModal] = useState(false);
  const save = async () => { try { await invoke('save_settings', { cfg: settings }); toast('Settings saved'); } catch (e) { toast(String(e)); } };
  const setField = (section, field, value) => setSettings(s => ({ ...s, [section]: { ...s[section], [field]: value } }));
  return <div><div className="subtabs">{apiTabs.map(t => <button key={t} className={subtab === t ? 'active' : ''} onClick={() => setSubtab(t)}>{t}</button>)}</div>{subtab === 'AI Model' && <Panel title="AI Model"><button className="btn secondary" onClick={() => setModelModal(true)}>{settings.model}</button><div className="form-grid">{settings.keys.map((k, i) => <label key={i}>API Key {i + 1}<input type="password" value={k} onChange={e => setSettings(s => ({ ...s, keys: s.keys.map((x, n) => n === i ? e.target.value : x) }))} /></label>)}</div></Panel>}{subtab === 'Custom Provider' && <Panel title="Custom Provider"><div className="form-grid">{settings.customProviders.map((p, i) => <div className="panel" key={i}><input placeholder="Name" value={p.name} onChange={e => setSettings(s => ({ ...s, customProviders: s.customProviders.map((x, n) => n === i ? { ...x, name: e.target.value } : x) }))} /><input placeholder="Base URL" value={p.baseUrl} onChange={e => setSettings(s => ({ ...s, customProviders: s.customProviders.map((x, n) => n === i ? { ...x, baseUrl: e.target.value } : x) }))} /><input placeholder="API Key" type="password" value={p.apiKey} onChange={e => setSettings(s => ({ ...s, customProviders: s.customProviders.map((x, n) => n === i ? { ...x, apiKey: e.target.value } : x) }))} /><button className="btn danger" onClick={() => setSettings(s => ({ ...s, customProviders: s.customProviders.filter((_, n) => n !== i) }))}>Remove</button></div>)}</div><button className="btn secondary" onClick={() => setSettings(s => ({ ...s, customProviders: [...s.customProviders, { name: '', baseUrl: '', apiKey: '' }] }))}><Plus size={14} />Add Provider</button></Panel>}{subtab === 'Telegram' && <SimpleForm data={settings.telegram} fields={['botToken', 'chatId']} onChange={(f, v) => setField('telegram', f, v)} />}{subtab === 'Google API' && <SimpleForm data={settings.google} fields={['clientId', 'clientSecret', 'apiKey']} onChange={(f, v) => setField('google', f, v)} />}{subtab === 'Ngrok' && <SimpleForm data={settings.ngrok} fields={['authtoken', 'domain']} onChange={(f, v) => setField('ngrok', f, v)} />}{subtab === 'N8N' && <SimpleForm data={settings.n8n} fields={['url', 'apiKey']} onChange={(f, v) => setField('n8n', f, v)} />}<div className="button-row"><button className="btn primary" onClick={save}>Save</button></div>{modelModal && <div className="modal-backdrop"><div className="modal"><h3>Choose model</h3><div className="model-grid">{models.map(m => <button key={m} className="btn secondary" onClick={() => { setSettings(s => ({ ...s, model: m })); setModelModal(false); }}>{m}</button>)}</div></div></div>}</div>;
}
function SimpleForm({ data, fields, onChange }) { return <Panel title="Settings"><div className="form-grid">{fields.map(f => <label key={f}>{f}<input value={data[f] || ''} onChange={e => onChange(f, e.target.value)} /></label>)}</div></Panel>; }

function LogsTab({ toast }) {
  const [selected, setSelected] = useState('gateway.log');
  const [content, setContent] = useState('Select a log file to view output.');
  const logs = ['gateway.log', 'webui.log', 'n8n.log', 'ngrok.log', 'claude-code.log', '9router.log'];
  const open = async (name) => { setSelected(name); try { const res = await invoke('run_action', { action: 'logs-read-' + name }); setContent(String(res || '')); } catch (e) { setContent(String(e)); } };
  const openFolder = async () => { try { await invoke('run_action', { action: 'logs-open-folder' }); } catch (e) { toast(String(e)); } };
  return <div className="logs-page"><div className="log-list"><button className="btn secondary" onClick={openFolder}><FolderOpen size={15} />Open Folder</button>{logs.map(l => <button key={l} className={selected === l ? 'active' : ''} onClick={() => open(l)}>{l}</button>)}</div><pre className="log-view">{content}</pre></div>;
}

function ThanksTab() {
  return <div className="thanks-page"><div className="thanks-card"><Heart size={42} /><h1>Thanks</h1><p>Built with React, Tauri, OpenClaw, Claude Code, 9router, n8n, ngrok, and a pile of late-night curiosity.</p><p>Credits to the open-source maintainers and every human who makes automation friendlier.</p></div></div>;
}

export default App;
`;
fs.writeFileSync(path.join(__dirname,'src','App.jsx'), app, 'utf8');
console.log('Wrote src/App.jsx', app.length, 'chars');
