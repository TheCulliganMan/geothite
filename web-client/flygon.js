import { mountAssetProgress } from "./asset-progress.js";
const reportAsset = mountAssetProgress(document.getElementById("brain-loading-assets"), ["Brain engine", "Connectome", "Neuron metadata", "3D renderer"]);
import { createViewer } from "./flygon-view.js";
// Browser lifecycle and UI transport; all neural computations live in Rust.
const $ = (id) => document.getElementById(id);
// The document base is /; keep workspace jumps on the current experiment.
for (const link of document.querySelectorAll(".workspace-nav a")) {
  link.addEventListener("click", (event) => {
    event.preventDefault();
    document.querySelector(link.hash).scrollIntoView({ block: "start" });
  });
}
const query = new URLSearchParams(location.search);
const operantInitial =
  query.has("operant") ||
  !["reference", "fast", "balanced", "refractory", "lab"].some((k) =>
    query.has(k),
  );
const laboratory = new URLSearchParams(location.search).has("lab");
const balanced =
  !laboratory && new URLSearchParams(location.search).has("balanced");
const refractory =
  !laboratory && new URLSearchParams(location.search).has("refractory");
const fast =
  operantInitial ||
  balanced ||
  refractory ||
  new URLSearchParams(location.search).has("fast");
$("mode").value = operantInitial
  ? "operant"
  : balanced
    ? "balanced"
    : refractory
      ? "refractory"
      : laboratory
        ? fast
          ? "lab-fast"
          : "lab"
        : fast
          ? "fast"
          : "reference";
$("mode").onchange = () => {
  if ($("mode").value === "operant") {
    configPath = "./flygon-operant.json";
    history.replaceState(null, "", "?operant=1");
    $("load").click();
    return;
  }
  location.href =
    "/flygon" +
    {
      reference: "?reference=1",
      fast: "?fast=1",
      refractory: "?refractory=1",
      balanced: "?balanced=1",
      lab: "?lab=1",
      "lab-fast": "?lab=1&fast=1",
    }[$("mode").value];
};
let configPath = operantInitial
  ? "./flygon-operant.json"
  : balanced
    ? "./flygon-balanced.json"
    : refractory
      ? "./flygon-refractory.json"
      : `./flygon-${laboratory ? "conditioning" : "config"}${fast ? "-fast" : ""}.json`;
$("integration").textContent = fast
  ? "0.5 ms integration · approximate fast mode"
  : "0.1 ms integration · reference mode";
$("lab-controls").hidden = !laboratory;
if (laboratory) {
  $("play").disabled = true;
  $("auto-reward").checked = false;
  $("auto-reward").disabled = true;
  $("game-state").textContent = "Game controls disabled for conditioning";
}
const worker = new Worker("./flygon-worker.js", { type: "module" });
let lastObservation = null,
  sensorFailures = 0;
const transport = {
  neuralButtonSubmissions: 0,
  observationFailures: 0,
  unobservedActions: [],
  historyLimit: 64,
  maximumStaleSteps: 24,
};
function sensorFailure(e, button = null) {
  transport.observationFailures++;
  if (button !== null) {
    transport.unobservedActions.push({
      button,
      observationError: e.message,
      lastObservedFrame: lastObservation?.frame,
    });
    if (transport.unobservedActions.length > transport.historyLimit)
      transport.unobservedActions.shift();
  }
  log(
    `Sensor unavailable${button ? " after neural " + button : ""}: ${e.message}`,
  );
}
const pending = new Map();
let sequence = 0,
  config,
  viewStyle = {},
  gameClockMode = "hosted game clock",
  running = false,
  busy = false,
  loaded = false;
