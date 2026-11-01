// Verify the deployed server accepts its own advertised pack over browser WebSocket.
import { webkit } from 'playwright';
import assert from 'node:assert/strict';
const [base] = process.argv.slice(2);
assert.ok(base, 'usage: node tools/browser-online-smoke.mjs BASE_URL');
const browser = await webkit.launch({ headless: true });
try {
  const page = await browser.newPage();
  await page.goto(new URL('/healthz', base).href);
  const result = await page.evaluate(async () => {
    const status = await (await fetch('/v1/status')).json();
    const { token } = await (await fetch('/v1/session', { method: 'POST' })).json();
    const claims = JSON.parse(atob(token.split('.')[0].replace(/-/g, '+').replace(/_/g, '/')));
    return new Promise((resolve, reject) => {
      const scheme = location.protocol === 'https:' ? 'wss:' : 'ws:';
      const socket = new WebSocket(`${scheme}//${location.host}/v1/ws?token=${token}`);
      const fail = message => { clearTimeout(timer); socket.close(); reject(new Error(message)); };
      const timer = setTimeout(() => fail('Online handshake timed out'), 15000);
      socket.onopen = () => socket.send(JSON.stringify({
        type: 'hello', protocol_version: status.protocolVersion,
        identity: { user_id: claims.user_id, display_name: 'Browser Audit',
          world: { world_id: 'main', modpack: status.modpacks[0] } },
      }));
      socket.onmessage = event => {
        const message = JSON.parse(event.data);
        if (message.type === 'error') return fail(message.message);
        clearTimeout(timer); socket.close();
        resolve({ type: message.type, modpack: status.modpacks[0].id });
      };
      socket.onerror = () => fail('Online socket failed');
    });
  });
  assert.equal(result.type, 'welcome');
  console.log(JSON.stringify(result));
} finally { await browser.close(); }
