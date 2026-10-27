// Three presentation modes, one Rust engine and one input/save session.
const bindings = new WeakMap();
export function preferredRendererView(location, storage, legacyEnabled = false) {
  const requested = new URLSearchParams(location.search).get('view');
  if (['2d','3d','tui'].includes(requested)) return requested;
  if (location.pathname.startsWith('/tui')) return 'tui';
  const saved = storage.getItem('crystal.display.mode');
  return ['2d','3d','tui'].includes(saved) ? saved : legacyEnabled ? '3d' : '2d';
}
export function mountViewNavigation({ document, location, history, setMode, currentMode }) {
  const nav = document.querySelector('#view-navigation');
  const existing = bindings.get(nav);
  if (existing) {
    existing.setMode = setMode; existing.currentMode = currentMode;
    existing.sync(); return existing.sync;
  }
  const binding = { setMode, currentMode };
  const sync = () => {
    for (const link of nav.querySelectorAll('[data-view]')) {
      if (link.dataset.view === binding.currentMode()) link.setAttribute('aria-current','page');
      else link.removeAttribute('aria-current');
    }
  };
  binding.sync = sync; bindings.set(nav,binding);
  nav.addEventListener('click', event => {
    const link = event.target.closest('[data-view]');
    if (!link || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    binding.setMode(link.dataset.view);
    const url = new URL(location.href); url.searchParams.set('view',link.dataset.view);
    history.replaceState(null,'',url); sync();
  });
  sync(); return sync;
}

export function mountRendererViews(wasm, viewControls, { document, window, canvas, startIn3D = false }) {
  const {location,history,localStorage:storage}=window;
  if(document.documentElement.classList.contains('flygon-embedded')) {
    wasm.crystal_configure_text_view(false,80,40,true);
    return;
  }
  const surface=document.querySelector('#tui-surface');
  const terminal=document.querySelector('#tui-screen');
  const output=document.querySelector('#tui-canvas');
  const styleButton=document.querySelector('#tui-style-toggle');
  const legacyToggle=document.querySelector('#view-toggle');
  let mode='2d', painted=storage.getItem('geothite.tui.view')!=='text';
  let painter, dispose, timer, previousHtml, previousDetail, currentColumns, currentRows;
  let sync=()=>{};
  const configure=()=>{
    const coarse=window.matchMedia('(pointer: coarse)').matches;
    const bounds=surface.getBoundingClientRect?.();
    const width=bounds?.width||window.innerWidth;
    const height=Math.max(180,bounds?.height||(window.innerHeight-72-(coarse?180:0)));
    const cell=Math.min(11,width/40,height/52);
    currentColumns=Math.max(40,Math.min(128,Math.floor(width/cell)));
    currentRows=Math.max(26,Math.min(70,Math.floor(height/(cell*2))));
    surface.style.setProperty('--tui-width',`${currentColumns*cell}px`);
    wasm.crystal_configure_text_view(mode==='tui'&&!document.hidden,currentColumns,currentRows,painted);
  };
  const sample=()=>{
    clearTimeout(timer);
    if(mode!=='tui'||document.hidden)return;
    const raw=wasm.crystal_text_view_frame();
    if(raw&&painter){
      const frame=JSON.parse(raw);
      if(frame.error){terminal.textContent=frame.error;terminal.style.opacity='1';output.hidden=true;}
      else {
        output.hidden=false;terminal.style.opacity='';
        if(frame.html!==previousHtml||frame.detail?.html!==previousDetail){
          terminal.innerHTML=frame.html;
          dispose?.();dispose=painter(output,terminal,frame.detail??undefined);
          previousHtml=frame.html;previousDetail=frame.detail?.html;
        } else if(frame.detail?.sizes) dispose?.updateDots(frame.detail.sizes);
        // Read-only evidence from the renderer's real controller snapshot.
        surface.dataset.frame=String(frame.frame);
      }
    }
    timer=window.setTimeout(sample,100);
  };
  const setMode=value=>{
    if(!['2d','3d','tui'].includes(value))return;
    mode=value;
    viewControls.setEnabled(mode==='3d');
    document.body.dataset.view=mode;
    surface.hidden=mode!=='tui';styleButton.hidden=mode!=='tui';
    styleButton.textContent=painted?'Text':'Paint';
    styleButton.setAttribute('aria-label',painted?'Switch TUI to text':'Switch TUI to painted');
    canvas.setAttribute('aria-hidden',String(mode==='tui'));
    storage.setItem('crystal.display.mode',mode);
    configure();
    clearTimeout(timer);
    if(mode==='tui'){
      import('./tui/ascii-frame.js').then(module=>{painter=module.paintTerminal;sample();});
    } else {dispose?.();dispose=undefined;previousHtml=previousDetail=undefined;}
    sync();canvas.focus({preventScroll:true});
  };
  sync=mountViewNavigation({document,location,history,setMode,currentMode:()=>mode});
  styleButton.addEventListener('click',()=>{
    painted=!painted;storage.setItem('geothite.tui.view',painted?'painted':'text');
    previousHtml=previousDetail=undefined;setMode('tui');
  });
  legacyToggle.addEventListener('click',()=>{setMode(legacyToggle.getAttribute('aria-pressed')==='true'?'3d':'2d');const url=new URL(location.href);url.searchParams.set('view',mode);history.replaceState(null,'',url);});
  surface.addEventListener('pointerdown',()=>canvas.focus({preventScroll:true}));
  window.addEventListener('resize',()=>{previousHtml=previousDetail=undefined;configure();});
  document.addEventListener('visibilitychange',()=>{configure();sample();});
  window.addEventListener('pagehide',()=>{clearTimeout(timer);dispose?.();});
  const explicit=new URLSearchParams(location.search).get('view');
  setMode(startIn3D&&!['2d','3d','tui'].includes(explicit)?'3d':preferredRendererView(location,storage,legacyToggle.getAttribute('aria-pressed')==='true'));
  return {setMode};
}
