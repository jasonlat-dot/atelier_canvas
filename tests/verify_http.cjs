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
 const providerApi=load('providerApi');
 const providerView={id:'p1',name:'Test',hasApiKey:true,revision:1};
 sandbox.fetch=async(url,options)=>{calls.push({url,options});return {ok:true,json:async()=>providerView};};
 const beforeProviderStorage=storage.get('atelier-react-v1');
 await providerApi.saveProvider({name:'Test',protocol:'chat_completions',baseUrl:'https://example.test/v1',enabled:true,requireApiKey:true,revision:0,apiKey:'synthetic-test-secret'});
 assert.equal(calls.at(-1).url,'/api/v1/admin/providers');
 assert.equal(calls.at(-1).options.method,'POST');
 assert.equal(calls.at(-1).options.headers.get('Content-Type'),'application/json');
 await providerApi.saveProvider({name:'Test',protocol:'chat_completions',baseUrl:'https://example.test/v1',enabled:false,requireApiKey:true,revision:1},'p1');
 assert.equal(calls.at(-1).options.method,'PUT');
 assert.equal(JSON.parse(calls.at(-1).options.body).apiKey,undefined,'blank edit must preserve key on server');
 await providerApi.testProvider('p1',2);assert.equal(JSON.parse(calls.at(-1).options.body).revision,2);
 assert.equal(calls.at(-1).url,'/api/v1/admin/providers/p1/test');
 await providerApi.listTextModelCatalog();assert.equal(calls.at(-1).url,'/api/v1/text-model-catalog');
 assert.equal(storage.get('atelier-react-v1'),beforeProviderStorage,'provider credentials must never enter demo persistence');
 sandbox.fetch=async()=>({ok:false,status:403,text:async()=>'forbidden'});
 await assert.rejects(()=>providerApi.listProviders(),/403: forbidden/);
 const mockSource=fs.readFileSync(path.join(web,'api.ts'),'utf8').replaceAll('import.meta.env','({VITE_API_MODE:"mock"})');
 const mockCode=ts.transpileModule(mockSource,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
 const mockModule={exports:{}};
 sandbox.fetch=async()=>{throw Error('Mock capabilities must not call the network');};
 vm.runInNewContext('(function(require,module,exports){'+mockCode+'})',sandbox)(()=>db,mockModule,mockModule.exports);
 const mockProviderCode=ts.transpileModule(fs.readFileSync(path.join(web,'providerApi.ts'),'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
 const mockProviderModule={exports:{}};
 vm.runInNewContext('(function(require,module,exports){'+mockProviderCode+'})',sandbox)(()=>mockModule.exports,mockProviderModule,mockProviderModule.exports);
 await assert.rejects(()=>mockProviderModule.exports.listProviders(),/Rust HTTP/);
 assert.equal((await mockProviderModule.exports.listTextModelCatalog()).length,0);
 const mockCapabilities=await mockModule.exports.getAgentCapabilities();
 assert.equal(mockCapabilities.mode,'mock');
 assert.ok(Object.values(mockCapabilities.features).every(enabled=>enabled===false));
 assert.ok(Object.values(mockCapabilities.core).every(enabled=>enabled===false));
 console.log('HTTP adapter OK: JSON headers, revision, idempotency, errors, wallet ledger, cache isolation and Agent capabilities');
})().catch(error=>{console.error(error);process.exitCode=1;});