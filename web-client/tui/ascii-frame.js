import { mount } from './ascii-mount.js';

// Rust's Ratatui buffer is the piece. ascii.rest owns glyph rasterization,
// HiDPI sizing and its canvas atlas; it never advances the game clock.
export function paintTerminal(canvas, terminal, detail, cellRatio = 2, dotSizes) {
  const ground = '#030312';
  const palette = [];
  const colors = [];
  const blocks = [];
  let text = '';
  let x = 0, y = 0;
  const append = (character, foreground, background) => {
    if (character === '\n') { text += '\n'; x = 0; y++; return; }
    const half = character === '▀';
    const dot = '·•●'.includes(character);
    const opaque = background && background !== 'inherit' && background !== 'rgb(3, 3, 18)';
    if (half || opaque || dot) blocks.push({ x, y, index:colors.length, character, foreground, background: background || ground });
    // Raster cells are composed below, not entered in ascii.rest's 8-bit ink
    // palette. This preserves arbitrary RGB foreground/background pairs.
    // Opaque RGB cells are painted directly below as well. Do not put their
    // arbitrary colors into the library's 8-bit palette (which would wrap).
    const ink = half || opaque || dot ? ground : foreground;
    let index = palette.indexOf(ink);
    let overflow = false;
    if (index < 0) {
      if (palette.length >= 254) {
        // Area-averaged battle portraits can have many real RGB colors. Keep
        // those colors, rather than truncating indices or retinting species.
        overflow = true;
        blocks.push({x,y,character,foreground,background:background || ground});
        index = palette.indexOf(ground);
        if (index < 0) { index = palette.length; palette.push(ground); }
      } else { index = palette.length; palette.push(ink); }
    }
    colors.push(index);
    text += half || opaque || dot || overflow ? ' ' : character;
    x++;
  };
  for (const node of terminal.childNodes) {
    if (node.nodeType === 3) { for (const character of node.textContent) append(character, '#e7ebff', ground); continue; }
    const foreground = node.style.color || '#e7ebff';
    const background = node.style.backgroundColor;
    for (const character of node.textContent) append(character, foreground, background);
  }
  // Do not trim row widths: the library indexes each palette cell by column.
  const cols = text.indexOf('\n');
  const rows = text.split('\n').length - 1;
  const picture = text.slice(0, -1);
  const stop = mount(canvas, {
    meta: { name: 'geothite', category: 'ui', cols, rows, fps: 0, palette, ground, cell: cellRatio },
    default: () => (_, env) => { env.color?.set(colors); return picture; },
  });
  let fineCanvas, stopFine;
  if (detail) {
    // Rust supplies the finer scene and its exact terminal bounds. This only
    // composites two fps-zero pieces; it doesn't construct scenery or state.
    const fine = document.createElement('pre'); fine.innerHTML = detail.html;
    fineCanvas = document.createElement('canvas'); fineCanvas.setAttribute('aria-hidden','true');
    fineCanvas.style.cssText = `position:fixed;left:-10000px;top:0;visibility:hidden;width:${canvas.clientWidth * detail.bounds[2] / cols}px`;
    document.body.append(fineCanvas);
    stopFine = paintTerminal(fineCanvas, fine, null, 1, detail.sizes);
  }
  const compose = () => {
    const ctx = canvas.getContext('2d');
    const w = canvas.width / cols, h = canvas.height / rows;
    if (dotSizes) {ctx.fillStyle=ground;ctx.fillRect(0,0,canvas.width,canvas.height);}
    const drawCells = cells => { for (const cell of cells) {
      const x0 = Math.round(cell.x * w), x1 = Math.round((cell.x + 1) * w);
      const y0 = Math.round(cell.y * h), y1 = Math.round((cell.y + 1) * h);
      if (!dotSizes) {ctx.fillStyle = cell.background;ctx.fillRect(x0, y0, x1 - x0, y1 - y0);}
      ctx.fillStyle = cell.foreground;
      if (cell.character === '▀') {
        ctx.fillRect(x0, y0, x1 - x0, Math.round((y0 + y1) / 2) - y0);
      } else if ('·•●'.includes(cell.character)) {
        const coverage = {'·':0.065,'•':0.28,'●':0.78}[cell.character];
        const px = (x0+x1)/2;
        const py = (y0+y1)/2;
        const diameter = dotSizes ? dotSizes[cell.index]/255 : 2*Math.sqrt(coverage/Math.PI);
        const radius = diameter*0.5*Math.min(w,h);
        if (radius>0) {ctx.beginPath();ctx.arc(px,py,radius,0,Math.PI*2);ctx.fill();}
      } else if (cell.character !== ' ') {
        ctx.save(); ctx.beginPath(); ctx.rect(x0, y0, x1 - x0, y1 - y0); ctx.clip();
        ctx.font = `${w / 0.6}px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace`;
        ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
        ctx.fillText(cell.character, (x0 + x1) / 2, (y0 + y1) / 2);
        ctx.restore();
      }
    }};
    drawCells(blocks);
    if (fineCanvas) {
      const [x,y,width,height] = detail.bounds;
      ctx.drawImage(fineCanvas, Math.round(x*w),Math.round(y*h),Math.round(width*w),Math.round(height*h));
    }
  };
  compose();
  // Registered after the painter's observer so its resized frame is composed
  // first. No animation loop and no gameplay advancement, including reduced motion.
  const resize = new ResizeObserver(compose);
  resize.observe(canvas);
  const dispose = () => { resize.disconnect(); stop(); stopFine?.(); fineCanvas?.remove(); };
  dispose.updateDots = sizes => {if(stopFine) {stopFine.updateDots(sizes);} else {dotSizes=sizes;} compose();};
  return dispose;
}
