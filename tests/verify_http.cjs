const {createRequire}=require('node:module');
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const assert=require('node:assert/strict');
const ts=createRequire(path.join(__dirname,'../apps/web/package.json'))('typescript');
const web=path.join(__dirname,'../apps/web/src/lib');
const storage=new Map(),cache={},calls=[];
let response={ok:true,json:async()=>({})};
const sandbox={console,crypto:require('node:crypto').webcrypto,Headers,Date,Math,JSON,Set,Error,
 localStorage:{getItem:k=>storage.get(k)||null,setItem:(k,v)=>storage.set(k,v)},
 fetch:async(url,options)=>{calls.push({url,options});return response;}};
function load(name){
 if(cache[name])return cache[name];
 const source=fs.readFileSync(path.join(web,name+'.ts'),'utf8').replaceAll('import.meta.env','({VITE_API_MODE:"http"})');
 const code=ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
 const mod={exports:{}};
 vm.runInNewContext('(function(require,module,exports){'+code+'})',sandbox,{filename:name+'.ts'})(
  ref=>{if(ref.startsWith('./'))return load(ref.slice(2));throw Error('Unexpected import '+ref);},mod,mod.exports);
 cache[name]=mod.exports;return mod.exports;
}
(async()=>{
 const api=load('api'),db=load('mockDb'),me={...db.currentUser()},project=db.userProjects()[0];
 response={ok:true,json:async()=>({...project,revision:project.revision+1})};
 await api.saveGraph(project.id,project.graph,project.revision);
 let request=calls.at(-1);
 assert.equal(request.url,'/api/v1/projects/'+project.id+'/graph');
 assert.equal(request.options.headers.get('Content-Type'),'application/json');
 assert.equal(request.options.headers.get('If-Match'),String(project.revision));
 assert.equal(request.options.credentials,'include');
 assert.equal(JSON.parse(request.options.body).revision,project.revision);
 const job={id:'remote-job',ownerId:me.id,projectId:project.id,cost:18,status:'queued',resultAssetIds:[]};
 response={ok:true,json:async()=>job};
 await api.createJob({projectId:project.id,prompt:'sunrise'},'repeated-key');
 request=calls.at(-1);
 assert.equal(request.options.headers.get('Content-Type'),'application/json');
 assert.equal(request.options.headers.get('Idempotency-Key'),'repeated-key');
 await api.createJob({projectId:project.id,prompt:'sunrise'},'repeated-key');
 assert.equal(db.userJobs().filter(j=>j.id===job.id).length,1);
 assert.equal(db.currentUser().credits,me.credits,'job cache must not invent balance changes');
 response={ok:false,status:409,text:async()=>'stale revision'};
 await assert.rejects(()=>api.saveGraph(project.id,project.graph,0),/409: stale revision/);
 const capabilities={contractVersion:1,mode:'http',core:{toolArgumentValidation:true,contextBudgetPlanning:true},
  features:{chat:false,history:false,tools:false,approvals:false,compaction:false,memory:false}};
 response={ok:true,json:async()=>capabilities};
 assert.equal(await api.getAgentCapabilities(),capabilities);
 assert.equal(calls.at(-1).url,'/api/v1/agent/capabilities');
 assert.equal(calls.at(-1).options.credentials,'include');
 response={ok:false,status:401,text:async()=>'missing session'};
 await assert.rejects(()=>api.getAgentCapabilities(),/401: missing session/,'HTTP failures must not fall back to simulated Agent');
 const seedLocal=storage.get('atelier-react-v1');
 const wallet={available:450,ledger:[{id:'entry-1',amount:-18,reason:'hold',at:'2026-10-21T00:00:00Z',jobId:job.id}]};
 sandbox.fetch=async(url)=>({ok:true,json:async()=>({
  '/api/v1/me':me,'/api/v1/projects':[project],'/api/v1/generation/jobs':[job],
  '/api/v1/assets':[],'/api/v1/model-catalog':[], '/api/v1/wallet':wallet
 }[url])});
 await api.refreshServerView();
 assert.equal(db.currentUser().credits,450);
 assert.equal(db.snapshot().ledger.length,1);
 assert.equal(db.snapshot().ledger[0].userId,me.id,'server ledger belongs to current session user');
 assert.equal(db.snapshot().ledger[0].amount,-18);
 assert.equal(storage.get('atelier-react-v1'),seedLocal,'HTTP snapshots must not overwrite local demo data');
 db.hydrateRemote({me:{...me,id:'another-user'},projects:[],jobs:[],assets:[],models:[]});
 assert.equal(db.snapshot().ledger.length,0,'identity refresh must clear previous wallet entries');
 const mockSource=fs.readFileSync(path.join(web,'api.ts'),'utf8').replaceAll('import.meta.env','({VITE_API_MODE:"mock"})');
 const mockCode=ts.transpileModule(mockSource,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
 const mockModule={exports:{}};
 sandbox.fetch=async()=>{throw Error('Mock capabilities must not call the network');};
 vm.runInNewContext('(function(require,module,exports){'+mockCode+'})',sandbox)(()=>db,mockModule,mockModule.exports);
 const mockCapabilities=await mockModule.exports.getAgentCapabilities();
 assert.equal(mockCapabilities.mode,'mock');
 assert.ok(Object.values(mockCapabilities.features).every(enabled=>enabled===false));
 assert.ok(Object.values(mockCapabilities.core).every(enabled=>enabled===false));
 console.log('HTTP adapter OK: JSON headers, revision, idempotency, errors, wallet ledger, cache isolation and Agent capabilities');
})().catch(error=>{console.error(error);process.exitCode=1;});