function command(kind, data = {}) {
  return new Promise((resolve, reject) => {
    const id = ++sequence;
    pending.set(id, { resolve, reject });
    worker.postMessage({ id, kind, ...data });
  });
}
worker.onmessage = ({ data }) => {
  if (data.kind === "asset-progress") { reportAsset(data.progress); return; }
  const p = pending.get(data.id);
  if (!p) return;
  pending.delete(data.id);
  data.error ? p.reject(Error(data.error)) : p.resolve(data.result);
};
worker.onerror = (e) => {
  running = false;
  $("phase").textContent = "WORKER ERROR";
  for (const p of pending.values()) p.reject(Error(e.message));
  pending.clear();
};
function log(text) {
  const row = document.createElement("div");
  row.textContent = `${new Date().toLocaleTimeString()} ${text}`;
  $("events").prepend(row);
  while ($("events").children.length > 30) $("events").lastChild.remove();
}
function error(e) {
  running = false;
  bridge()?.cancel();
  $("run").textContent = "Run brain";
  $("phase").textContent = "PAUSED · ERROR";
  $("status-detail").textContent = e.message;
  $("status-detail").hidden = false;
  log(e.message);
}
const neuralKeys = [
  "stimulus_mv",
  "dopamine_mv",
  "learning_rate",
  "reward_pulse_ms",
  "eligibility_ms",
];
const rewardKeys = [
  "novelty",
  "badge",
  "starter",
  "level_up",
  "collision",
  "collision_attempts",
  "story",
  "approach",
  "dialogue",
];
function syncTuning() {
  $("reward-strengths").hidden = Boolean(config.operant);
  $("reward").disabled = $("aversive").disabled = Boolean(config.operant);
  $("operant-note").hidden = !config.operant;
  for (const key of neuralKeys) {
    $(key).value = config[key];
    const out = $(key).parentElement.querySelector("output");
    if (out) out.value = config[key];
  }
  for (const key of rewardKeys)
    $("reward-" + key).value =
      config.rewards[key] ??
      { story: 1.5, approach: 0.15, dialogue: 0.4 }[key] ??
      0;
  $("integration").textContent =
    `${config.dt_ms} ms integration · ${config.reset_synaptic_current_on_spike ? "synaptic reset ON" : "retained-current model"}${config.dt_ms > 0.1 ? " · approximate fast step" : ""}${config.operant ? " · action-cue MBON BCI" : balanced ? " · calibrated BCI" : ""}`;
  $("learning").checked = config.learning;
  $("auto-reward").checked = config.rewards.enabled;
  $("step").textContent = config.operant
    ? "One neural decision"
    : `Step ${config.decision_ms} ms`;
}
function download(value, name) {
  const url = URL.createObjectURL(
    new Blob([value], { type: "application/json" }),
  );
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
function bridge() {
  return $("game").contentWindow?.__flygonGameBridge;
}
const viewer = createViewer($("brain"), {
  onProgress: reportAsset,
  command,
  inspect: (r) => {
    $("inspection").textContent = JSON.stringify(r, null, 2);
  },
  report: (message) => error(Error(message)),
});
async function render() {
  await viewer.update();
}
function showMetrics(t) {
  showCircuits(t);
  $("metrics").textContent =
    `${t.neurons.toLocaleString()} neurons · ${t.edges.toLocaleString()} connections · ${(t.operant_trial_ms ?? t.neural_ms).toFixed(0)} ${t.operant_trial_ms ? "summed trial ms" : "neural ms"} · ${t.changed_edges}/${t.plastic_edges} plastic edges changed`;
}
function showCircuits(t) {
  $("circuits").textContent =
    `Last ${Number(t.activity_window_ms || 0).toFixed(1)} neural ms · ` +
    (t.circuit_activity || [])
      .map(
        (c) =>
          `${c.population}: ${c.spikes} spikes / ${c.active} active of ${c.neurons}`,
      )
      .join(" · ") +
    ` · Scheduled current: PAM ${((t.reward_ticks || 0) * (config?.dt_ms || 0)).toFixed(0)} ms / PPL ${((t.pulse_ticks || 0) * (config?.dt_ms || 0)).toFixed(0)} ms remaining`;
}

let controlEpoch = 0;
async function step() {
  if (busy || !loaded) return;
  busy = true;
  const epoch = controlEpoch;
  const loopStart = performance.now();
  try {
    const game = bridge();
    let observation = null,
      stale = false;
    if (!laboratory && game) {
      try {
        observation = await game.execute({ kind: "observe" });
        lastObservation = observation;
        sensorFailures = 0;
      } catch (e) {
        sensorFailure(e);
        if (!lastObservation || ++sensorFailures > transport.maximumStaleSteps)
          throw e;
        observation = lastObservation;
        stale = true;
      }
    }
    if (running)
      $("phase").textContent = stale
        ? `RUNNING · STALE SENSOR ${sensorFailures}/${transport.maximumStaleSteps}`
        : "RUNNING";
    const r = await command("step", {
      duration: config.decision_ms,
      observation,
      freezeLearning: stale,
    });
    if (epoch !== controlEpoch) return;
    showMetrics(r);
    $("speed").textContent = config.operant
      ? `${r.wall_ms.toFixed(0)} ms candidate probes · ${(r.wasm_memory_bytes / 1048576).toFixed(0)} MiB neural WASM memory`
      : `${(config.decision_ms / r.wall_ms).toFixed(2)}× neural speed · ${r.wall_ms.toFixed(0)} ms compute · ${(r.wasm_memory_bytes / 1048576).toFixed(0)} MiB neural WASM memory`;
    $("action").textContent = r.action.button
      ? `Neural output: ${r.action.button.toUpperCase()}`
      : "Neural output: no-op";
    $("game-state").textContent = laboratory
      ? "Game controls disabled for conditioning"
      : observation
        ? `${observation.status.screen} · ${observation.map_info.name}${stale ? " · LAST VALID OBSERVATION" : ""}`
        : "Waiting for game bridge";
    $("readouts").textContent = JSON.stringify(
      {
        readouts: r.action.readouts,
        spike_populations: config.operant ? ["MBON01", "MBON11"] : undefined,
        sensory: r.sensory,
        observation: {
          stale,
          plasticityDuringStep: config.operant ? false : r.learning,
          frame: observation?.frame,
          consecutiveFailures: sensorFailures,
        },
        dan_spikes: r.dan_spikes,
      },
      null,
      2,
    );
    if ($("play").checked && game) {
      let outcome = null;
      try {
        if (r.action.button) transport.neuralButtonSubmissions++;
        outcome = r.action.button
          ? await game.execute({
              kind: "press",
              button: r.action.button,
              frames: r.action.frames,
            })
          : stale
            ? null
            : observation;
        if (outcome) {
          lastObservation = outcome;
          sensorFailures = 0;
          $("game-state").textContent =
            `${outcome.status.screen} · ${outcome.map_info.name}`;
        }
      } catch (e) {
        sensorFailure(e, r.action.button);
      }
      if (epoch !== controlEpoch) return;
      if (outcome && !stale) {
        const reward = await command("outcome", {
          observation: outcome,
          button: r.action.button || "",
        });
        $("story-state").textContent =
          `Story progress: ${reward.story?.milestones?.length || 0} recorded milestones${reward.events.length ? " · " + reward.events.slice(-3).join(" · ") : ""}`;
        if (reward.training)
          showMetrics({
            ...reward.training.telemetry,
            operant_trial_ms: reward.neural_trial_ms,
          });
        if (reward.goal_reached) {
          running = false;
          $("run").textContent = "Run brain";
          $("phase").textContent =
            `${reward.story?.target || "GOAL"} · ARRIVAL VERIFIED`;
          log(
            `Actual game observation confirms ${reward.story?.target || "goal"}; controller paused.`,
          );
        }
        if (reward.events.length)
          log(
            `Outcome${reward.enabled ? " → DAN pulse" : " (pulses disabled)"}: ${reward.events.join(", ")}`,
          );
      }
    }
    await render();
    $("speed").textContent = config.operant
      ? `${r.wall_ms.toFixed(0)} ms candidate probes · ${(performance.now() - loopStart).toFixed(0)} ms full loop · ${(r.wasm_memory_bytes / 1048576).toFixed(0)} MiB neural WASM memory`
      : `${(config.decision_ms / r.wall_ms).toFixed(2)}× neural compute · ${r.wall_ms.toFixed(0)} ms compute · ${(performance.now() - loopStart).toFixed(0)} ms full loop · ${(r.wasm_memory_bytes / 1048576).toFixed(0)} MiB neural WASM memory`;
  } catch (e) {
    error(e);
  } finally {
    busy = false;
    if (running) setTimeout(step, 0);
  }
}
$("load").onclick = async () => {
  try {
    $("status-detail").hidden = true;
    running = false;
    $("run").textContent = "Run brain";
    while (busy) await new Promise((r) => setTimeout(r, 25));
    loaded = false;
    lastObservation = null;
    sensorFailures = 0;
    transport.observationFailures = 0;
    transport.neuralButtonSubmissions = 0;
    transport.unobservedActions = [];
    for (const id of [
      "run",
      "step",
      "aversive",
      "reward",
      "reload",
      "save-brain",
      "restore-brain",
      "save-record",
      "find",
      "inspect",
      "probe-a",
      "probe-b",
      "teach-a",
      "teach-b",
      "erase",
    ])
      $(id).disabled = true;
    $("load").disabled = true;
    $("phase").textContent = "LOADING FULL CONNECTOME";
    config = await fetch(configPath, { cache: "no-store" }).then((r) =>
      r.json(),
    );
    const r = await command("load", { config, laboratory });
    loaded = true;
    await viewer.load();
    showMetrics(r);
    $("action").textContent = "No neural action";
    $("readouts").textContent = "No activity yet";
    $("inspection").textContent = "";
    $("story-state").textContent = "Story progress: waiting for gameplay.";
    $("lab-result").textContent =
      "Naive brain; probes freeze learning and apply no reward.";
    $("load").disabled = false;
    $("load").textContent = "Reset brain";
    for (const id of [
      "run",
      "step",
      "aversive",
      "reward",
      "reload",
      "save-brain",
      "restore-brain",
      "save-record",
      "find",
      "inspect",
      ...(laboratory
        ? ["probe-a", "probe-b", "teach-a", "teach-b", "erase"]
        : []),
    ])
      $(id).disabled = false;
    $("phase").textContent = laboratory ? "LAB · PAUSED" : "READY";
    syncTuning();
    log(
      `Loaded ${r.neurons} neurons and ${r.edges} edges in ${(r.load_ms / 1000).toFixed(1)} s · ${(r.wasm_memory_bytes / 1048576).toFixed(0)} MiB neural WASM memory (game/browser memory additional)`,
    );
    await render();
  } catch (e) {
    $("load").disabled = false;
    error(e);
  }
};
$("run").onclick = (event) => {
  if (!running) $("game").contentWindow?.__flygonActivateAudio?.(event);
  running = !running;
  $("run").textContent = running ? "Pause" : "Run brain";
  $("phase").textContent = running ? "RUNNING" : "PAUSED";
  if (running) {
    $("play").checked = !laboratory;
    lastObservation = null;
    sensorFailures = 0;
    step();
  }
};
$("step").onclick = () => step();
$("reward").onclick = () =>
  command("reward")
    .then(() => log("Manual appetitive DAN stimulation · assisted run"))
    .catch(error);
$("aversive").onclick = () =>
  command("aversive")
    .then(() => log("Manual aversive DAN stimulation · assisted run"))
    .catch(error);
async function tune() {
  if (!loaded) return;
  const next = structuredClone(config);
  for (const key of neuralKeys) next[key] = Number($(key).value);
  for (const key of rewardKeys)
    next.rewards[key] = Number($("reward-" + key).value);
  next.learning = $("learning").checked;
  next.rewards.enabled = $("auto-reward").checked;
  await command("configure", { config: next });
  config = next;
  syncTuning();
  log(
    "Tuning applied without rebuilding or resetting memory; recorded as intervention",
  );
}
for (const id of [
  ...neuralKeys,
  ...rewardKeys.map((k) => "reward-" + k),
  "learning",
  "auto-reward",
])
  $(id).onchange = () => tune().catch(error);
$("reload").onclick = async () => {
  try {
    const next = await fetch(configPath, { cache: "no-store" }).then((r) =>
      r.json(),
    );
    await command("configure", { config: next });
    config = next;
    syncTuning();
    log("Configuration reloaded; brain memory retained");
  } catch (e) {
    error(e);
  }
};
$("save-record").onclick = async () => {
  try {
    download(
      JSON.stringify(
        {
          ...(await command("record")),
          game_clock: gameClockMode,
          transport,
          decision_note:
            "Native ledger counts observed outcomes; transport lists up to 64 unobserved button submissions. A failed response does not establish whether its button executed.",
        },
        null,
        2,
      ),
      "flygon-run-record.json",
    );
    log("Run record exported, including manual interventions and tuning");
  } catch (e) {
    error(e);
  }
};

$("inspect").onclick = () =>
  viewer.select(Number($("neuron").value)).catch(error);
$("fullscreen").onclick = () => {
  const action = document.fullscreenElement
    ? document.exitFullscreen()
    : document.documentElement.requestFullscreen();
  action.catch((e) => log(e.message));
};
document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    const wasRunning = running;
    running = false;
    $("run").textContent = "Run brain";
    if (loaded) $("phase").textContent = "PAUSED · TAB HIDDEN";
    bridge()?.cancel();
    if (wasRunning) log("Controller paused because this tab was hidden.");
  }
});
window.addEventListener("pagehide", () => worker.terminate());

