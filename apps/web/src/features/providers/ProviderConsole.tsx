import {useEffect,useRef,useState} from 'react';
import * as api from '../../lib/providerApi';
import {BACKEND_MODE} from '../../lib/api';
import type {TextProvider,ProviderInput,TextModel,TextModelInput} from './types';
const emptyProvider=():ProviderInput=>({name:'',protocol:'chat_completions',baseUrl:'',enabled:true,requireApiKey:true,revision:0});
const emptyModel=(providerId=''):TextModelInput=>({providerId,modelId:'',label:'',enabled:false,contextWindow:32768,maxOutputTokens:2048,revision:0});
const labels:Record<string,string>={OK:'连接正常',UPSTREAM_AUTH:'上游认证失败，请检查密钥',UPSTREAM_RATE_LIMIT:'上游请求受限',UPSTREAM_CONNECT:'无法连接上游',UPSTREAM_TIMEOUT:'上游连接超时',UPSTREAM_HTTP:'上游返回错误状态',UPSTREAM_REDIRECT:'上游要求重定向，请核对基础地址',UPSTREAM_FORMAT:'上游模型目录格式不兼容',UPSTREAM_TOO_LARGE:'上游响应超过限制',UPSTREAM_READ:'读取上游响应失败'};
export default function ProviderConsole({tab}:{tab:'providers'|'models'}){
 const [providers,setProviders]=useState<TextProvider[]>([]),[models,setModels]=useState<TextModel[]>([]);
 const [providerId,setProviderId]=useState<string>(),[modelId,setModelId]=useState<string>();
 const [provider,setProvider]=useState(emptyProvider),[model,setModel]=useState<TextModelInput>(()=>emptyModel());
 const [key,setKey]=useState(''),[removeKey,setRemoveKey]=useState(false);
 const [suggestions,setSuggestions]=useState<Record<string,string[]>>({});
 const [busy,setBusy]=useState(false),[error,setError]=useState(''),[notice,setNotice]=useState('');
 const active=useRef(true),pending=useRef(false);
 useEffect(()=>{active.current=true;return()=>{active.current=false;};},[]);
 async function reload(){const [p,m]=await Promise.all([api.listProviders(),api.listTextModels()]);if(!active.current)return;setProviders(p);setModels(m);setModel(current=>current.providerId?current:{...current,providerId:p[0]?.id||''});}
 async function run(action:()=>Promise<void>){if(pending.current)return;pending.current=true;setBusy(true);setError('');setNotice('');try{await action();}catch(e){if(active.current)setError(String((e as Error).message||e));}finally{pending.current=false;if(active.current)setBusy(false);}}
 useEffect(()=>{if(BACKEND_MODE)void run(reload);},[]);
 if(!BACKEND_MODE)return <section className="surface-block"><h2>{tab==='providers'?'供应商管理':'文本模型配置'}</h2><p>请使用 Rust HTTP 模式登录管理员后管理真实供应商。Mock 模式不会存储或发送 API Key。</p></section>;
 function editProvider(p?:TextProvider){setProviderId(p?.id);setProvider(p?{name:p.name,protocol:p.protocol,baseUrl:p.baseUrl,enabled:p.enabled,requireApiKey:p.requireApiKey,revision:p.revision}:emptyProvider());setKey('');setRemoveKey(false);setError('');setNotice('');}
 function editModel(m?:TextModel){setModelId(m?.id);setModel(m?{providerId:m.providerId,modelId:m.modelId,label:m.label,enabled:m.enabled,contextWindow:m.contextWindow,maxOutputTokens:m.maxOutputTokens,revision:m.revision}:emptyModel(providers[0]?.id));setError('');setNotice('');}
 const selected=providers.find(p=>p.id===providerId);
 return <section className="provider-console" aria-label={tab==='providers'?'文本供应商管理':'文本模型配置'}>
  <div className="surface-title"><h2>{tab==='providers'?'文本供应商管理':'文本模型配置'}</h2><button className="btn btn-light" disabled={busy} onClick={()=>void run(reload)}>刷新列表</button></div>
  {error&&<div role="alert" className="provider-alert">{error}<p>如提示版本冲突，请刷新并重新选择要编辑的配置。</p></div>}
  {notice&&<p role="status" className="provider-notice">{notice}</p>}
  {tab==='providers'?<div className="provider-layout"><section className="surface-block"><div className="surface-title"><strong>已保存供应商</strong><button className="btn btn-light" disabled={busy} onClick={()=>editProvider()}>新增供应商</button></div>
   {!providers.length?<p className="empty-caption">还没有供应商，先填写右侧配置并保存。</p>:providers.map(p=><article key={p.id} className="provider-item"><strong>{p.name}</strong><small>{p.baseUrl}</small><p>{p.enabled?'已启用':'已停用'} · {p.hasApiKey?'密钥已保存':'未保存密钥'}</p><p>最近连接：{p.lastTestStatus?(labels[p.lastTestCode||'']||'测试失败'):'未测试'}</p><div className="inline-actions"><button disabled={busy} onClick={()=>editProvider(p)}>编辑</button><button disabled={busy||!p.enabled} onClick={()=>void run(async()=>{const result=await api.testProvider(p.id,p.revision);if(!active.current)return;setSuggestions(s=>({...s,[p.id]:result.models}));await reload();if(active.current)setNotice(`${p.name}：${labels[result.code]||'测试失败'} · ${result.latencyMs} ms${result.ok?` · ${result.models.length} 个模型`:''}`);})}>测试连接</button></div></article>)}
   <p className="empty-caption">连接测试使用已保存配置查询模型目录，不发送生成请求。停用供应商后，其文本模型会从用户目录隐藏。</p>
  </section><form className="surface-block provider-form" onSubmit={e=>{e.preventDefault();void run(async()=>{const saved=await api.saveProvider({...provider,...(key?{apiKey:key}:{}),clearApiKey:removeKey},providerId);if(!active.current)return;editProvider(saved);await reload();if(active.current)setNotice('供应商已保存。可测试连接后到模型配置中开放模型。');});}}>
   <h3>{providerId?'编辑供应商':'新增供应商'}</h3><label>名称<input required maxLength={120} className="input" value={provider.name} disabled={busy} onChange={e=>setProvider({...provider,name:e.target.value})}/></label>
   <label>协议<select className="select" value={provider.protocol} disabled={busy}><option value="chat_completions">Chat Completions 兼容协议</option></select></label>
   <label>基础地址<input required className="input" type="url" placeholder="https://api.example.com/v1" value={provider.baseUrl} disabled={busy} onChange={e=>setProvider({...provider,baseUrl:e.target.value})}/></label><small>填写 API 基础地址，不包含 /models 或 /chat/completions。</small>
   <label className="provider-check"><input type="checkbox" checked={provider.enabled} disabled={busy} onChange={e=>setProvider({...provider,enabled:e.target.checked})}/>启用供应商</label>
   <label className="provider-check"><input type="checkbox" checked={provider.requireApiKey} disabled={busy} onChange={e=>setProvider({...provider,requireApiKey:e.target.checked})}/>需要 API Key</label>
   <label>API Key<input className="input" type="password" autoComplete="new-password" value={key} disabled={busy||removeKey} onChange={e=>setKey(e.target.value)} placeholder={selected?.hasApiKey?'已保存；留空保留原密钥':'输入密钥'}/></label>
   <small>已保存的密钥不会回传。更换基础地址时需重新输入或移除密钥。</small>
   {selected?.hasApiKey&&<label className="provider-check"><input type="checkbox" checked={removeKey} disabled={busy} onChange={e=>{setRemoveKey(e.target.checked);if(e.target.checked)setKey('');}}/>移除已保存密钥（需同时关闭“需要 API Key”）</label>}
   <button className="btn btn-dark" disabled={busy} type="submit">{busy?'处理中…':'保存供应商'}</button>
  </form></div>:<div className="provider-layout"><section className="surface-block"><div className="surface-title"><strong>文本模型</strong><button className="btn btn-light" disabled={busy} onClick={()=>editModel()}>新增模型</button></div>
   {!models.length?<p className="empty-caption">先添加供应商，再配置上游模型 ID。</p>:models.map(m=><article className="provider-item" key={m.id}><strong>{m.label}</strong><small>{m.modelId} · {providers.find(p=>p.id===m.providerId)?.name||'供应商'}</small><p>{m.enabled?'已开放':'未开放'} · 上下文 {m.contextWindow.toLocaleString()} · 输出上限 {m.maxOutputTokens.toLocaleString()}</p><p>{providers.find(p=>p.id===m.providerId)?.enabled?'':'供应商已停用，用户目录中不可见'}</p><button className="text-action" disabled={busy} onClick={()=>editModel(m)}>编辑模型</button></article>)}
   <p className="empty-caption">只有模型和所属供应商均启用，且凭据满足要求，用户端才可读取该模型。</p>
  </section><form className="surface-block provider-form" onSubmit={e=>{e.preventDefault();void run(async()=>{const saved=await api.saveTextModel(model,modelId);if(!active.current)return;editModel(saved);await reload();if(active.current)setNotice('文本模型已保存。普通对话发送将在下一阶段接入。');});}}>
   <h3>{modelId?'编辑文本模型':'新增文本模型'}</h3>
   <label>所属供应商<select className="select" required value={model.providerId} disabled={busy} onChange={e=>setModel({...model,providerId:e.target.value})}><option value="">请选择</option>{providers.map(p=><option key={p.id} value={p.id}>{p.name}{p.enabled?'':'（已停用）'}</option>)}</select></label>
   <button className="btn btn-light" type="button" disabled={busy||!providers.find(p=>p.id===model.providerId)?.enabled} onClick={()=>void run(async()=>{const p=providers.find(p=>p.id===model.providerId);if(!p)return;const result=await api.testProvider(p.id,p.revision);if(!active.current)return;setSuggestions(s=>({...s,[p.id]:result.models}));await reload();if(active.current)setNotice(`${labels[result.code]||'测试失败'} · ${result.models.length} 个模型`);})}>获取上游模型</button>
   <label>上游模型 ID<input className="input" required maxLength={256} list="upstream-text-models" value={model.modelId} disabled={busy} onChange={e=>setModel({...model,modelId:e.target.value})}/></label><datalist id="upstream-text-models">{(suggestions[model.providerId]||[]).map(id=><option key={id} value={id}/>)}</datalist>
   <label>展示名称<input className="input" required maxLength={120} value={model.label} disabled={busy} onChange={e=>setModel({...model,label:e.target.value})}/></label>
   <label>上下文容量（tokens）<input className="input" type="number" min={1024} max={4000000} required value={model.contextWindow} disabled={busy} onChange={e=>setModel({...model,contextWindow:Number(e.target.value)})}/></label>
   <label>最大输出（tokens）<input className="input" type="number" min={1} max={model.contextWindow-1} required value={model.maxOutputTokens} disabled={busy} onChange={e=>setModel({...model,maxOutputTokens:Number(e.target.value)})}/></label>
   <label className="provider-check"><input type="checkbox" checked={model.enabled} disabled={busy} onChange={e=>setModel({...model,enabled:e.target.checked})}/>向用户开放</label>
   <button className="btn btn-dark" disabled={busy||!providers.length} type="submit">{busy?'处理中…':'保存文本模型'}</button>
  </form></div>}
 </section>;
}