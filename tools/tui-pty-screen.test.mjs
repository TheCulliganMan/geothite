import assert from 'node:assert/strict';
import {test} from 'node:test';
import {PtyScreen} from './tui-pty-screen.mjs';
test('PTY snapshots assemble differential writes and ignore split graphics payloads',()=>{
  const screen=new PtyScreen(80,24);
  screen.feed('\x1b[?1049h\x1b[2J\x1b[3;4HSA\x1b7\x1b[1;1H\x1b_Ga=T;NOT');
  screen.feed(' A MENU\x1b\\\x1b8\x1b[38;2;1;2;3mVE');
  assert(screen.text().includes('SAVE'));
  assert(!screen.text().includes('NOT A MENU'));
  screen.feed('\x1b[3;4H\x1b[K');
  assert(!screen.text().includes('SAVE'));
  screen.resize(100,32);
  screen.feed('\x1b[32;96HDONE');
  assert(screen.text().includes('DONE'));
});
