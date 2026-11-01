import test from 'node:test';
import assert from 'node:assert/strict';
import { JSDOM } from 'jsdom';
import { mountRendererViews, mountViewNavigation, preferredRendererView } from './view-navigation.js';
import { mountViewToggle } from './view-toggle.js';
const markup='<nav id="view-navigation"><a href="/?view=2d" data-view="2d">2D</a><a href="/?view=3d" data-view="3d">3D</a><a href="/?view=tui" data-view="tui">TUI</a></nav>';
test('Minnesota ignores 3D URLs and both generations of stored display preferences',()=>{
 const storage={getItem:()=> '3d'};
 assert.equal(preferredRendererView({search:'?view=3d',pathname:'/'},storage,true,false),'2d');
 assert.equal(preferredRendererView({search:'',pathname:'/'},storage,true,false),'2d');
 assert.equal(preferredRendererView({search:'?view=tui',pathname:'/'},storage,true,false),'tui');
 assert.equal(preferredRendererView({search:'?view=3d',pathname:'/'},storage,true),'3d');
});

test('Minnesota hides 3D navigation and rejects clicks after loading-time rebinding',()=>{
 const dom=new JSDOM(markup,{url:'https://game.test/'});
 const {document,location,history}=dom.window;let mode='2d';
 const settings={document,location,history,currentMode:()=>mode,setMode:value=>{mode=value;}};
 mountViewNavigation(settings);
 mountViewNavigation({...settings,allow3D:false});
 const link=document.querySelector('[data-view="3d"]');
 assert.equal(link.hidden,true);
 link.click();assert.equal(mode,'2d');assert.equal(location.search,'');
 link.dispatchEvent(new dom.window.MouseEvent('click',{bubbles:true,cancelable:true,ctrlKey:true}));
 assert.equal(mode,'2d');
 document.querySelector('[data-view="tui"]').click();assert.equal(mode,'tui');
 dom.window.close();
});

for (const compiled3D of [true,false]) test(`Minnesota browser stays 2D through preview and API requests (voxel build: ${compiled3D})`,()=>{
 const dom=new JSDOM(markup.replace('</nav>','<button id="tui-style-toggle"></button></nav>')+
  '<canvas></canvas><button id="view-toggle"></button><details data-voxel-controls></details>'+
  '<div id="tui-surface"><pre id="tui-screen"></pre><canvas id="tui-canvas"></canvas></div>',
  {url:'https://game.test/?view=3d&multiplayer=off'});
 const {window}=dom;const {document,localStorage:storage}=window;
 window.matchMedia=()=>({matches:false});
 storage.setItem('crystal.display.mode','3d');storage.setItem('crystal.display.voxel','true');
 const enabled=[];
 const wasm={crystal_voxel_view_allowed:()=>false,crystal_configure_text_view:()=>{},
  ...(compiled3D?{crystal_set_voxel_view:value=>enabled.push(value),crystal_set_voxel_camera:()=>{}}:{})};
 const canvas=document.querySelector('canvas'),button=document.querySelector('#view-toggle');
 const controls=mountViewToggle(wasm,{button,canvas,storage,startIn3D:true});
 const renderer=mountRendererViews(wasm,controls,{document,window,canvas,startIn3D:true});
 renderer.setMode('3d');controls.setEnabled(true);button.click();
 assert.equal(enabled.every(value=>value===false),true);
 assert.equal(button.hidden,true);assert.equal(button.disabled,true);
 assert.equal(document.querySelector('[data-view="3d"]').hidden,true);
 assert.equal(document.querySelector('[data-voxel-controls]').hidden,true);
 assert.equal(document.body.dataset.view,'2d');
 assert.equal(storage.getItem('crystal.display.mode'),'2d');
 assert.equal(new URL(window.location.href).searchParams.get('view'),'2d');
 assert.equal(new URL(window.location.href).searchParams.get('multiplayer'),'off');
 dom.window.close();
});
test('all three choices swap the current renderer without navigating or replacing the session',()=>{
 const dom=new JSDOM(markup,{url:'https://game.test/?multiplayer=off'});
 const {document,location,history}=dom.window;
 let mode='2d';const swaps=[];
 mountViewNavigation({document,location,history,currentMode:()=>mode,setMode:value=>{mode=value;swaps.push(value);}});
 for(const value of ['3d','tui','2d']){
  const event=new dom.window.MouseEvent('click',{bubbles:true,cancelable:true});
  assert.equal(document.querySelector(`[data-view="${value}"]`).dispatchEvent(event),false);
  assert.equal(location.pathname,'/');
  assert.equal(new URL(location.href).searchParams.get('multiplayer'),'off');
  assert.equal(new URL(location.href).searchParams.get('view'),value);
  assert.equal(document.querySelector('[aria-current]').dataset.view,value);
 }
 assert.deepEqual(swaps,['3d','tui','2d']);dom.window.close();
});
test('direct TUI routes stay in place when swapping and modified clicks retain normal link behavior',()=>{
 for(const route of ['/tui','/tui/']){
  const dom=new JSDOM(markup,{url:'https://game.test'+route});
  const {document,location,history}=dom.window;let mode='tui';
  mountViewNavigation({document,location,history,currentMode:()=>mode,setMode:value=>{mode=value;}});
  assert.equal(document.querySelector('[aria-current]').dataset.view,'tui');
  const link=document.querySelector('[data-view="2d"]');
  const event=new dom.window.MouseEvent('click',{bubbles:true,cancelable:true,ctrlKey:true});
  link.dispatchEvent(event);assert.equal(event.defaultPrevented,false);assert.equal(mode,'tui');
  link.dispatchEvent(new dom.window.MouseEvent('click',{bubbles:true,cancelable:true}));
  assert.equal(location.pathname,route);assert.equal(mode,'2d');dom.window.close();
 }
});

test('loading-time selection rebinds to the live renderer without duplicate clicks',()=>{
 const dom=new JSDOM(markup,{url:'https://game.test/'});
 const {document,location,history}=dom.window;let mode='2d',pending=0,live=0;
 mountViewNavigation({document,location,history,currentMode:()=>mode,setMode:value=>{mode=value;pending++;}});
 document.querySelector('[data-view="tui"]').click();
 assert.equal(pending,1);assert.equal(mode,'tui');
 mountViewNavigation({document,location,history,currentMode:()=>mode,setMode:value=>{mode=value;live++;}});
 document.querySelector('[data-view="3d"]').click();
 assert.equal(pending,1);assert.equal(live,1);assert.equal(mode,'3d');
 dom.window.close();
});
