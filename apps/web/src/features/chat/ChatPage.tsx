import {useEffect,useRef,useState} from 'react';
import {ArrowUp,Plus,MessageSquare,CornerDownRight,Sparkles} from 'lucide-react';
import * as api from '../../lib/api';
import {useDemoDB} from '../../lib/store';
import type {AgentCapabilities} from './types';

type Message={id:string;role:'user'|'assistant';body:string;timestamp:string};
const initial:Message[]=[{id:'intro',role:'assistant',body:'这里是创作对话界面演示。你可以输入想法查看模拟建议；真实模型和画布工具尚未接入。',timestamp:''}];

export default function ChatPage(){
 const userId=useDemoDB().currentUserId;
 // Identity changes discard another user's transient demo conversation.
 return <ChatWorkspace key={userId}/>;
}
function ChatWorkspace(){
 const [threads,setThreads]=useState<Record<string,Message[]>>({'创作方案讨论':api.BACKEND_MODE?[]:initial});
 const [thread,setThread]=useState('创作方案讨论');
 const [input,setInput]=useState('');
 const [loading,setLoading]=useState(false);
 const [capabilities,setCapabilities]=useState<AgentCapabilities|null>(null);
 const [capabilityError,setCapabilityError]=useState('');
 const [attempt,setAttempt]=useState(0);
 const bottom=useRef<HTMLDivElement>(null);
 const replyTimer=useRef<ReturnType<typeof setTimeout>|null>(null);
 const msgs=threads[thread]||[];
 useEffect(()=>{bottom.current?.scrollIntoView({behavior:'smooth'});},[thread,msgs.length]);
 useEffect(()=>{
  let live=true;setCapabilities(null);setCapabilityError('');
  void api.getAgentCapabilities().then(value=>{if(live)setCapabilities(value);})
   .catch(()=>{if(live)setCapabilityError('无法读取 Agent 状态，请检查后端连接后重试。');});
  return()=>{live=false;};
 },[attempt]);
 useEffect(()=>()=>{if(replyTimer.current!==null)clearTimeout(replyTimer.current);},[]);
 const send=()=>{
  const text=input.trim();if(!text||loading||api.BACKEND_MODE)return;
  const target=thread;
  const timestamp=()=>new Date().toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'});
  setThreads(t=>({...t,[target]:[...(t[target]||[]),{id:crypto.randomUUID(),role:'user',body:text,timestamp:timestamp()}]}));
  setInput('');setLoading(true);
  replyTimer.current=setTimeout(()=>{
   const isVideo=/视频|镜头|运镜|短片/.test(text),isImage=/图片|海报|生图|画面/.test(text);
   const reply=isVideo?'模拟建议：先整理分镜和参考帧，再到画布设置比例、画质和时长。当前没有调用生成工具。'
    :isImage?'模拟建议：先明确主体、光线、构图和风格，再到画布设置画质、比例和参考素材。当前没有调用生成工具。'
    :'模拟建议：补充目标受众、画面风格和交付形式，有助于整理创作方案。';
   setThreads(t=>({...t,[target]:[...(t[target]||[]),{id:crypto.randomUUID(),role:'assistant',body:reply,timestamp:timestamp()}]}));
   setLoading(false);replyTimer.current=null;
  },850);
 };
 const features:[keyof AgentCapabilities['features'],string][]=[['chat','真实模型对话'],['history','服务端历史'],['tools','创作工具'],['approvals','工具审批'],['compaction','上下文压缩'],['memory','长期记忆']];
 return <div className="chat-layout">
  <aside className="thread-sidebar">
   <div className="mini-head"><strong>当前演示会话</strong><button className="icon-btn" disabled={api.BACKEND_MODE} onClick={()=>{const id=`新会话 ${Object.keys(threads).length}`;setThreads(t=>({...t,[id]:[]}));setThread(id);}} aria-label="创建对话"><Plus size={17}/></button></div>
   <div className="thread-list">{Object.keys(threads).map(name=><button key={name} className={thread===name?'selected':''} onClick={()=>setThread(name)}><MessageSquare size={15}/><span>{name}</span></button>)}</div>
   <p className="empty-caption">{api.BACKEND_MODE?'服务端会话尚未接入。':'演示会话仅保留在当前页面，刷新后清空。'}</p>
  </aside>
  <section className="conversation">
   <div className="chat-header"><div><strong>{thread}</strong><p>{api.BACKEND_MODE?'Rust 模式 · 真实对话尚未接入':'Mock 模式 · 模拟建议'}</p></div></div>
   {capabilityError&&<div className="chat-status-banner" role="alert">{capabilityError}<button className="text-action" onClick={()=>setAttempt(a=>a+1)}>重试</button></div>}
   {api.BACKEND_MODE&&<div className="chat-status-banner" role="status">对话内核正在接入。接通模型与会话服务后开放发送。</div>}
   <div className="chat-messages">{msgs.map(m=><div key={m.id} className={`chat-msg ${m.role}`}><div className="message-avatar">{m.role==='assistant'?<Sparkles size={16}/>:<span>我</span>}</div><div><div className="message-bubble">{m.body}</div><small>{m.role==='assistant'?'Atelier 演示':'你'}{m.timestamp&&` · ${m.timestamp}`}</small></div></div>)}
    {loading&&<div className="chat-msg assistant"><div className="message-avatar"><Sparkles size={16}/></div><div className="message-bubble">正在整理模拟建议…</div></div>}
    <div ref={bottom}/>
   </div>
   <div className="composer"><textarea aria-label="创作对话输入" placeholder={api.BACKEND_MODE?'真实对话尚未接入':'描述你的创意，查看模拟建议…'} disabled={api.BACKEND_MODE} value={input} onChange={e=>setInput(e.target.value)} onKeyDown={e=>{if(e.key==='Enter'&&(e.ctrlKey||e.metaKey)){e.preventDefault();send();}}}/>
    <div className="composer-actions"><span>{api.BACKEND_MODE?'模型待接入':'模拟回复 · Ctrl + Enter 发送'}</span><button className="btn btn-dark" onClick={send} disabled={api.BACKEND_MODE||!input.trim()||loading}><ArrowUp size={15}/> 发送</button></div>
   </div>
  </section>
  <aside className="context-sidebar"><div className="mini-head"><strong>Agent 接入状态</strong></div>
   <div className="context-card"><div className="context-icon"><CornerDownRight size={15}/></div><strong>对话理解</strong><p>{api.BACKEND_MODE?'后续由主模型结合当前项目和可用工具理解创作需求。':'当前仅按关键词生成模拟建议。'}</p></div>
   <div className="context-card"><strong>上下文窗口</strong><p>尚无真实模型用量数据</p></div>

   <div className="context-card"><strong>功能接入</strong><dl className="agent-capabilities">{features.map(([key,label])=><div key={key}><dt>{label}</dt><dd>{capabilities?(capabilities.features[key]?'已接通':'未接入'):capabilityError?'未知':'读取中'}</dd></div>)}</dl></div>
  </aside>
 </div>;
}