$("save-brain").onclick = async () => {
  try {
    running = false;
    $("phase").textContent = "PAUSED";
    while (busy) await new Promise((r) => setTimeout(r, 30));
    $("run").textContent = "Run brain";
    const r = await command("checkpoint");
    download(r.checkpoint, "flygon-brain.json");
    log(
      "Brain dynamics and memory exported. Pokémon save is managed separately.",
    );
  } catch (e) {
    error(e);
  }
};
$("restore-brain").onchange = async () => {
  try {
    running = false;
    $("phase").textContent = "PAUSED";
    while (busy) await new Promise((r) => setTimeout(r, 30));
    $("run").textContent = "Run brain";
    const file = $("restore-brain").files[0];
    if (!file) return;
    if (file.size > 64 * 1024 * 1024)
      throw Error("Brain checkpoint exceeds 64 MB");
    const restored = await command("restore", {
      checkpoint: await file.text(),
    });
    config = restored.config;
    syncTuning();
    showMetrics(restored);
    if (restored.story)
      $("story-state").textContent =
        `Story progress: ${restored.story.length} recorded milestones · Restored brain; game state unchanged.`;
    await render();
    log("Brain checkpoint restored; game state was not changed.");
  } catch (e) {
    error(e);
  }
};

async function lab(kind, cue) {
  try {
    running = false;
    $("phase").textContent = "LOCAL CONDITIONING LAB · PAUSED";
    while (busy) await new Promise((r) => setTimeout(r, 30));
    $("run").textContent = "Run brain";
    busy = true;
    const r = await command(kind, { cue });
    if (r.outputs) {
      $("lab-result").textContent =
        `Cue ${cue}: MBON spikes ${r.outputs.map((o) => o.spikes).join(" / ")} in ${r.probe_ms} ms · reward OFF · learning OFF`;
    } else
      $("lab-result").textContent =
        kind === "erase"
          ? "Learned efficacies erased"
          : `Cue ${cue} paired with actual PAM01 activity; ${r.telemetry.changed_edges} changed connections`;
    const t = r.telemetry || r;
    showCircuits(t);
    $("metrics").textContent =
      `${t.neurons.toLocaleString()} neurons · ${t.edges.toLocaleString()} connections · ${(t.operant_trial_ms ?? t.neural_ms).toFixed(0)} ${t.operant_trial_ms ? "summed trial ms" : "neural ms"} · ${t.changed_edges}/${t.plastic_edges} plastic edges changed`;
    $("readouts").textContent = JSON.stringify(
      {
        sensory: r.sensory,
        outputs: r.outputs,
        game_decoder: r.game_decoder,
        dan_emitted_total: t.dan_spikes,
      },
      null,
      2,
    );
    log($("lab-result").textContent);
    await render();
  } catch (e) {
    error(e);
  } finally {
    busy = false;
  }
}
$("probe-a").onclick = () => lab("probe", "A");
$("probe-b").onclick = () => lab("probe", "B");
$("teach-a").onclick = () => lab("teach", "A");
$("teach-b").onclick = () => lab("teach", "B");
$("erase").onclick = () => lab("erase");

