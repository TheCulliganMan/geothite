// Browser transport only; byte counts come from the response stream.
export async function fetchAsset(url, { label, expectedBytes, report = () => {} } = {}) {
  let loaded = 0, total = null;
  const update = phase => report({ label, loaded, total, phase });
  update('connecting');
  try {
    const response = await fetch(url);
    if (!response.ok) throw Error(`${label}: HTTP ${response.status}`);
    const size = expectedBytes ?? response.headers.get('x-asset-bytes') ??
      (!response.headers.get('content-encoding') ? response.headers.get('content-length') : null);
    total = Number(size) > 0 ? Number(size) : null;
    const chunks = [];
    let last = 0;
    if (response.body) {
      const reader = response.body.getReader();
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        chunks.push(value);
        loaded += value.byteLength;
        if (performance.now() - last > 80) { update('downloading'); last = performance.now(); }
      }
    } else {
      const bytes = new Uint8Array(await response.arrayBuffer());
      chunks.push(bytes); loaded = bytes.byteLength;
    }
    if (total !== null && loaded !== total) throw Error(`${label}: incomplete download (${loaded} / ${total} bytes)`);
    const bytes = new Uint8Array(loaded);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    update('downloaded');
    return bytes;
  } catch (error) { report({ label, loaded, total, phase: 'error', error: String(error.message || error) }); throw error; }
}
export async function loadWasm(init, url, label, report) {
  const bytes = await fetchAsset(url, { label, report });
  report({ label, loaded: bytes.byteLength, total: bytes.byteLength, phase: 'initializing' });
  try {
    const runtime = await init({ module_or_path: bytes });
    report({ label, loaded: bytes.byteLength, total: bytes.byteLength, phase: 'ready' });
    return runtime;
  } catch (error) {
    report({ label, phase: 'error', error: String(error.message || error) });
    throw error;
  }
}
export function mountAssetProgress(container, labels) {
  const rows = new Map();
  const format = bytes => `${(bytes / 1048576).toFixed(1)} MiB`;
  const report = ({ label, loaded = 0, total = null, phase = 'waiting', error }) => {
    let row = rows.get(label);
    if (!row) {
      const element = document.createElement('div'); element.className = 'asset-progress';
      const name = document.createElement('strong'); name.textContent = label;
      const value = document.createElement('span');
      const bar = document.createElement('progress'); bar.max = 1; bar.setAttribute('aria-label', label);
      element.append(name, value, bar); container.append(element);
      row = { element, value, bar }; rows.set(label, row);
    }
    row.element.dataset.phase = phase;
    const sizes = total ? `${format(loaded)} / ${format(total)}` : format(loaded);
    row.value.textContent = phase === 'downloading' ? `${sizes}${total ? ` · ${Math.floor(loaded / total * 100)}%` : ''}` :
      phase === 'downloaded' ? `${sizes} · Downloaded` :
      phase === 'ready' ? `${format(loaded)} · Ready` :
      phase === 'initializing' ? 'Initializing…' : phase === 'error' ? error : phase === 'connecting' ? 'Connecting…' : 'Waiting';
    if ((phase === 'downloading' && total) || phase === 'downloaded' || phase === 'ready') row.bar.value = total ? loaded / total : 1;
    else row.bar.removeAttribute('value');
  };
  labels.forEach(label => report({ label }));
  return report;
}
