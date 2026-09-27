import test from 'node:test';
import assert from 'node:assert/strict';
import {inputContext,hasInputSurface} from './flygon-input-context.js';
test('revealed choice invalidates a prior plain-dialogue decision',()=>{
 const page={frame:1,status:{screen:'overworld'},map_info:{name:'VioletPokecenter1F',player:{x:3,y:3,facing:'Right'}},observe:{visible_dialogue:'Will you take the EGG?',menus:[]}};
 const choice=structuredClone(page);choice.observe.menus=[{kind:'yes_no',entries:['>YES','NO']}];
 assert.ok(hasInputSurface(page));assert.notEqual(inputContext(page),inputContext(choice));
 const next=structuredClone(page);next.frame=20;assert.equal(inputContext(page),inputContext(next));
 const cursor=structuredClone(choice);cursor.observe.menus[0].entries=['YES','>NO'];assert.notEqual(inputContext(choice),inputContext(cursor));
 const field=structuredClone(page);field.observe.visible_dialogue='';assert.equal(hasInputSurface(field),false);
});
