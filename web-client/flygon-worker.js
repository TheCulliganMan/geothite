// Transport only. Neural dynamics, encoding, decoding and pixels are Rust/WASM.
import init, { Brain } from "./flygon/crystal_flygon.js";
import { fetchAsset, loadWasm } from "./asset-progress.js";
const report = progress => self.postMessage({ kind: "asset-progress", progress });
let brain,
  runtime,
  laboratory = false,
  operantMode = false,
  lastSensory = null;
self.onmessage = async ({ data }) => {
  const { id, kind } = data;
  try {
    let result;
    if (kind === "load") {
      if (brain) {
        brain.free();
        brain = null;
      }
      lastSensory = null;
      runtime = await loadWasm(init, "./flygon/crystal_flygon_bg.wasm", "Brain engine", report);
      const loadStart = performance.now();
      laboratory = data.laboratory === true;
      operantMode = Boolean(data.config.operant);
      const base = data.base || "./flygon-data/";
      const response = await fetch("./flygon-dataset.json", { cache: "no-store" });
      if (!response.ok) throw Error("Dataset identity download failed");
      const dataset = await response.json();
      const [graph, metadata] = await Promise.all([
        fetchAsset(base + "graph.bin", { label: "Connectome", expectedBytes: dataset.graph_bytes, report }),
        fetchAsset(base + "metadata.json", { label: "Neuron metadata", expectedBytes: dataset.metadata_bytes, report }),
      ]);
      report({ label: "Connectome", phase: "initializing" });
      brain = new Brain(graph, new TextDecoder().decode(metadata), JSON.stringify(data.config));
      report({ label: "Connectome", loaded: graph.byteLength, total: graph.byteLength, phase: "ready" });
      const telemetry = JSON.parse(brain.telemetry());
      if (
        telemetry.graph_id !== dataset.graph_id ||
        telemetry.neurons !== dataset.neurons ||
        telemetry.edges !== dataset.edges
      ) {
        brain.free();
        brain = null;
        throw Error(
          "Loaded graph does not match the pinned full MaleCNS dataset",
        );
      }
      result = {
        ...telemetry,
        wasm_memory_bytes: runtime.memory.buffer.byteLength,
        load_ms: performance.now() - loadStart,
      };
    } else if (!brain) throw Error("Load the brain first");
    else if (kind === "step" && operantMode) {
      if (!data.observation)
        throw Error("Action-memory mode requires a game observation");
      const start = performance.now(),
        r = JSON.parse(brain.operant_decide(JSON.stringify(data.observation)));
      result = {
        ...r.telemetry,
        action: r.action,
        sensory: {
          encoding: "spatial-map-v2 situation/action KC cues; MBON spike readout",
          map_features: r.map_features,
          spatial_channels: r.spatial_channels,
          context: r.context,
          probe_learning: false,
        },
        operant_trial_ms: r.neural_trial_ms,
        wasm_memory_bytes: runtime.memory.buffer.byteLength,
        wall_ms: performance.now() - start,
      };
    } else if (kind === "step") {
      const sensory =
        !laboratory && data.observation
          ? JSON.parse(brain.observe(JSON.stringify(data.observation)))
          : lastSensory;
      const previous = data.freezeLearning
        ? JSON.parse(brain.configuration())
        : null;
      const start = performance.now();
      let telemetry;
      try {
        if (previous)
          brain.configure(JSON.stringify({ ...previous, learning: false }));
        brain.advance(data.duration);
        telemetry = JSON.parse(brain.summary());
      } finally {
        if (previous) brain.configure(JSON.stringify(previous));
      }
      result = {
        ...telemetry,
        wasm_memory_bytes: runtime.memory.buffer.byteLength,
        wall_ms: performance.now() - start,
        sensory,
        action: JSON.parse(brain.action()),
      };
    } else if (kind === "configure") {
      brain.configure(JSON.stringify(data.config));
      operantMode = Boolean(data.config.operant);
      brain.mark_intervention("tuning");
      result = { configured: true };
    } else if (kind === "aversive") {
      brain.aversive_pulse();
      brain.mark_intervention("manual_aversive");
      result = { stimulated: true };
    } else if (kind === "reward") {
      brain.appetitive_pulse();
      brain.mark_intervention("manual_appetitive");
      result = { stimulated: true };
    } else if (kind === "probe") {
      if (!laboratory) throw Error("Open the conditioning lab first");
      result = JSON.parse(brain.conditioning_probe(data.cue));
      lastSensory = result.sensory;
    } else if (kind === "teach") {
      if (!laboratory) throw Error("Open the conditioning lab first");
      brain.mark_intervention("lab_conditioning");
      result = JSON.parse(brain.conditioning_trial(data.cue, true));
      lastSensory = result.sensory;
    } else if (kind === "erase") {
      brain.erase_memory();
      brain.mark_intervention("erased_memory");
      result = JSON.parse(brain.telemetry());
    } else if (kind === "outcome")
      result = JSON.parse(
        operantMode
          ? brain.operant_feedback(JSON.stringify(data.observation))
          : brain.outcome(JSON.stringify(data.observation), data.button || ""),
      );
    else if (kind === "intervention") {
      brain.mark_intervention(data.reason);
      result = { recorded: true };
    } else if (kind === "connections")
      result = JSON.parse(brain.view_connections(data.index));
    else if (kind === "anatomy") {
      const records = brain.view_anatomy();
      self.postMessage({ id, result: { records } }, [records.buffer]);
      return;
    } else if (kind === "activity") {
      const indices = brain.view_activity();
      self.postMessage({ id, result: { indices } }, [indices.buffer]);
      return;
    } else if (kind === "summary") result = JSON.parse(brain.summary());
    else if (kind === "record") result = JSON.parse(brain.run_record());
    else if (kind === "checkpoint") result = { checkpoint: brain.checkpoint() };
    else if (kind === "restore") {
      brain.restore(data.checkpoint);
      operantMode = Boolean(JSON.parse(brain.configuration()).operant);
      brain.mark_intervention("restored_checkpoint");
      const record = JSON.parse(brain.run_record());
      result = {
        ...JSON.parse(brain.telemetry()),
        operant_trial_ms: operantMode
          ? record.operant.neural_trial_ms
          : undefined,
        story: operantMode ? record.operant.story?.milestones : undefined,
        config: JSON.parse(brain.configuration()),
      };
    } else if (kind === "find")
      result = JSON.parse(brain.find_cells(data.query));
    else if (kind === "pick")
      result = JSON.parse(
        brain.pick(
          data.width,
          data.height,
          data.x,
          data.y,
          data.yaw,
          data.pitch,
        ),
      );
    else if (kind === "inspect") result = JSON.parse(brain.inspect(data.index));
    else if (kind === "render") {
      const pixels = brain.render_styled(
        data.width,
        data.height,
        data.yaw,
        data.pitch,
        data.group || "all",
        JSON.stringify(data.style || {}),
      );
      self.postMessage(
        { id, result: { pixels, width: data.width, height: data.height } },
        [pixels.buffer],
      );
      return;
    } else throw Error("Unknown brain command");
    self.postMessage({ id, result });
  } catch (error) {
    self.postMessage({ id, error: String(error?.message || error) });
  }
};