$("game").addEventListener("load", () => {
  const frame = $("game").contentWindow;
  const takeover = (event) => {
    if (event.isTrusted && ($("play").checked || running)) {
      controlEpoch++;
      running = false;
      $("play").checked = false;
      $("run").textContent = "Run brain";
      bridge()?.cancel();
      $("phase").textContent = "PAUSED · HUMAN CONTROL";
      if (loaded)
        command("intervention", { reason: "human_input" }).catch(error);
      log("Human game input paused the neural controller; assisted run.");
    }
  };
  frame.addEventListener("keydown", takeover, { capture: true });
  frame.addEventListener("pointerdown", takeover, { capture: true });
});

$("find").onclick = () =>
  command("find", { query: $("cell-query").value })
    .then((r) => ($("inspection").textContent = JSON.stringify(r, null, 2)))
    .catch(error);

async function reloadView() {
  const response = await fetch("./flygon-view.json", { cache: "no-store" });
  if (!response.ok) throw Error("View configuration unavailable");
  const next = await response.json();
  for (const key of ["background", "inactive", "spike", "dopamine"])
    if (
      next[key] !== undefined &&
      (!Array.isArray(next[key]) ||
        next[key].length !== 3 ||
        next[key].some((v) => !Number.isInteger(v) || v < 0 || v > 255))
    )
      throw Error(`Invalid view color: ${key}`);
  for (const [key, max] of [
    ["spike_radius", 3],
    ["soma_radius", 2],
  ])
    if (
      next[key] !== undefined &&
      (!Number.isInteger(next[key]) || next[key] < 0 || next[key] > max)
    )
      throw Error(`Invalid point radius: ${key}`);
  viewStyle = next;
  viewer.setStyle(next);
}
$("reload-view").onclick = () =>
  reloadView()
    .then(() => log("View reloaded; game and brain state retained"))
    .catch((e) => log(e.message));
reloadView().catch((e) => log(e.message));
fetch("./flygon-dev.json", { cache: "no-store" })
  .then(async (response) => {
    if (!response.ok) return;
    const info = await response.json();
    gameClockMode = info.local_clock
      ? "local development UTC clock"
      : "configured server clock";
    $("clock-note").textContent = gameClockMode + " · multiplayer off";
    if (!info.live_view) return;
    const events = new EventSource("./flygon-dev-events");
    events.onmessage = ({ data }) => {
      const event = JSON.parse(data);
      if (event.kind === "css") {
        const link = document.querySelector("link[rel=stylesheet]");
        const url = new URL(link.href);
        url.searchParams.set("v", event.revision);
        link.href = url.href;
      } else if (event.kind === "view")
        reloadView().catch((e) => log(e.message));
    };
    window.addEventListener("pagehide", () => events.close(), { once: true });
  })
  .catch(() => {});

// Load the local connectome on entry; gameplay still starts with Run.
$("load").click();
