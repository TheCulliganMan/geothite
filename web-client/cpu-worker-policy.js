// Keep worker availability separate from activation: a 2D session installs only
// dormant callbacks, retaining the original model string until its first 3D use.
export function cpuWorkerPolicy({ params, wasm, storage, embedded, workerSupported }) {
  const preview3D = params.get('view') === '3d' || ['new-bark', 'bedroom', 'goldenrod', 'saffron', 'department-store',
    'underground', 'route26', 'route26-border', 'route26-canyon', 'route29', 'route29-border', 'route29-battle',
    'falkner-battle', 'national-park', 'national-park-pool', 'ilex', 'ilex-forest',
    'union-cave', 'ice-path', 'whirl-islands', 'mount-silver', 'world-stream',
    'cherrygrove-border'].includes(params.get('preview'));
  const enabled = (flag, exported) => workerSupported && params.get(flag) !== '0'
    && typeof wasm[exported] === 'function';
  return {
    terrain: enabled('terrainWorker', 'crystal_terrain_worker_header'),
    actor: enabled('actorWorker', 'crystal_actor_worker_header'),
    eager: !embedded && (preview3D || storage.getItem('crystal.display.voxel') === 'true'),
    preview3D,
  };
}
