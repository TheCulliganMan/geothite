// UI transport guard: no neural computation or button choice belongs here.
export function inputContext(observation) {
  return JSON.stringify([observation?.status?.screen, observation?.map_info?.name,
    observation?.map_info?.player, observation?.observe?.menus || [],
    observation?.observe?.visible_dialogue || ""]);
}
export function hasInputSurface(observation) {
  return Boolean(observation?.observe?.menus?.length || observation?.observe?.visible_dialogue);
}
