const SERIAL_MAGIC = [0x94, 0xc3];
const BLE_SERVICE = '6ba1b218-15a8-461f-9fa8-5dcae273eafd';
const BLE_FROM_RADIO = '2c55e69e-4993-11ed-b878-0242ac120002';
const BLE_TO_RADIO = 'f75c76d2-129e-4dad-a1dd-7866124401e7';
const BLE_FROM_NUM = 'ed9da18c-a800-4f66-a670-aa7547e34453';

export function encodeSerialFrame(bytes) {
  if (bytes.byteLength > 0xffff) throw new Error('Meshtastic serial message is too large.');
  const frame = new Uint8Array(bytes.byteLength + 4);
  frame.set(SERIAL_MAGIC, 0);
  frame[2] = bytes.byteLength >>> 8;
  frame[3] = bytes.byteLength;
  frame.set(bytes, 4);
  return frame;
}

export function extractSerialFrames(input) {
  const frames = [];
  let offset = 0;
  while (offset + 4 <= input.byteLength) {
    if (input[offset] !== SERIAL_MAGIC[0] || input[offset + 1] !== SERIAL_MAGIC[1]) {
      offset += 1;
      continue;
    }
    const length = (input[offset + 2] << 8) | input[offset + 3];
    if (offset + 4 + length > input.byteLength) break;
    frames.push(input.slice(offset + 4, offset + 4 + length));
    offset += 4 + length;
  }
  return { frames, remainder: input.slice(offset) };
}

function connectionDialog(document, location) {
  const params = new URLSearchParams(location.search);
  const escape = value => String(value).replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;');
  const dialog = document.createElement('dialog');
  dialog.setAttribute('aria-labelledby', 'meshtastic-title');
  dialog.innerHTML = `
    <form method="dialog">
      <h2 id="meshtastic-title">Connect Meshtastic</h2>
      <p>Choose a radio already configured for your private channel. Geothite never reads or changes its PSK or radio settings.</p>
      <label>Trainer name <input name="trainer" maxlength="16" value="${escape(params.get('player_name') ?? 'PLAYER')}"></label>
      <label>Channel index <input name="channel" type="number" min="0" max="7" value="${escape(params.get('meshtastic_channel') ?? '0')}"></label>
      <p data-status role="status">A device chooser opens only after you press a connection button.</p>
      <div><button value="serial" type="submit">USB serial</button><button value="ble" type="submit">Bluetooth</button></div>
    </form>`;
  document.body.append(dialog);
  return dialog;
}

export async function openSerialMeshtastic(navigator, receive, disconnected) {
  if (!navigator.serial) throw new Error('Web Serial is unavailable. Use a Chromium-based desktop browser over HTTPS.');
  const port = await navigator.serial.requestPort();
  await port.open({ baudRate: 115200 });
  const writer = port.writable.getWriter();
  void (async () => {
    let remainder = new Uint8Array();
    try {
      const reader = port.readable.getReader();
      while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        const joined = new Uint8Array(remainder.byteLength + value.byteLength);
        joined.set(remainder);
        joined.set(value, remainder.byteLength);
        const decoded = extractSerialFrames(joined);
        remainder = decoded.remainder;
        for (const frame of decoded.frames) receive(frame);
      }
    } catch (error) {
      disconnected(error);
    }
  })();
  return { send: bytes => writer.write(encodeSerialFrame(bytes)) };
}

export async function openBluetoothMeshtastic(navigator, receive, disconnected) {
  if (!navigator.bluetooth) throw new Error('Web Bluetooth is unavailable. Use a supported Chromium browser over HTTPS.');
  const device = await navigator.bluetooth.requestDevice({ filters: [{ services: [BLE_SERVICE] }] });
  device.addEventListener('gattserverdisconnected', () => disconnected(Error('Meshtastic Bluetooth device disconnected.')));
  const server = await device.gatt.connect();
  const service = await server.getPrimaryService(BLE_SERVICE);
  const fromRadio = await service.getCharacteristic(BLE_FROM_RADIO);
  const toRadio = await service.getCharacteristic(BLE_TO_RADIO);
  const fromNum = await service.getCharacteristic(BLE_FROM_NUM);
  await fromNum.startNotifications();
  fromNum.addEventListener('characteristicvaluechanged', async () => {
    try {
      const value = await fromRadio.readValue();
      receive(new Uint8Array(value.buffer, value.byteOffset, value.byteLength));
    } catch (error) {
      disconnected(error);
    }
  });
  return { send: bytes => toRadio.writeValueWithResponse(bytes) };
}

export function prepareMeshtasticBridge({ global = globalThis, navigator, document, location }) {
  if (new URLSearchParams(location.search).get('multiplayer') !== 'meshtastic') return null;
  let resolveReady;
  let rejectReady;
  let transport = null;
  let wasm = null;
  let writes = Promise.resolve();
  global.__crystalMeshtasticReady = new Promise((resolve, reject) => {
    resolveReady = resolve;
    rejectReady = reject;
  });
  global.crystalMeshtasticSendToRadio = bytes => {
    if (!transport) return false;
    writes = writes.then(() => transport.send(new Uint8Array(bytes))).catch(error => wasm?.crystal_meshtastic_error(String(error)));
    return true;
  };
  return {
    async connect(wasmModule) {
      wasm = wasmModule;
      const dialog = connectionDialog(document, location);
      dialog.showModal();
      const choice = await new Promise(resolve => dialog.addEventListener('close', () => resolve(dialog.returnValue), { once: true }));
      const form = dialog.querySelector('form');
      const trainer = form.elements.trainer.value.trim() || 'PLAYER';
      const channel = Number(form.elements.channel.value);
      if (!Number.isInteger(channel) || channel < 0 || channel > 7) throw new Error('Meshtastic channel must be from 0 through 7.');
      const params = new URLSearchParams(location.search);
      if ((params.get('player_name') ?? 'PLAYER') !== trainer
          || (params.get('meshtastic_channel') ?? '0') !== String(channel)) {
        params.set('player_name', trainer);
        params.set('meshtastic_channel', String(channel));
        location.search = params.toString();
        return;
      }
      const receive = bytes => wasm.crystal_meshtastic_receive(bytes);
      const disconnected = error => {
        wasm.crystal_meshtastic_error(String(error?.message ?? error));
        rejectReady(error);
      };
      try {
        transport = choice === 'serial'
          ? await openSerialMeshtastic(navigator, receive, disconnected)
          : await openBluetoothMeshtastic(navigator, receive, disconnected);
        await transport.send(wasm.crystal_meshtastic_begin());
        for (let attempt = 0; attempt < 200 && !wasm.crystal_meshtastic_connected_node(); attempt += 1) {
          await new Promise(resolve => setTimeout(resolve, 100));
        }
        const node = wasm.crystal_meshtastic_connected_node();
        if (!node) throw new Error('The radio did not return its Meshtastic node number.');
        dialog.remove();
        resolveReady(node);
      } catch (error) {
        dialog.querySelector('[data-status]').textContent = String(error?.message ?? error);
        dialog.showModal();
        rejectReady(error);
        throw error;
      }
    },
  };
}
