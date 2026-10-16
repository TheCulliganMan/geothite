/** Align the independent CPU worker clock to the measured main header callback. */
import assert from 'node:assert/strict';
export function mergeTerrainWorkerTrace(main, jobs, options = {}) {
  const { tid = 2, threadName = 'Terrain worker CPU', headerName = 'worker.callback_header',
    spanName = 'worker.cpu_job', jobArgument = 'workerJob' } = options;
  const events = [...main.traceEvents];
  const alignments = [];
  if (jobs.length) events.push({ name: 'thread_name', ph: 'M', pid: 1, tid,
    args: { name: threadName } });
  for (const job of jobs) {
    const headers = main.traceEvents.filter(event => event.name === headerName
      && event.ph === 'X' && !event.inProgress && event.args?.job === job.id);
    assert.equal(headers.length, 1, `job ${job.id} requires exactly one measured header callback`);
    for (const field of ['cpuStart', 'cpuEnd', 'traceOriginEpoch', 'headerCallbackEpoch']) {
      assert.ok(Number.isFinite(job[field]), `job ${job.id} has no ${field} clock`);
    }
    assert.ok(job.cpuStart <= job.traceOriginEpoch && job.traceOriginEpoch <= job.cpuEnd);
    assert.ok(job.cpuEnd <= job.headerCallbackEpoch, 'worker CPU completion must precede main header consumption');
    const worker = JSON.parse(job.trace);
    assert.equal(worker.droppedEvents, 0, `job ${job.id} worker trace dropped events`);
    const origin = headers[0].ts + (job.traceOriginEpoch - job.headerCallbackEpoch) * 1000;
    const start = headers[0].ts + (job.cpuStart - job.headerCallbackEpoch) * 1000;
    events.push({ name: spanName, cat: 'geothite', ph: 'X', pid: 1, tid,
      ts: start, dur: (job.cpuEnd - job.cpuStart) * 1000, args: { job: job.id, rawBytes: job.rawBytes } });
    for (const event of worker.traceEvents) {
      assert.ok(!event.inProgress, `job ${job.id} must close worker spans before publication`);
      if (event.ph !== 'X') continue;
      assert.ok(event.ts >= 0 && event.dur >= 0);
      events.push({ ...event, pid: 1, tid, ts: origin + event.ts,
        args: { ...event.args, [jobArgument]: job.id } });
    }
    alignments.push({ job: job.id, originTraceMicroseconds: origin,
      method: 'worker epoch sampled at Rust job entry paired with main JavaScript header callback',
      uncertainty: 'clock-call/trace-entry and callback-report overhead; not a GPU clock calibration' });
  }
  return { trace: { ...main, traceEvents: events }, alignments };
}

export function mergeActorWorkerTrace(main, jobs) {
  return mergeTerrainWorkerTrace(main, jobs, { tid: 3, threadName: 'Actor worker CPU',
    headerName: 'actor_worker.callback_header', spanName: 'actor_worker.cpu_job', jobArgument: 'actorWorkerJob' });
}
