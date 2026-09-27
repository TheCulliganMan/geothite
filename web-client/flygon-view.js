// Pointer/DOM transport only. Geometry, projection, picking and pixels are Rust.
export function createViewer(canvas, { command, inspect, report, onProgress = () => {} }) {
  const worker = new Worker("./flygon-view-worker.js", { type: "module" });
  const pending = new Map();
  let sequence = 0,
    ready = false,
    rendering = false,
    dirty = false,
    lastFrame,
    scheduled = false,
    activityAt = -Infinity,
    lastDraw = 0;
  const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
  let camera = { yaw: 0, pitch: 0, zoom: 1 };
  function motionEnabled() {
    return $("animate-brain").checked && !reducedMotion.matches;
  }
  function schedule() {
    if (scheduled || rendering || !ready || document.hidden) return;
    scheduled = true;
    requestAnimationFrame(draw);
  }
  let yaw = 0,
    pitch = 0,
    zoom = 1,
    style = {},
    selected = -1;
  const pointers = new Map();
  let moved = false,
    pinchDistance = 0;
  function call(kind, data = {}, transfer = []) {
    return new Promise((resolve, reject) => {
      const id = ++sequence;
      pending.set(id, { resolve, reject });
      worker.postMessage({ id, kind, ...data }, transfer);
    });
  }
  worker.onmessage = ({ data }) => {
    if (data.kind === "asset-progress") { onProgress(data.progress); return; }
    const p = pending.get(data.id);
    if (!p) return;
    pending.delete(data.id);
    data.error ? p.reject(Error(data.error)) : p.resolve(data.result);
  };
  worker.onerror = (e) => {
    ready = false;
    for (const p of pending.values()) p.reject(Error(e.message));
    pending.clear();
    report(e.message);
  };
  const $ = (id) => document.getElementById(id);
  function parameters() {
    const rect = canvas.getBoundingClientRect(),
      ratio = Math.min(devicePixelRatio || 1, 1.5);
    return {
      width: Math.max(1, Math.min(1200, Math.round(rect.width * ratio))),
      height: Math.max(1, Math.min(1200, Math.round(rect.height * ratio))),
      ...camera,
      motion: motionEnabled(),
      activityAge: Number.isFinite(activityAt)
        ? Math.max(0, (performance.now() - activityAt) / 1000)
        : 10,
      group: $("population").value,
      style: { ...style, color_mode: $("brain-color").value, show_activity_edges: $("connections").checked },
    };
  }
  function changed() {
    yaw =
      ((((yaw + Math.PI) % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI)) -
      Math.PI;
    pitch = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, pitch));
    zoom = Math.max(0.4, Math.min(4, zoom));
    $("yaw").value = yaw;
    $("pitch").value = pitch;
    $("view-scale").textContent = `${zoom.toFixed(1)}×`;
    dirty = true;
    schedule();
  }
  async function draw(now) {
    scheduled = false;
    if (!ready || rendering || document.hidden) return;
    const dt = Math.min(0.1, (now - lastDraw) / 1000);
    lastDraw = now;
    const blend = motionEnabled() ? 1 - Math.exp(-18 * dt) : 1;
    const angle = Math.atan2(
      Math.sin(yaw - camera.yaw), Math.cos(yaw - camera.yaw),
    );
    camera.yaw += angle * blend;
    camera.pitch += (pitch - camera.pitch) * blend;
    camera.zoom += (zoom - camera.zoom) * blend;
    const moving = Math.abs(angle) + Math.abs(pitch - camera.pitch)
      + Math.abs(zoom - camera.zoom) > 0.001;
    if (!moving) camera = { yaw, pitch, zoom };
    if (!dirty && !moving && !(motionEnabled() && now - activityAt < 3000)) return;
    dirty = false;
    rendering = true;
    const frame = parameters();
    try {
      const r = await call("render", frame);
      if (canvas.width !== r.width) canvas.width = r.width;
      if (canvas.height !== r.height) canvas.height = r.height;
      canvas
        .getContext("2d", { alpha: false })
        .putImageData(
          new ImageData(
            new Uint8ClampedArray(r.pixels.buffer),
            r.width,
            r.height,
          ),
          0,
          0,
        );
      lastFrame = frame;
      canvas.dataset.renderMs = r.wall_ms.toFixed(1);
      canvas.dataset.yaw = frame.yaw.toFixed(3);
      canvas.dataset.pitch = frame.pitch.toFixed(3);
      canvas.dataset.frames = String(Number(canvas.dataset.frames || 0) + 1);
    } catch (e) {
      report(e.message);
    } finally {
      rendering = false;
      if (dirty || moving || (motionEnabled() && performance.now() - activityAt < 3000))
        schedule();
    }
  }
  async function select(index) {
    if (!ready) return;
    selected = index;
    if (index < 0) {
      await call("select", { index: -1, edges: [] });
      $("selection").hidden = true;
      $("connection-note").textContent = "Select a neuron to explore its strongest incoming and outgoing connections.";
    } else {
      const r = await command("connections", { index });
      if (selected !== index) return;
      await call("select", {
        index,
        edges: $("connections").checked ? r.edges : [],
      });
      $("selection").hidden = false;
      $("selected-name").textContent =
        r.cell.kind || r.cell.class || "Source neuron";
      $("selected-id").textContent =
        `${r.cell.id} · ${r.cell.transmitter || "unassigned transmitter"}`;
      $("selected-links").textContent =
        `${Math.min(r.incoming, 48)} / ${r.incoming} incoming · ${Math.min(r.outgoing, 48)} / ${r.outgoing} outgoing`;
      $("connection-note").textContent =
        `Strongest 48 per direction · ${r.omitted_missing_somas} links lack soma coordinates. Curves join somas, not axon paths; brighter links have more contacts. Green pulses mark source spikes; travel timing is illustrative.`;
      inspect(r);
    }
    changed();
  }
  canvas.addEventListener("pointerdown", (e) => {
    if (e.button !== 0 && e.pointerType === "mouse") return;
    canvas.focus({ preventScroll: true });
    canvas.setPointerCapture(e.pointerId);
    if (!pointers.size) moved = false;
    pointers.set(e.pointerId, {
      x: e.clientX,
      y: e.clientY,
      startX: e.clientX,
      startY: e.clientY,
    });
    pinchDistance = 0;
  });
  canvas.addEventListener("pointermove", (e) => {
    const old = pointers.get(e.pointerId);
    if (!old) return;
    if (Math.hypot(e.clientX - old.startX, e.clientY - old.startY) > 4)
      moved = true;
    const dx = e.clientX - old.x,
      dy = e.clientY - old.y;
    pointers.set(e.pointerId, { ...old, x: e.clientX, y: e.clientY });
    if (pointers.size === 2) {
      moved = true;
      const [a, b] = [...pointers.values()],
        distance = Math.hypot(a.x - b.x, a.y - b.y);
      if (pinchDistance > 0) zoom *= distance / pinchDistance;
      pinchDistance = distance;
    } else {
      yaw += dx * 0.008;
      pitch += dy * 0.008;
    }
    changed();
  });
  async function release(e) {
    if (!pointers.has(e.pointerId)) return;
    pointers.delete(e.pointerId);
    pinchDistance = 0;
    if (!moved && e.type === "pointerup" && ready && lastFrame) {
      const rect = canvas.getBoundingClientRect();
      try {
        const r = await call("pick", {
          ...lastFrame,
          x: ((e.clientX - rect.left) * lastFrame.width) / rect.width,
          y: ((e.clientY - rect.top) * lastFrame.height) / rect.height,
        });
        await select(r.index);
      } catch (error) {
        report(error.message);
      }
    }
  }
  canvas.addEventListener("pointerup", release);
  canvas.addEventListener("pointercancel", release);
  canvas.addEventListener(
    "wheel",
    (e) => {
      e.preventDefault();
      zoom *= Math.exp(-Math.max(-100, Math.min(100, e.deltaY)) * 0.002);
      changed();
    },
    { passive: false },
  );
  canvas.addEventListener("keydown", (e) => {
    if (
      ![
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
        "+",
        "=",
        "-",
        "0",
        "Escape",
      ].includes(e.key)
    )
      return;
    e.preventDefault();
    if (e.key === "ArrowLeft") yaw -= 0.12;
    if (e.key === "ArrowRight") yaw += 0.12;
    if (e.key === "ArrowUp") pitch -= 0.12;
    if (e.key === "ArrowDown") pitch += 0.12;
    if (["+", "="].includes(e.key)) zoom *= 1.1;
    if (e.key === "-") zoom /= 1.1;
    if (e.key === "0") {
      yaw = pitch = 0;
      zoom = 1;
    }
    if (e.key === "Escape") select(-1).catch((e) => report(e.message));
    changed();
  });
  $("reset-view").onclick = () => {
    yaw = pitch = 0;
    zoom = 1;
    changed();
  };
  $("clear-selection").onclick = () =>
    select(-1).catch((e) => report(e.message));
  $("connections").onchange = () =>
    select(selected).catch((e) => report(e.message));
  $("population").onchange = changed;
  $("brain-color").onchange = changed;
  $("animate-brain").onchange = changed;
  reducedMotion.addEventListener("change", changed);
  document.addEventListener("visibilitychange", changed);
  $("yaw").oninput = $("pitch").oninput = () => {
    yaw = Number($("yaw").value);
    pitch = Number($("pitch").value);
    changed();
  };
  new ResizeObserver(changed).observe(canvas);
  window.addEventListener("pagehide", () => worker.terminate(), { once: true });
  return {
    async load() {
      ready = false;
      const { records } = await command("anatomy");
      await call("load", { records }, [records.buffer]);
      ready = true;
      selected = -1;
      $("selection").hidden = true;
      $("view-empty").hidden = true;
      changed();
    },
    async update() {
      if (!ready) return;
      const { samples, edges } = await command("activity");
      const hasActivity = samples.length > 0;
      await call("activity", { samples, edges }, [samples.buffer]);
      activityAt = hasActivity ? performance.now() : -Infinity;
      changed();
    },
    setStyle(next) {
      style = next;
      changed();
    },
    select,
  };
}
