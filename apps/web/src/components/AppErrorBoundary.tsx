import React from 'react';

type Props = {children: React.ReactNode};
type State = {error: Error | null};

/** 开发期 React 渲染错误提示，避免用户只看到空白页面。 */
export default class AppErrorBoundary extends React.Component<Props, State> {
  state: State = {error: null};
  static getDerivedStateFromError(error: Error): State { return {error}; }
  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('[Atelier] React rendering failed', error, info.componentStack);
  }
  render() {
    if (!this.state.error) return this.props.children;
    const dev = ['localhost','127.0.0.1'].includes(window.location.hostname);
    return <main style={{fontFamily:'Segoe UI, sans-serif',padding:36,background:'#fafafa',minHeight:'100vh',color:'#252930'}}>
      <div style={{maxWidth:760,margin:'50px auto',background:'#fff',border:'1px solid #ddd',borderRadius:12,padding:28}}>
        <h2 style={{marginTop:0}}>Atelier 页面加载遇到错误</h2>
        <p>这是前端 React 运行错误，与 PostgreSQL 是否启动无关。请按 F12 查看 Console 的第一条红色报错。</p>
        {dev && <pre style={{whiteSpace:'pre-wrap',overflowWrap:'anywhere',background:'#f5f5f5',padding:15,borderRadius:6}}>{String(this.state.error.stack || this.state.error.message)}</pre>}
        <button onClick={()=>window.location.reload()} style={{padding:'10px 17px',borderRadius:7,border:'1px solid #ddd',cursor:'pointer'}}>重新加载</button>
        <button onClick={()=>{if(window.confirm('清除本地 Mock 数据并重新加载？未导出的画布更改将丢失。')) { localStorage.removeItem('atelier-react-v1');window.location.reload();}}} style={{marginLeft:9,padding:'10px 17px',borderRadius:7,border:'1px solid #ddd',cursor:'pointer'}}>重置演示数据</button>
      </div>
    </main>;
  }
}
