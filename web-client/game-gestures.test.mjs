import test from 'node:test';
import assert from 'node:assert/strict';
import { JSDOM } from 'jsdom';
import { mountGameGestures } from './game-gestures.js';

test('gameplay cancels Safari pinch and double-tap gestures while menus retain browser gestures', t => {
  const dom = new JSDOM('<div id="game"><canvas></canvas><button>A</button></div><dialog open><input></dialog>');
  t.after(() => dom.window.close());
  const { document, Event } = dom.window;
  mountGameGestures({ document, surfaces: '#game' });
  for (const type of ['gesturestart', 'gesturechange', 'gestureend', 'dblclick']) {
    for (const [selector, prevented] of [['canvas', true], ['button', true], ['input', false]]) {
      const event = new Event(type, { bubbles: true, cancelable: true });
      document.querySelector(selector).dispatchEvent(event);
      assert.equal(event.defaultPrevented, prevented, `${type} on ${selector}`);
    }
  }
});

test('multi-touch across game and control surfaces cannot pinch the page; single taps remain clicks', t => {
  const dom = new JSDOM('<canvas id="game"></canvas><button id="controls">A</button><input>');
  t.after(() => dom.window.close());
  const { document, Event } = dom.window;
  const game = document.querySelector('#game'), controls = document.querySelector('#controls'), input = document.querySelector('input');
  mountGameGestures({ document, surfaces: '#game, #controls' });
  for (const type of ['touchstart', 'touchmove']) {
    for (const [targets, prevented] of [[[game], false], [[controls], false], [[game, controls], true], [[input, input], false]]) {
      const event = new Event(type, { bubbles: true, cancelable: true });
      Object.defineProperty(event, 'touches', { value: targets.map(target => ({ target })) });
      targets.at(-1).dispatchEvent(event);
      assert.equal(event.defaultPrevented, prevented, `${type} with ${targets.length} fingers`);
    }
  }
  const zoomShortcut = new dom.window.KeyboardEvent('keydown', { key: '+', ctrlKey: true, bubbles: true, cancelable: true });
  game.dispatchEvent(zoomShortcut);
  assert.equal(zoomShortcut.defaultPrevented, false, 'explicit browser zoom shortcuts remain available');
});
