// Copyright (c) 2026 Vu. All rights reserved.
// Proprietary source. See OWNERSHIP.md at the repository root.

import { useCallback, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
import { translate, translateState } from './i18n.js';

const APPS = ['node','openclaw','n8n','ngrok','claude-code','9router','git','python'];
const VERSION_KEYS = ['node','openclaw','claude-code','9router','n8n','ngrok','git','python'];
const GATEWAY_MODES = [
  { icon: '🏠', label: 'Local Only', desc: 'Access from this machine only', value: 'local' },
  { icon: '🌐', label: 'LAN/Remote', desc: 'Access from other devices on your network', value: 'lan' },
  { icon: '🔒', label: 'Tailscale Funnel', desc: 'Expose securely through Tailscale', value: 'tailscale' },
  { icon: '☁️', label: 'Cloud/VPS', desc: 'Run OpenClaw on a remote server', value: 'cloud' },
];
const SECTIONS = [
  ['workspace', '📁', 'Workspace'], ['model', '🤖', 'Model Provider'], ['web', '🔍', 'Web Tools'], ['gateway', '🌐', 'Gateway'], ['daemon', '⚙️', 'Daemon'],
  ['channels', '💬', 'Channels'], ['plugins', '🧩', 'Plugins'], ['skills', '🎯', 'Skills'], ['health', '🏥', 'Health Check'], ['continue', '✅', 'Continue/Outro'],
];
const PROVIDERS = [
  ['OpenAI', '🟢'], ['Anthropic', '🟠'], ['Google Gemini', '🔵'], ['Mistral', '🔷'], ['Groq', '⚡'], ['Together AI', '🤝'], ['Fireworks', '🎆'], ['Perplexity', '🔮'], ['DeepSeek', '🐋'], ['Cohere', '🧬'], ['AI21', '📐'], ['Inflection', '💠'], ['xAI (Grok)', '🧠'], ['Cerebras', '🧊'], ['SambaNova', '🔶'], ['Anyscale', '📡'], ['Replicate', '🔁'], ['HuggingFace', '🤗'], ['AWS Bedrock', '☁️'], ['Azure OpenAI', '🔷'], ['Cloudflare AI', '🌩️'], ['Databricks', '🧱'], ['NVIDIA NIM', '💚'], ['Ollama', '🦙'], ['LM Studio', '🖥️'], ['Jan', '🟣'], ['LocalAI', '🏠'], ['vLLM', '⚙️'], ['llama.cpp', '🦙'], ['Text Gen WebUI', '🌐'], ['KoboldCpp', '🐉'], ['Oobabooga', '🕷️'], ['GPT4All', '📦'], ['OpenRouter', '🔀'], ['LiteLLM', '💡'], ['FastChat', '💬'], ['Aphrodite', '🌺'], ['TabbyAPI', '🐱'], ['ExLlamaV2', '🦎'], ['MLC LLM', '📱'], ['PowerInfer', '⚡'], ['Petals', '🌸'], ['Modal', '☁️'], ['RunPod', '🚀'], ['Custom Provider', '🛠️']
];
const SEARCH_PROVIDERS = ['Perplexity','Brave','Bing','Google','DuckDuckGo','SearXNG','Tavily','Exa','You.com','Serper','Serply','Custom'];
const CHANNELS = ['Telegram','Discord','Slack','WhatsApp','Signal','iMessage','Mattermost','Matrix','IRC','Microsoft Teams','Google Chat','Feishu/Lark','WeCom','LINE','Zalo','Nostr','Twitch','Synology Chat','Nextcloud Talk','BlueBubbles','QQ Bot','Tlon','Webchat','REST API'];
const PLUGINS = [
  { name: 'Amazon Bedrock', icon: '☁️', fields: [['Enable Discovery', 'select', 'Find Bedrock models automatically.', ['Yes', 'No']], ['Discovery Region', 'text', 'AWS region used for discovery.'], ['Provider Filter', 'text', 'Optional provider allow/filter list.'], ['Discovery Refresh Interval', 'text', 'How often discovery refreshes.'], ['Default Context Window', 'text', 'Fallback context window for models.'], ['Default Max Tokens', 'text', 'Fallback max output tokens.'], ['Guardrail', 'text', 'Optional Bedrock guardrail id.']] },
  { name: 'Brave Search', icon: '🦁', fields: [['API Key', 'password', 'Sensitive field — use CLI or Web UI to set securely'], ['Search Mode', 'select', 'Choose regular web or LLM context search.', ['web', 'llm-context']], ['Base URL', 'text', 'Override Brave Search base URL.']] },
  { name: 'GitHub Copilot', icon: '🐙', fields: [['Enable Discovery', 'select', 'Discover Copilot-backed models.', ['Yes', 'No']]] },
  { name: 'Google (Gemini)', icon: '🔍', fields: [['API Key', 'password', 'Sensitive field — use CLI or Web UI to set securely'], ['Search Model', 'text', 'Gemini model for search grounding.'], ['Base URL', 'text', 'Optional API base URL.']] },
  { name: 'HuggingFace', icon: '🤗', fields: [['Enable Discovery', 'select', 'Discover HuggingFace models.', ['Yes', 'No']]] },
  { name: 'MiniMax', icon: '🔷', fields: [['Token Plan Key', 'password', 'Sensitive field — use CLI or Web UI to set securely'], ['Search Region', 'select', 'Global or China endpoint.', ['global', 'cn']]] },
  { name: 'Moonshot (Kimi)', icon: '🌙', fields: [['API Key', 'password', 'Sensitive field — use CLI or Web UI to set securely'], ['Base URL', 'text', 'Moonshot API base URL.'], ['Model', 'text', 'Default Kimi model.']] },
  { name: 'Ollama', icon: '🦙', fields: [['Enable Discovery', 'select', 'Discover local Ollama models.', ['Yes', 'No']]] },
  { name: 'xAI (Grok)', icon: '🧠', fields: [['API Key', 'password', 'Sensitive field — use CLI or Web UI to set securely'], ['Search Model', 'text', 'Model for search.'], ['Inline Citations', 'select', 'Include citations in answers.', ['Yes', 'No']], ['Enable Code Execution', 'select', 'Allow code execution tool.', ['Yes', 'No']], ['Enable X Search', 'select', 'Allow X search.', ['Yes', 'No']], ['X Search Model', 'text', 'Model used for X search.'], ['X Search Timeout', 'text', 'Timeout in ms.'], ['X Search Cache TTL', 'text', 'Cache time-to-live.'], ['Code Execution Model', 'text', 'Model used for code execution.'], ['Code Execution Timeout', 'text', 'Execution timeout in ms.']] },
  { name: 'Device Pairing', icon: '📱', fields: [['Gateway URL', 'text', 'Gateway URL shown to pairing devices.']] },
];

function titleCase(s){ return String(s).replace(/-/g,' ').replace(/\b\w/g,c=>c.toUpperCase()); }
function statusClass(v){ const s=String(v||'').toLowerCase(); if(s.includes('running')) return 'running'; if(s.includes('stopped')) return 'stopped'; if(s.includes('not')) return 'not-installed'; if(s.includes('install')) return 'installed'; return 'not-installed'; }
function Field({ label, type='text', options, desc, placeholder, value='', onChange, t=(x)=>x }){ return <label className="field"><span>{label}</span>{desc && <em>{desc}</em>}{type==='textarea' ? <textarea rows="5" value={value} onChange={e=>onChange?.(e.target.value)} placeholder={placeholder}/> : type==='select' ? <select value={value} onChange={e=>onChange?.(e.target.value)}>{(options||[]).map(o=><option key={o} value={o}>{t(o)}</option>)}</select> : <input type={type} value={value} onChange={e=>onChange?.(e.target.value)} placeholder={placeholder}/>}</label>; }
function Modal({ title, children, footer, onClose }){ return <div className="modal-backdrop"><section className="modal glass"><header className="modal-header"><h3>{title}</h3><button className="close-x" onClick={onClose}>×</button></header><main className="modal-body">{children}</main>{footer && <footer className="modal-footer">{footer}</footer>}</section></div>; }
function SaveButton({ onClick, children }){ return <div className="form-actions"><button className="btn btn-green" onClick={onClick}>{children}</button></div>; }
function slug(s){ return String(s||'').trim().toLowerCase().replace(/[^a-z0-9]+/g,'_').replace(/^_+|_+$/g,''); }
function formKey(scope,label){ return `${scope}.${slug(label)}`; }
function boolText(v){ return v === 'Yes' || v === true; }
function dmPolicyValue(v){ const key = String(v || 'pairing').trim().toLowerCase(); return ['open','allowlist','pairing'].includes(key) ? key : 'pairing'; }
function setPath(root,path,value){ const parts=path.split('.'); let cur=root; for(let i=0;i<parts.length-1;i++){ cur[parts[i]]=cur[parts[i]]||{}; cur=cur[parts[i]]; } cur[parts.at(-1)] = value; }

export default function HelpTab({ language='en' }){
  const t = (text)=>translate(language, text);
  const [statuses,setStatuses]=useState({}); const [versions,setVersions]=useState({}); const [busy,setBusy]=useState(false); const [busyMessage,setBusyMessage]=useState('Checking status...'); const [toast,setToast]=useState('');
  const [modal,setModal]=useState(null); const [ngrokInfo,setNgrokInfo]=useState(''); const [history,setHistory]=useState([]);
  const [providerFilter,setProviderFilter]=useState(''); const [pluginFilter,setPluginFilter]=useState(''); const [form,setForm]=useState({});
  const step = history[history.length-1] || { type:'gateway-mode' };
  const filteredProviders = useMemo(()=>PROVIDERS.filter(([name])=>name.toLowerCase().includes(providerFilter.toLowerCase())),[providerFilter]);
  const filteredPlugins = useMemo(()=>PLUGINS.filter(p=>p.name.toLowerCase().includes(pluginFilter.toLowerCase())),[pluginFilter]);
  const note = useCallback((m)=>{ setToast(String(m||'Done')); setTimeout(()=>setToast(''),2600); }, []);
  const setVal = (k,v)=>setForm(f=>({...f,[k]:v}));
  const getFormValue = (key, fallback='') => { const value = key.split('.').reduce((cur, part)=>cur?.[part], form); return value ?? form[key] ?? fallback; };
  const setFormPath = (path, value)=>setForm(f=>{ const next = { ...f }; setPath(next, path, value); return next; });
  const providerField = (name,label,fieldName,props={}) => <Field label={t(label)} value={getFormValue(`providers.${name}.${fieldName}`, props.defaultValue ?? '')} onChange={v=>setFormPath(`providers.${name}.${fieldName}`,v)} {...props} desc={props.desc ? t(props.desc) : props.desc} t={t}/>;
  const pluginField = (pluginName,label,props={}) => <Field label={t(label)} value={getFormValue(`pluginSettings.${pluginName}.${label}`, props.defaultValue ?? '')} onChange={v=>setFormPath(`pluginSettings.${pluginName}.${label}`,v)} {...props} desc={props.desc ? t(props.desc) : props.desc} t={t}/>;
  const field = (scope,label,props={}) => <Field label={t(label)} value={form[formKey(scope,label)] ?? props.defaultValue ?? ''} onChange={v=>setVal(formKey(scope,label),v)} {...props} desc={props.desc ? t(props.desc) : props.desc} t={t}/>;
  const save = async (m=t('Saved. Configuration written to openclaw.json.'))=>{
    setBusyMessage('Saving...');
    setBusy(true);
    try {
      const current = await invoke('load_openclaw_config');
      const next = { ...(current || {}) };
      if (form.gatewayMode) setPath(next,'gateway.mode',form.gatewayMode);
      if (form.openclawDir || form.workspaceDir) {
        next.agents = next.agents || {};
        next.agents.defaults = next.agents.defaults || {};
        if (form.openclawDir) next.agents.defaults.openclawDir = form.openclawDir;
        if (form.workspaceDir) next.agents.defaults.workspace = form.workspaceDir;
      }
      const providerName = form.currentProvider;
      if (providerName) {
        const providerForm = form.providers?.[providerName] || {};
        const displayName = form['provider.name'] || providerName;
        const id = slug(displayName);
        const provider = {};
        if (providerForm.apiKey || form['provider.api_key']) provider.apiKey = providerForm.apiKey || form['provider.api_key'];
        if (providerForm.api || form['provider.api']) provider.api = providerForm.api || form['provider.api'];
        if (providerForm.model || form['provider.default_model']) provider.model = providerForm.model || form['provider.default_model'];
        if (providerForm.baseUrl || form['provider.base_url']) provider.baseUrl = providerForm.baseUrl || form['provider.base_url'];
        if (form['provider.endpoint_compatibility']) provider.compatibility = form['provider.endpoint_compatibility'];
        if (providerForm.modelId || form['provider.model_id']) provider.modelId = providerForm.modelId || form['provider.model_id'];
        if (providerForm.contextWindow || form['provider.context_window']) provider.contextWindow = providerForm.contextWindow || form['provider.context_window'];
        if (providerForm.maxTokens || form['provider.max_tokens']) provider.maxTokens = providerForm.maxTokens || form['provider.max_tokens'];
        if (providerForm.reasoning || form['provider.reasoning']) provider.reasoning = (providerForm.reasoning || form['provider.reasoning']) === 'Yes';
        if (providerForm.alias || form['provider.alias']) provider.alias = providerForm.alias || form['provider.alias'];
        if (Object.keys(provider).length) {
          next.models = next.models || {};
          next.models.providers = next.models.providers || {};
          next.models.providers[id] = { ...(next.models.providers[id] || {}), name: displayName, ...provider };
        }
      }
      if (form['web_search.enable'] || form['web_search.search_provider'] || form['web_search.api_key']) {
        setPath(next,'web.search',{
          enabled: boolText(form['web_search.enable'] || 'Yes'),
          provider: form['web_search.search_provider'] || 'Perplexity',
          apiKey: form['web_search.api_key'] || ''
        });
      }
      if (form.webFetch) {
        next.web = next.web || {};
        next.web.fetch = { ...(next.web.fetch || {}), enabled: form.webFetch === 'Yes' };
      }
      if (form['gateway.port'] || form['gateway.bind_mode'] || form['gateway.auth_mode'] || form['gateway.token_password'] || form['gateway.tailscale_exposure']) {
        next.gateway = next.gateway || {};
        if (form['gateway.port']) next.gateway.port = form['gateway.port'];
        if (form['gateway.bind_mode']) next.gateway.bindMode = form['gateway.bind_mode'];
        if (form['gateway.auth_mode']) next.gateway.auth = { ...(next.gateway.auth || {}), mode: form['gateway.auth_mode'], token: form['gateway.token_password'] || next.gateway.auth?.token || '' };
        if (form['gateway.tailscale_exposure']) next.gateway.tailscaleExposure = form['gateway.tailscale_exposure'];
      }
      if (form['daemon.service_action'] || form['daemon.runtime']) {
        next.daemon = { ...(next.daemon || {}), serviceAction: form['daemon.service_action'] || 'Restart', runtime: form['daemon.runtime'] || 'Node' };
      }
      if (form.currentChannel) {
        const id = slug(form.currentChannel);
        next.channels = next.channels || {};
        const channelConfig = {
          ...(next.channels[id] || {}),
          enabled: true,
          botToken: form['channel.bot_token'] || '',
          defaultTo: form['channel.chat_id_target'] || '',
          dmPolicy: dmPolicyValue(form['channel.dm_policy'])
        };
        if (id === 'telegram') {
          if (channelConfig.botToken) delete channelConfig.tokenFile;
          channelConfig.allowFrom = Array.isArray(channelConfig.allowFrom) ? channelConfig.allowFrom : ['*'];
          channelConfig.groups = {
            ...(channelConfig.groups || {}),
            '*': { ...(channelConfig.groups?.['*'] || {}), requireMention: false }
          };
          channelConfig.streaming = { ...(channelConfig.streaming || {}) };
          if (!channelConfig.streaming.mode) channelConfig.streaming.mode = 'off';
          next.plugins = next.plugins || {};
          next.plugins.entries = next.plugins.entries || {};
          next.plugins.entries.telegram = {
            ...(next.plugins.entries.telegram || {}),
            enabled: true,
            config: next.plugins.entries.telegram?.config || {}
          };
        }
        next.channels[id] = channelConfig;
      }
      if (form.currentPlugin) {
        const id = slug(form.currentPlugin);
        next.plugins = next.plugins || {};
        next.plugins.entries = next.plugins.entries || {};
        const plugin = { ...(next.plugins.entries[id] || {}), enabled: true, name: form.currentPlugin };
        Object.entries(form.pluginSettings?.[form.currentPlugin] || {}).forEach(([k,v])=>{ if(v !== '') plugin[slug(k)] = v; });
        Object.entries(form).forEach(([k,v])=>{ if(k.startsWith('plugin.') && v !== '') plugin[k.slice(7)] = v; });
        next.plugins.entries[id] = plugin;
      }
      if (form['skills.install_missing_dependencies'] || form['skills.environment_variables']) {
        next.skills = { ...(next.skills || {}), installMissingDependencies: boolText(form['skills.install_missing_dependencies'] || 'Yes'), environmentVariables: form['skills.environment_variables'] || '' };
      }
      await invoke('save_openclaw_config', { config: next });
      note(m);
      return true;
    } catch(e) {
      note(e);
      return false;
    } finally {
      setBusy(false);
    }
  };
  const push = (s)=>setHistory(h=>[...h,s]); const pop=()=>setHistory(h=>h.length>1?h.slice(0,-1):h);
  const close = ()=>{ setModal(null); setHistory([]); };
  const refreshStatuses = useCallback(async()=>{
    setBusyMessage('Checking status...');
    setBusy(true);
    try{
      const [appStatuses, installed] = await Promise.all([invoke('app_statuses'), invoke('check_tools')]);
      const next = appStatuses || {};
      if (installed && Object.prototype.hasOwnProperty.call(installed, 'node')) {
        next.node = installed.node ? 'Installed' : 'Not Installed';
      }
      setStatuses(next);
    }catch(e){ note(e); } finally{ setBusy(false); }
  }, [note]);
  const refreshVersions = useCallback(async()=>{ setBusyMessage('Checking status...'); setBusy(true); try{ setVersions(await invoke('check_versions') || {}); }catch(e){ note(e); } finally{ setBusy(false); } }, [note]);
  async function startWizard(){ setBusyMessage('Please Wait...'); setBusy(true); try{ await invoke('load_openclaw_config'); }catch(e){ note(e); } finally{ setBusy(false); setModal('wizard'); setHistory([{type:'gateway-mode'}]); } }
  async function openNgrok(){ setBusyMessage('Please Wait...'); setBusy(true); try{ setNgrokInfo(String(await invoke('run_action',{action:'ngrok-view-account'}) || t('No account info.'))); setModal('ngrok'); }catch(e){ setNgrokInfo(String(e)); setModal('ngrok'); } finally{ setBusy(false); } }
  async function removeNgrok(){ setBusyMessage('Please Wait...'); setBusy(true); try{ setNgrokInfo(String(await invoke('run_action',{action:'reset-ngrok-mapping'}) || t('Removed'))); note(t('Ngrok account removed.')); }catch(e){ note(e); } finally{ setBusy(false); } }
  async function browse(key,title){ const p = await dialogOpen({ directory:true, title }); if(p) setVal(key, String(p)); }
  async function runDoctor(){ setBusyMessage('Please Wait...'); setBusy(true); try{ await invoke('run_action',{action:'doctor-fix'}); note(t('Success!')); }catch(e){ note(t('Error! ') + String(e).substring(0,200)); } finally{ setBusy(false); } }
  async function fixGateway(){ setBusyMessage('Please Wait...'); setBusy(true); try{ const res = await invoke('run_action',{action:'fix-gateway'}); note(String(res || t('Gateway fixed.'))); }catch(e){ note(t('Error! ') + String(e).substring(0,240)); } finally{ setBusy(false); } }
  const footer = <><button className="btn btn-ghost" disabled={history.length<=1} onClick={pop}>{t('Back')}</button><button className="btn btn-red" onClick={close}>{t('Cancel')}</button><button className="btn btn-yellow" onClick={pop}>{t('Skip for now')}</button></>;
  const title = step.type==='gateway-mode'?t('Gateway Mode'):step.type==='sections'?t('Section Picker'):step.type==='provider'?(step.name==='Custom Provider'?t('Custom Provider'):`${step.name} ${t('Setup')}`):step.type==='channel'?`${step.name} ${t('Channels')}`:step.type==='plugin'?`${step.name} ${t('Plugins')}`:t(step.title || 'Config Wizard');

  function renderStep(){
    if(step.type==='gateway-mode') return <><p className="muted">{t('Where will the Gateway run?')}</p><div className="card-grid">{GATEWAY_MODES.map(m=><button className="wiz-card" key={m.value} onClick={()=>{ setVal('gatewayMode',m.value); push({type:'sections'}); }}><span className="icon">{m.icon}</span><span><b>{t(m.label)}</b><small>{t(m.desc)}</small></span></button>)}</div></>;
    if(step.type==='sections') return <><p className="muted">{t('Pick a section to configure directly.')}</p><div className="card-grid">{SECTIONS.map(([id,icon,label])=><button className="wiz-card" key={id} onClick={()=>push({type:id,title:label})}><span className="icon">{icon}</span><b>{t(label)}</b></button>)}</div></>;
    if(step.type==='workspace') return <><Field label={t('OpenClaw Directory')} value={form.openclawDir||''} onChange={v=>setVal('openclawDir',v)} placeholder="C:\\Users\\you\\.openclaw"/><button className="btn btn-ghost browse" onClick={()=>browse('openclawDir',t('OpenClaw Directory'))}>{t('Browse')}</button><Field label={t('Workspace Directory')} value={form.workspaceDir||''} onChange={v=>setVal('workspaceDir',v)} placeholder="D:\\.openclaw\\workspace"/><button className="btn btn-ghost browse" onClick={()=>browse('workspaceDir',t('Workspace Directory'))}>{t('Browse')}</button><SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='model') return <><div className="search-box"><input placeholder={t('Search providers...')} value={providerFilter} onChange={e=>setProviderFilter(e.target.value)}/></div><div className="card-grid cols-3">{filteredProviders.map(([name,icon])=><button className="wiz-card center" key={name} onClick={()=>{ setVal('currentProvider',name); push({type:'provider',name}); }}><span className="icon">{icon}</span><b>{name}</b></button>)}</div></>;
    if(step.type==='provider' && step.name==='Custom Provider') return <>{field('provider','Name',{defaultValue:form.currentProvider||''})}{field('provider','Base URL',{placeholder:'https://api.example.com/v1'})}{providerField(step.name,'API Key','apiKey',{type:'password'})}{field('provider','API',{type:'select',options:['openai-completions','openai-responses','anthropic-messages','google-generative-ai'],defaultValue:'openai-completions'})}{field('provider','Endpoint Compatibility',{type:'select',options:['OpenAI','Anthropic','Unknown'],defaultValue:'OpenAI'})}{providerField(step.name,'Model ID','modelId')}{field('provider','Context Window')}{field('provider','Max Tokens')}{field('provider','Reasoning',{type:'select',options:['No','Yes'],defaultValue:'No'})}{field('provider','Alias')}<SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='provider') return <>{providerField(step.name,'API Key','apiKey',{type:'password',placeholder:'sk-...'})}{providerField(step.name,'Default Model','model',{placeholder:'e.g. gpt-4.1'})}<SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='web') return <div className="card-grid"><button className="wiz-card" onClick={()=>push({type:'web-search',title:'Web Search'})}><span className="icon">🔎</span><span><b>{t('Web Search')}</b><small>{t('Enable/configure search provider')}</small></span></button><button className="wiz-card" onClick={()=>push({type:'web-fetch',title:'Web Fetch'})}><span className="icon">📥</span><span><b>{t('Web Fetch')}</b><small>{t('Keyless HTTP fetch for pages')}</small></span></button></div>;
    if(step.type==='web-search') return <>{field('web_search','Enable',{type:'select',options:['Yes','No'],defaultValue:'Yes'})}{field('web_search','Search Provider',{type:'select',options:SEARCH_PROVIDERS,defaultValue:SEARCH_PROVIDERS[0]})}{field('web_search','API Key',{type:'password'})}<SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='web-fetch') return <><Field label={t('Enable Web Fetch')} type="select" options={['Yes','No']} value={form.webFetch||'Yes'} onChange={v=>setVal('webFetch',v)} t={t}/><SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='gateway') return <>{field('gateway','Port',{placeholder:'18789'})}{field('gateway','Bind Mode',{type:'select',options:['Loopback','LAN','Tailnet','Custom IP'],defaultValue:'Loopback'})}{field('gateway','Auth Mode',{type:'select',options:['Token','Password','Trusted Proxy'],defaultValue:'Token'})}{field('gateway','Token/Password',{type:'password'})}{field('gateway','Tailscale Exposure',{type:'select',options:['Off','Serve','Funnel'],defaultValue:'Off'})}<SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='daemon') return <>{field('daemon','Service Action',{type:'select',options:['Restart','Reinstall','Skip'],defaultValue:'Restart'})}{field('daemon','Runtime',{type:'select',options:['Node','Bun'],defaultValue:'Node'})}<SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='channels') return <><p className="muted">{t('Click a channel to configure:')}</p><div className="card-grid cols-3">{CHANNELS.map(c=><button className="wiz-card center" key={c} onClick={()=>{ setVal('currentChannel',c); push({type:'channel',name:c}); }}><b>{c}</b></button>)}</div></>;
    if(step.type==='channel') return <>{field('channel','Bot Token',{type:'password'})}{field('channel','Chat ID/Target')}{field('channel','DM Policy',{type:'select',options:['pairing','allowlist','open'],defaultValue:'pairing'})}<SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='plugins') return <><div className="search-box"><input placeholder={t('Search plugins...')} value={pluginFilter} onChange={e=>setPluginFilter(e.target.value)}/></div><div className="card-grid cols-3">{filteredPlugins.map(p=><button className="wiz-card" key={p.name} onClick={()=>{ setVal('currentPlugin',p.name); push({type:'plugin',name:p.name}); }}><span className="icon">{p.icon}</span><b>{p.name}</b></button>)}</div></>;
    if(step.type==='plugin'){ const p=PLUGINS.find(x=>x.name===step.name); return <>{p.fields.map(([label,type,desc,options])=>pluginField(step.name,label,{key:label,type,desc,options,defaultValue:options?.[0]||''}))}<SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>; }
    if(step.type==='skills') return <>{field('skills','Install Missing Dependencies',{type:'select',options:['Yes','Skip'],defaultValue:'Yes'})}{field('skills','Environment Variables',{type:'textarea',placeholder:'KEY=value\nANOTHER=value'})}<SaveButton onClick={()=>save()}>{t('Save')}</SaveButton></>;
    if(step.type==='health') return <div className="outro"><div className="big-icon">🏥</div><h2>{t('Health Check')}</h2><p>{t('Run OpenClaw doctor to audit configuration, connectivity, and local environment health.')}</p><button className="btn btn-green" onClick={runDoctor}>{t('Run Health Check')}</button></div>;
    if(step.type==='continue') return <div className="outro"><div className="big-icon">🎉</div><h2>{t('All Done!')}</h2><p>{t('Configuration will be saved to openclaw.json')}</p><button className="btn btn-green" onClick={async()=>{ const ok = await save(t('Configuration saved to openclaw.json')); if (ok) close(); }}>{t('Save & Finish')}</button></div>;
    return null;
  }

  return <div className="help-config-page"><style>{css}</style><div className="orb o1"/><div className="orb o2"/><div className="orb o3"/>
    <div className="dashboard-grid">
      <section className="dash-card glass"><h2>OpenClaw</h2><p>{t('Configure workspace, models, web tools, gateway, daemon, channels, plugins, skills, and health.')}</p><div style={{display:'flex',gap:8,flexWrap:'wrap'}}><button className="btn btn-green" onClick={startWizard}>{t('Setup')}</button><button className="btn btn-yellow" onClick={runDoctor}>{t('Doctor')}</button><button className="btn btn-blue" onClick={fixGateway}>{t('Fix Gateway')}</button></div></section>
      <section className="dash-card glass"><h2>Ngrok</h2><p>{t('View the active ngrok account and mapping information.')}</p><div><button className="btn btn-green" onClick={openNgrok}>{t('View Account')}</button></div></section>
      <section className="dash-card glass"><div className="card-head"><h2>{t('Installed Apps')}</h2><button className="btn btn-ghost small" onClick={refreshStatuses}>{t('Refresh')}</button></div><div className="status-list">{APPS.map(a=><div className="status-row" key={a}><span>{titleCase(a)}</span><b className={statusClass(statuses[a])}>{translateState(language, statuses[a]||'not-installed')}</b></div>)}</div></section>
      <section className="dash-card glass"><div className="card-head"><h2>{t('Versions')}</h2><button className="btn btn-ghost small" onClick={refreshVersions}>{t('Refresh')}</button></div><div className="status-list">{VERSION_KEYS.map(k=><div className="status-row" key={k}><span>{titleCase(k)}</span><b>{translateState(language, versions[k]||'Not Installed')}</b></div>)}</div></section>
    </div>
    {modal==='wizard' && <Modal title={title} onClose={close} footer={footer}>{renderStep()}</Modal>}
    {modal==='ngrok' && <Modal title={t('Ngrok Account')} onClose={close} footer={<><button className="btn btn-red" onClick={removeNgrok}>{t('Remove Account')}</button><button className="btn btn-ghost" onClick={close}>{t('Close')}</button></>}><pre className="pre-info">{ngrokInfo}</pre></Modal>}
    {busy && <div className="busy-backdrop"><div className="busy-card glass"><div className="spinner"/><h2>{t('Please Wait')}</h2><p>{t(busyMessage)}</p></div></div>}
    {toast && <div className="toast">{toast}</div>}
  </div>;
}

