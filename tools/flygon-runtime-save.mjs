// Save a paired generation from an observed runner at a normal safe boundary.
import {chromium} from 'playwright';
import {saveBoundary} from './flygon-runtime-state.mjs';
const browser=await chromium.connectOverCDP(process.env.FLYGON_CDP||'http://127.0.0.1:9346');
try {
 const page=browser.contexts()[0].pages().find(p=>p.url().includes('/flygon'));
 if(!page)throw Error('No observed Flygon session');
 const saved=await saveBoundary(page,process.env.FLYGON_EVIDENCE_DIR||'target/flygon-runtime-watch');
 if(!saved)console.log('Save deferred until dialogue, menus, and movement finish.');
}finally{await browser.close();}
