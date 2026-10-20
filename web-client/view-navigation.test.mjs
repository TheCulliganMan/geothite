import test from 'node:test';
import assert from 'node:assert/strict';
import { JSDOM } from 'jsdom';
import { mountViewNavigation } from './view-navigation.js';
const markup='<nav id="view-navigation"><a href="/?view=2d" data-view="2d">2D</a><a href="/?view=3d" data-view="3d">3D</a><a href="/?view=tui" data-view="tui">TUI</a></nav>';
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