const css = `

.help-config-page{--text:#eef7ff;--muted:#99a9ba;--green:#39f29a;--red:#ff5f73;--yellow:#ffd166;--cyan:#60d8ff;--border:rgba(255,255,255,.18);--shadow:0 24px 70px rgba(0,0,0,.42)}
.help-config-page{min-height:calc(100vh - 80px);position:relative;overflow:hidden;color:var(--text);padding:22px;border-radius:22px;background:radial-gradient(circle at 20% 10%,#1b3550 0,#0b1220 36%,#070b12 100%);font-family:Inter,ui-sans-serif,system-ui,-apple-system,'Segoe UI',Roboto,Arial,sans-serif}.help-config-page .orb{position:absolute;border-radius:50%;filter:blur(35px);opacity:.35;pointer-events:none;animation:float 12s ease-in-out infinite alternate}.help-config-page .o1{width:330px;height:330px;background:#16f2a8;left:-90px;top:70px}.help-config-page .o2{width:420px;height:420px;background:#5e5cff;right:-150px;top:100px;animation-delay:-4s}.help-config-page .o3{width:280px;height:280px;background:#ffd166;left:42%;bottom:-110px;animation-delay:-7s}@keyframes float{to{transform:translate3d(40px,-28px,0) scale(1.08)}}
.help-config-page .glass{background:linear-gradient(135deg,rgba(255,255,255,.12),rgba(255,255,255,.055));border:1px solid var(--border);box-shadow:var(--shadow);backdrop-filter:blur(22px);-webkit-backdrop-filter:blur(22px);position:relative;overflow:hidden}.help-config-page .glass:before{content:"";position:absolute;inset:0 0 auto;height:1px;background:linear-gradient(90deg,transparent,rgba(255,255,255,.6),transparent)}.help-config-page .dashboard-grid{position:relative;z-index:1;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px}.help-config-page .dash-card{min-height:220px;border-radius:28px;padding:24px;display:flex;flex-direction:column;justify-content:center}.help-config-page .dash-card .btn{align-self:flex-start}.help-config-page .dash-card h2{margin:0 0 10px;font-size:24px}.help-config-page .dash-card p{color:var(--muted);line-height:1.55}.help-config-page .card-head{display:flex;align-items:center;justify-content:space-between;gap:12px}.help-config-page .btn{border:1px solid rgba(255,255,255,.2);color:#fff;border-radius:15px;padding:9px 14px;cursor:pointer;font-weight:800;font-size:12px;min-width:90px;text-align:center;display:inline-flex;align-items:center;justify-content:center;gap:6px;background:rgba(255,255,255,.08);box-shadow:0 8px 20px rgba(0,0,0,.2);transition:.18s;white-space:nowrap}.help-config-page .btn:hover:not(:disabled){transform:translateY(-1px);filter:brightness(1.08)}.help-config-page .btn:disabled{opacity:.45;cursor:not-allowed}.help-config-page .btn-green{background:linear-gradient(135deg,rgba(57,242,154,.92),rgba(20,160,105,.72));color:#062417;border-color:rgba(57,242,154,.4)}.help-config-page .btn-blue{background:linear-gradient(135deg,rgba(96,165,250,.95),rgba(37,99,235,.78));color:#eff6ff;border-color:rgba(96,165,250,.45)}.help-config-page .btn-red{background:linear-gradient(135deg,rgba(255,95,115,.92),rgba(168,37,58,.72));border-color:rgba(255,95,115,.4)}.help-config-page .btn-yellow{background:linear-gradient(135deg,rgba(255,209,102,.95),rgba(191,129,20,.72));color:#2d2100;border-color:rgba(255,209,102,.4)}.help-config-page .btn-ghost{background:rgba(255,255,255,.06)}.help-config-page .btn.small{padding:7px 12px;font-size:12px}.help-config-page .status-list{display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-top:14px}.help-config-page .status-row{display:flex;justify-content:space-between;gap:10px;padding:10px 12px;border-radius:12px;background:rgba(0,0,0,.22);border:1px solid rgba(255,255,255,.08);font-size:13px}.help-config-page .status-row b{text-align:right}.help-config-page .running{color:#39f29a}.help-config-page .installed{color:#60a5fa}.help-config-page .stopped{color:#fb923c}.help-config-page .not-installed{color:#ff5f73}.help-config-page .modal-backdrop,.help-config-page .busy-backdrop{position:fixed;inset:0;background:rgba(2,6,12,.72);display:flex;align-items:center;justify-content:center;z-index:1000;padding:20px}.help-config-page .modal{width:calc(100vw - 40px);max-width:none;max-height:94vh;display:flex;flex-direction:column;padding:0;border-radius:28px}.help-config-page .modal-header{display:flex;align-items:center;justify-content:space-between;padding:16px 16px 10px}.help-config-page .modal-header h3{font-size:22px;font-weight:800;margin:0}.help-config-page .close-x{width:34px;height:34px;border-radius:50%;border:1px solid rgba(255,255,255,.15);background:rgba(255,255,255,.06);color:#fff;font-size:22px;cursor:pointer}.help-config-page .close-x:hover{background:rgba(255,95,115,.25);border-color:rgba(255,95,115,.5)}.help-config-page .modal-body{flex:1;overflow-y:auto;overflow-x:hidden;padding:4px 16px 18px;scrollbar-width:none}.help-config-page .modal-body::-webkit-scrollbar{display:none}.help-config-page .modal-footer{display:flex;align-items:center;justify-content:center;padding:14px 24px;border-top:1px solid rgba(255,255,255,.1);min-height:56px;gap:12px}.help-config-page .muted{color:var(--muted);margin:0 0 16px}.help-config-page .card-grid{display:grid;grid-template-columns:repeat(2,1fr);gap:10px}.help-config-page .card-grid.cols-3{grid-template-columns:repeat(5,1fr)}.help-config-page .wiz-card{padding:12px 14px;border-radius:14px;background:rgba(255,255,255,.06);border:1px solid rgba(255,255,255,.1);cursor:pointer;transition:.18s;display:flex;align-items:center;gap:8px;color:var(--text);text-align:left}.help-config-page .wiz-card:hover{background:rgba(255,255,255,.14);border-color:rgba(96,216,255,.55);transform:translateY(-1px)}.help-config-page .wiz-card.center{justify-content:center;text-align:center}.help-config-page .wiz-card .icon{font-size:20px;flex:0 0 auto}.help-config-page .wiz-card b{font-size:13px}.help-config-page .wiz-card small{display:block;color:var(--muted);font-size:11px;margin-top:2px}.help-config-page .field{display:grid;gap:6px;margin-bottom:14px}.help-config-page .field span{font-size:13px;color:#c9d6e3;font-weight:700}.help-config-page .field em{font-style:normal;color:#99a9ba;font-size:12px}.help-config-page .field input,.help-config-page .field textarea,.help-config-page .field select,.help-config-page .search-box input{width:100%;border:1px solid rgba(255,255,255,.16);border-radius:14px;background:rgba(0,0,0,.22);color:#fff;padding:12px;outline:none;font-size:14px;font-family:inherit}.help-config-page .field input:focus,.help-config-page .field textarea:focus,.help-config-page .field select:focus,.help-config-page .search-box input:focus{border-color:rgba(96,216,255,.65)}.help-config-page .field select option{background:#111827}.help-config-page .search-box{margin-bottom:14px}.help-config-page .browse{margin:-6px 0 14px}.help-config-page .form-actions{text-align:center;margin-top:16px}.help-config-page .outro{text-align:center;padding:20px}.help-config-page .outro .big-icon{font-size:52px}.help-config-page .outro h2{margin:6px 0}.help-config-page .outro p{color:var(--muted);line-height:1.5}.help-config-page .pre-info{white-space:pre-wrap;word-break:break-word;padding:16px;border-radius:16px;background:rgba(0,0,0,.22);border:1px solid rgba(255,255,255,.1);color:#e6f2ff;min-height:140px}.help-config-page .busy-card{width:min(380px,90vw);border-radius:28px;padding:32px;text-align:center}.help-config-page .busy-card h2{margin:12px 0 6px}.help-config-page .busy-card p{color:var(--muted);margin:0}.help-config-page .spinner{width:48px;height:48px;margin:0 auto;border-radius:50%;border:4px solid rgba(255,255,255,.18);border-top-color:var(--green);animation:spin 1s linear infinite}@keyframes spin{to{transform:rotate(360deg)}}.help-config-page .toast{position:fixed;right:22px;bottom:22px;background:rgba(57,242,154,.16);border:1px solid rgba(57,242,154,.38);color:#dfffee;border-radius:16px;padding:14px 18px;box-shadow:var(--shadow);backdrop-filter:blur(18px);z-index:1200}@media(max-width:760px){.help-config-page .dashboard-grid,.help-config-page .card-grid,.help-config-page .card-grid.cols-3,.help-config-page .status-list{grid-template-columns:1fr}.help-config-page .modal{width:100%;max-height:96vh}.help-config-page{padding:12px}}

`;
