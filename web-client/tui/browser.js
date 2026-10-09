import init, { BrowserTui } from './geothite.js';
import { keyButton, modalInput, registerTuiTools } from './bridge.js';
import { paintTerminal } from './ascii-frame.js';

const screen = document.getElementById('screen');
const status = document.getElementById('status');
const canvas = document.getElementById('ascii');
const controls = document.getElementById('touch-controls');
const viewToggle = document.getElementById('view-toggle');
let painted = true;
try { painted = localStorage.getItem('geothite.tui.view') !== 'text'; } catch {}
let stopPainting;
let game, observation, registration;
let phoneLayout;
let visualTimer;
let inkTimer;
let inkLast = performance.now();
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
function pauseVisual() { clearTimeout(visualTimer); visualTimer = undefined; }
function stopVisual() { pauseVisual(); game?.cancel_visual(); }
function stopInk() {clearTimeout(inkTimer);inkTimer=undefined;inkLast=performance.now();}
function animateInk() {
  if (!painted || reducedMotion.matches || document.hidden || !game.world_bounds().length) {stopInk();return;}
  // Input/resize redraws replace the cached painter, not this pending deadline.
  // Restarting the timeout on every redraw makes held movement freeze the ink.
  if (inkTimer !== undefined) return;
  inkTimer=setTimeout(()=>{
    inkTimer=undefined;
    try {
      const now=performance.now();
      game.advance_ink((now-inkLast)/1000);inkLast=now;
      stopPainting?.updateDots(game.world_dot_sizes());animateInk();
    }
    catch(error) {stopInk();report(error);}
  },Math.max(0,1000/30-(performance.now()-inkLast)));
}
function replayVisual() {
  // Finite Rust-produced frames. No RAF game loop, inputs always interrupt.
  if (!painted || reducedMotion.matches || !game.visual_active()) { stopVisual(); return; }
  visualTimer = setTimeout(() => {
    visualTimer = undefined;
    try { game.advance_visual(); draw(); replayVisual(); } catch (error) { stopVisual(); report(error); }
  }, 50);
}
reducedMotion.addEventListener('change', () => { if (game) { stopVisual(); draw(); } });
const report = error => { status.textContent = `Geothite: ${error?.message ?? error}`; status.hidden = false; };

function draw() {
  screen.dataset.animation = String(game.visual_active());
  const probe = document.createElement('span');
  probe.textContent = 'MMMMMMMMMM';
  probe.style.cssText = 'position:absolute;visibility:hidden;white-space:pre';
  screen.append(probe);
  // A narrow mouse-only desktop is not a touchscreen. Width alone must never
  // make Game Boy buttons appear there.
  const touchscreen = matchMedia('(any-pointer: coarse)').matches || navigator.maxTouchPoints > 0;
  const mobile = touchscreen && document.documentElement.clientWidth <= 960;
  if (mobile) {
    const style = getComputedStyle(document.body);
    const availableWidth = document.documentElement.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
    const availableHeight = Math.min(innerHeight, visualViewport?.height ?? innerHeight) - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom);
    const cell = Math.min(8.5, availableWidth / 40, availableHeight / 52);
    const columns = Math.max(40, Math.min(80, Math.floor(availableWidth / cell)));
    const rows = Math.max(26, Math.min(60, Math.floor(availableHeight / (cell * 2))));
    probe.remove();
    screen.innerHTML = painted ? game.render_painted(columns, rows, true) : game.render_text(columns, rows, true);
    screen.dataset.view = painted ? 'painted' : 'text';
    canvas.style.width = `${columns * cell}px`;
    phoneLayout = { columns, rows };
    stopPainting?.(); stopPainting = paintTerminal(canvas, screen, worldDetail());
    const box = canvas.getBoundingClientRect();
    controls.hidden = false;
    controls.style.cssText = `left:${box.left}px;top:${box.bottom - box.height / rows * 8}px;width:${box.width}px;height:${box.height / rows * 8}px`;
    placeToggle(box);
    observation = JSON.parse(game.observe());
    animateInk();
    return;
  }
  phoneLayout = null;
  controls.hidden = true;
  const width = Math.max(mobile ? 8 : 11, probe.getBoundingClientRect().width / 10);
  probe.remove();
  const columns = Math.max(40, Math.min(128, Math.floor(document.documentElement.clientWidth / width)));
  const rows = Math.max(26, Math.min(painted ? 60 : 28, Math.floor(window.innerHeight / (width * 2))));
  screen.innerHTML = painted ? game.render_painted(columns, rows, false) : game.render(columns, rows);
  screen.dataset.view = painted ? 'painted' : 'text';
  canvas.style.width = `${Math.min(document.documentElement.clientWidth, columns * width)}px`;
  stopPainting?.();
  stopPainting = paintTerminal(canvas, screen, worldDetail());
  placeToggle(canvas.getBoundingClientRect());
  observation = JSON.parse(game.observe());
  animateInk();
}
function worldDetail() {
  if (!painted) return;
  const html = game.world_dots();
  return html ? { html, bounds: [...game.world_bounds()], sizes:game.world_dot_sizes() } : undefined;
}
function placeToggle(box) {
  viewToggle.hidden = false;
  viewToggle.style.cssText = `left:${box.right - 90}px;top:${box.top}px`;
  viewToggle.textContent = painted ? 'TEXT · V' : 'PAINT · V';
  viewToggle.setAttribute('aria-label', painted ? 'Switch to text view' : 'Switch to painted view');
  viewToggle.setAttribute('aria-pressed', String(!painted));
}
function toggleView() {
  stopVisual();
  painted = !painted;
  try { localStorage.setItem('geothite.tui.view', painted ? 'painted' : 'text'); } catch {}
  draw();
  screen.focus({ preventScroll: true });
}
viewToggle.addEventListener('click', () => { if (game) { try { toggleView(); } catch (error) { report(error); } } });

