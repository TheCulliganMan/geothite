// Summarize observed gameplay, not inferred progress or reward totals.
import fs from 'node:fs/promises';
import {dirname,join,basename} from 'node:path';
const path = process.argv[2];
if (!path) throw new Error('Usage: node tools/flygon-exploration-report.mjs <runtime.json>');
const segments=basename(path)==='runtime.json'?(await fs.readdir(dirname(path))).filter(n=>/^runtime-segment-\d+\.json$/.test(n)).sort().map(n=>join(dirname(path),n)):[];
async function* samples(){for(const file of [...segments,path])for(const sample of JSON.parse(await fs.readFile(file,'utf8')))yield sample;}
let sampleCount=0;
const tiles = new Map(), transitions = [], actions = [];
let previousMap, previousTile;
const plainDialogue={decisions:0,nonDialogueButtons:0};
let directionalChoiceInputs=0;
const interactionOutcomes={completedConversations:0,captures:0,packLoopPenalties:0};
const learnedFieldMoves=new Set(),usedFieldMoves=new Set();
const recent=[], milestones=new Set();let shortLoops=0,unchanged=0;
function visit(map, player) {
  if (!map || !player) return;
  if (previousMap && previousMap !== map) transitions.push({from: previousMap, to: map});
  const key=`${map}:${player.x},${player.y}`;
  if(key===previousTile)unchanged++;
  else {if(recent.slice(-12).filter(k=>k===key).length>=3)shortLoops++;recent.push(key);if(recent.length>64)recent.shift();}
  previousTile=key;
  previousMap = map;
  if (!tiles.has(map)) tiles.set(map, new Set());
  tiles.get(map).add(`${player.x},${player.y}`);
}
for await (const sample of samples()) {
  sampleCount++;
  for (const entry of sample.trace) {
    for(const event of entry.events||[])if(/^(story:|badge:|place:|battle:victory|battle:capture|caught:|field_move_|dialogue:complete:)/.test(event))milestones.add(event);
    // Read each feedback result once; `events` also repeats reward labels.
    const rewards=(entry.event?.reward||'').split('; ').filter(Boolean);
    const aversions=(entry.event?.aversion||'').split('; ').filter(Boolean);
    interactionOutcomes.completedConversations+=rewards.filter(s=>s.startsWith('dialogue:complete:')).length;
    if(rewards.includes('battle:capture'))interactionOutcomes.captures++;
    if(aversions.includes('action:battle_pack_loop'))interactionOutcomes.packLoopPenalties++;
    for(const reward of rewards){
      if(reward.startsWith('field_move_learned:'))learnedFieldMoves.add(reward.slice('field_move_learned:'.length));
      if(reward.startsWith('field_move_used:'))usedFieldMoves.add(reward.slice('field_move_used:'.length));
    }
    if (entry.action) {
      actions.push(entry.action);
      const features=entry.sensory||[];
      const menu=features.some(f=>f.startsWith('menu:'));
      if(features.includes('screen:overworld')&&features.includes('dialogue:present')&&!menu){
        plainDialogue.decisions++;
        if(!['a','b'].includes(entry.action.button))plainDialogue.nonDialogueButtons++;
      }
      if(menu&&['up','down','left','right'].includes(entry.action.button))directionalChoiceInputs++;
    }
    if (entry.game) visit(entry.game.map, entry.game.player);
  }
  visit(sample.observation?.map_info?.name, sample.observation?.map_info?.player);
}
const exclusions = Object.fromEntries(['start', 'select'].map(button => {
  const masked = actions.filter(action => action.readouts?.find(r => r.button === button)?.probability === 0);
  return [button, {excludedDecisions: masked.length, submissionsWhileExcluded: masked.filter(a => a.button === button).length}];
}));
console.log(JSON.stringify({
  samples: sampleCount, recordedDecisions: actions.length,
  observedTiles: Object.fromEntries([...tiles].map(([map, positions]) => [map, positions.size])),
  observedTransitions: transitions, exclusions, plainDialogue, directionalChoiceInputs,
  reachedRoute29: tiles.has('Route29'), reachedCherrygrove:tiles.has('CherrygroveCity'), reachedMrPokemonHouse:tiles.has('MrPokemonsHouse'),
  observedShortLoops:shortLoops,unchangedObservations:unchanged,milestones:[...milestones],
  interactionOutcomes, learnedFieldMoves:[...learnedFieldMoves], usedFieldMoves:[...usedFieldMoves],
  lastMap: previousMap,
  interpretation: 'Observed coverage is a lower bound. This records navigation, not causal proof of learning or repeatable story completion.'
}, null, 2));
