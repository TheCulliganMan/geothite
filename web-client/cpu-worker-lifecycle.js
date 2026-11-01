// Worker startup is deferred until 3D needs CPU geometry. Keep the original
// validated source string by reference, then release it when posted once.
export function createCpuWorkerLifecycle(createClient, metrics, { eager = true } = {}) {
  let client = null;
  let initialized = null;
  let state = 1;
  let disposed = false;
  let sourcesStarted = false;
  let sourceInstallStarted = false;
  let sourceJson;
  let initializationFailureRecorded = false;
  const releaseSource = () => { sourceJson = undefined; metrics.retainedSourceCharacters = 0; };
  const fail = () => { state = 0; metrics.status = 'failed'; releaseSource(); };
  const initializationFailed = () => {
    if (disposed) return;
    if (!initializationFailureRecorded) { metrics.initFailures++; initializationFailureRecorded = true; }
    fail();
  };
  function installSources() {
    if (!initialized || !sourcesStarted || sourceInstallStarted || state === 0) return;
    sourceInstallStarted = true;
    const started = performance.now();
    const hasModels = sourceJson != null;
    initialized.then(() => {
      if (disposed) return;
      const json = sourceJson;
      releaseSource();
      return json == null ? undefined : client.installScenery('models', json);
    }).then(() => {
      if (disposed || state === 0) return;
      state = 2; metrics.status = 'ready'; metrics.modelsInstalled = hasModels ? 1 : 0;
      metrics.workerSourcesMs = performance.now() - started;
    }, initializationFailed);
  }
  function activate() {
    if (client || disposed || state === 0) return;
    metrics.initializations++; metrics.status = 'initializing';
    const started = performance.now();
    try {
      client = createClient();
      initialized = client.initialize('boot');
      initialized.then(() => { if (!disposed) metrics.workerBootMs = performance.now() - started; }, initializationFailed);
      installSources();
    } catch { initializationFailed(); }
  }
  const lifecycle = {
    ready(activateRequested = false) { if (activateRequested) activate(); return state; },
    sources(json) {
      if (sourcesStarted) throw new Error('CPU worker sources were already finalized');
      sourcesStarted = true;
      if (state === 0 || disposed) return;
      sourceJson = json;
      metrics.retainedSourceCharacters = typeof json === 'string' ? json.length : 0;
      installSources();
    },
    submit(...args) {
      if (state !== 2 || !client) throw new Error('CPU worker is not ready');
      client.submit(...args);
    },
    cancel() { client?.cancel(); },
    fail,
    dispose() {
      if (disposed) return;
      disposed = true; state = 0; metrics.status = 'disposed'; releaseSource(); client?.dispose();
    },
  };
  if (eager) activate();
  return lifecycle;
}