function press(button) {
  if (!game) throw new Error('Game is still loading.');
  stopVisual();
  game.press(button);
  if (!painted || reducedMotion.matches) game.cancel_visual();
  status.hidden = true;
  draw();
  replayVisual();
  return observation;
}

let resolveReady, rejectReady;
const ready = new Promise((resolve, reject) => { resolveReady = resolve; rejectReady = reject; });
ready.catch(() => {});
window.geothiteTui = Object.freeze({
  ready,
  observe() { if (!game) throw new Error('Game is still loading.'); return JSON.parse(game.observe()); },
  press,
  search(query = '') { if (!game) throw new Error('Game is still loading.'); return JSON.parse(game.search(query)); },
  execute(code) {
    if (!game) throw new Error('Game is still loading.');
    pauseVisual();
    try { return JSON.parse(game.execute_code(code)); }
    finally {
      // Failed scripts can have applied earlier inputs: paint the actual state.
      if (!painted || reducedMotion.matches) game.cancel_visual();
      draw(); replayVisual();
    }
  },
  save() { if (!game) throw new Error('Game is still loading.'); stopVisual(); game.save(); draw(); return observation; },
});
// Register before downloading assets. Executions await this same live session.
registerTuiTools(document, window.geothiteTui).then(value => { registration = value; }).catch(report);
window.addEventListener('pagehide', () => { stopInk();stopVisual(); registration?.dispose(); stopPainting?.(); }, { once: true });
document.addEventListener('visibilitychange',()=>{if(document.hidden)stopInk();else if(game)animateInk();});

window.addEventListener('keydown', event => {
  if (event.target.closest?.('#touch-controls, #view-toggle')) return;
  if (!game || event.ctrlKey || event.metaKey || event.altKey) return;
  const button = keyButton(event.key, modalInput(observation));
  if (!button && !['F5', '?', '.', 'r', 'R', 'v', 'V'].includes(event.key)) return;
  event.preventDefault();
  try {
    if (button) press(button);
    else {
      stopVisual();
      if (event.key === 'v' || event.key === 'V') { toggleView(); return; }
      if (event.key === 'F5') game.save();
      if (event.key === '?') game.toggle_help();
      if (event.key === '.') game.wait(8);
      draw();
    }
  } catch (error) { report(error); }
});
controls.addEventListener('click', event => {
  const button = event.target.closest('button')?.dataset.button;
  if (button) { try { press(button); } catch (error) { report(error); } }
});
// Touch stays on the TUI itself: swipe to move, tap A, hold Start, two fingers B.
let touch;
screen.addEventListener('pointerdown', event => {
  if (event.pointerType !== 'touch' || !game) return;
  if (touch) { touch.cancel = true; try { press('b'); } catch (error) { report(error); } return; }
  touch = { id: event.pointerId, x: event.clientX, y: event.clientY, time: Date.now() };
  if (event.isTrusted) screen.setPointerCapture(event.pointerId);
});
screen.addEventListener('pointerup', event => {
  if (!touch || touch.id !== event.pointerId) return;
  const previous = touch; touch = null;
  if (previous.cancel) return;
  const dx = event.clientX - previous.x, dy = event.clientY - previous.y;
  const button = Math.max(Math.abs(dx), Math.abs(dy)) > 20
    ? Math.abs(dx) > Math.abs(dy) ? dx > 0 ? 'right' : 'left' : dy > 0 ? 'down' : 'up'
    : Date.now() - previous.time > 500 ? 'start' : 'a';
  try { press(button); } catch (error) { report(error); }
});
screen.addEventListener('pointercancel', () => { touch = null; });
window.addEventListener('resize', () => { if (game) { try { draw(); } catch (error) { report(error); } } });
window.visualViewport?.addEventListener('resize', () => { if (game && phoneLayout) { try { draw(); } catch (error) { report(error); } } });

try {
  const [, response] = await Promise.all([init(), fetch('/realtime-clock.browser.crystalpack')]);
  if (!response.ok) throw new Error(`Game pack download failed (${response.status})`);
  game = BrowserTui.open(new Uint8Array(await response.arrayBuffer()));
  screen.hidden = false; canvas.hidden = false; status.hidden = true;
  draw(); screen.focus(); resolveReady();
} catch (error) { rejectReady(error); report(error); }
