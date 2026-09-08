import { loadKeyBindings, keyLabel } from './player-customization.js?v=5';

const aliases = { s: 'say', say: 'say', '1': 'general', general: 'general', '2': 'trade', trade: 'trade', '3': 'lfg', lfg: 'lfg' };
export const channelLabels = { say: 'Say', general: '1. General', trade: '2. Trade', lfg: '3. Looking for Group', whisper: 'Whisper' };

export function channelName(value) {
  const name = value.toLowerCase();
  if (aliases[name]) return aliases[name];
  const custom = name.replace(/^custom:/, '');
  if (!/^[a-z0-9-]{1,24}$/.test(custom)) throw new Error('Channel names use 1–24 letters, digits or hyphens.');
  return `custom:${custom}`;
}

export function parseChat(input, selected = 'say', replyTarget = null) {
  let text = input.trim();
  let channel = selected;
  let target = null;
  if (text.startsWith('/')) {
    const match = /^\/(\S+)(?:\s+([\s\S]*))?$/.exec(text);
    if (!match) throw new Error('Enter a chat command or message.');
    const command = match[1].toLowerCase();
    text = (match[2] ?? '').trim();
    if (command === 'cancel') return { type: 'interaction_cancel' };
    if (command === 'battle' || command === 'tradewith') {
      if (!text || /\s/.test(text)) throw new Error(`Use /${command} trainer-ID, or click a trainer’s name.`);
      return { type: 'interaction_request', target_user_id: text, kind: command === 'battle' ? 'battle' : 'trade' };
    }
    if (command === 'help') return { help: true };
    if (command === 'join' || command === 'leave') {
      if (!text) throw new Error(`Use /${command} channel-name.`);
      return { type: command === 'join' ? 'chat_join' : 'chat_leave', channel: channelName(text) };
    }
    if (command === 'w' || command === 'whisper') {
      const whisper = /^(\S+)\s+([\s\S]+)$/.exec(text);
      if (!whisper) throw new Error('Use /w player-ID message. Click a trainer’s name to whisper.');
      target = whisper[1];
      text = whisper[2].trim();
      channel = 'whisper';
    } else if (command === 'r' || command === 'reply') {
      if (!replyTarget) throw new Error('No whisper to reply to yet.');
      target = replyTarget;
      channel = 'whisper';
    } else if (aliases[command]) {
      channel = aliases[command];
      if (!text) return { select: channel };
    } else {
      throw new Error('Unknown command. Use /help for chat commands.');
    }
  }
  if (!text || [...text].length > 280 || /[\u0000-\u001f\u007f-\u009f]/.test(text)) {
    throw new Error('Messages must contain 1–280 characters without control characters.');
  }
  return { type: 'chat', channel, target_user_id: target, text };
}

export function mountSpeechBubbles({ document, window, canvas, selfUserId }) {
  const layer = document.createElement('div');
  layer.id = 'speech-bubbles'; layer.setAttribute('aria-hidden', 'true');
  document.body.append(layer);
  const bubbles = new Map();
  return {
    update(state) {
      const now = window.Date.now();
      for (const event of state.events ?? []) {
        if (event.type !== 'chat' || event.channel !== 'say' || !event.text) continue;
        const id = event.from_user_id === selfUserId ? '__self__' : event.from_user_id;
        let bubble = bubbles.get(id);
        if (!bubble) {
          if (bubbles.size >= 24) { const oldest = bubbles.keys().next().value; bubbles.get(oldest).element.remove(); bubbles.delete(oldest); }
          const element = document.createElement('div'); element.className = 'speech-bubble';
          layer.append(element); bubble = { element }; bubbles.set(id, bubble);
        }
        bubble.element.textContent = event.text;
        bubble.expires = now + Math.min(10000, Math.max(5000, event.text.length * 45));
      }
      const rect = canvas.getBoundingClientRect();
      const heads = new Map((state.heads ?? []).map(head => [head.user_id, head]));
      for (const [id, bubble] of bubbles) {
        if (now >= bubble.expires) { bubble.element.remove(); bubbles.delete(id); continue; }
        const head = heads.get(id);
        bubble.element.hidden = !head || document.hidden || !!document.querySelector('dialog[open]');
        if (!head) continue;
        bubble.element.style.left = `${rect.left + head.x * rect.width}px`;
        bubble.element.style.top = `${rect.top + head.y * rect.height - 8}px`;
        bubble.element.classList.toggle('fading', bubble.expires - now < 700);
      }
    },
    clear() { for (const bubble of bubbles.values()) bubble.element.remove(); bubbles.clear(); },
    destroy() { layer.remove(); bubbles.clear(); },
  };
}

export function mountSocialChat(wasm, { document, window, playerId }) {
  const panel = document.createElement('section');
  panel.id = 'social-chat';
  panel.setAttribute('aria-label', 'Community');
  panel.innerHTML = `<button class="chat-toggle" type="button" aria-label="Open chat" title="Chat (Enter)" aria-expanded="false"><svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 11.5a7.5 7.5 0 0 1-7.5 7.5H5l-3 3V11.5A7.5 7.5 0 0 1 9.5 4h3a7.5 7.5 0 0 1 7.5 7.5Z"/></svg></button>
    <header class="chat-header" hidden><div><strong>Community</strong><span class="chat-status" role="status">Connecting…</span></div><button class="chat-nearby" type="button" aria-expanded="false">Nearby · 0</button><button class="chat-close" type="button" aria-label="Close chat" title="Return to game (Esc)" hidden>Close ×</button></header>
    <nav class="community-tabs" role="tablist" aria-label="Community" hidden><button type="button" role="tab" data-tab="chat" aria-selected="true" aria-controls="community-chat">Chat</button><button type="button" role="tab" data-tab="social" aria-selected="false" aria-controls="community-social" tabindex="-1">Social</button><button type="button" role="tab" data-tab="leaderboard" aria-selected="false" aria-controls="community-leaderboard" tabindex="-1">Leaderboard</button></nav>
    <section id="community-social" role="tabpanel" aria-label="All users" hidden><div class="community-toolbar"><label class="sr-only" for="social-search">Search all users</label><input id="social-search" type="search" placeholder="Find a trainer…" maxlength="128"><span class="directory-count" aria-live="polite">All users</span></div><div class="directory-list"></div><footer class="directory-pages"><button type="button" data-page="previous">‹ Previous</button><span></span><button type="button" data-page="next">Next ›</button></footer></section>
    <section id="community-leaderboard" role="tabpanel" aria-label="Leaderboard" hidden><div class="community-toolbar"><label class="sr-only" for="leaderboard-metric">Rank by</label><select id="leaderboard-metric"><option value="pvp_wins">PvP · Battles won</option><option value="pvp_battles">PvP · Battles played</option><option value="pve_wins">PvE · Battles won</option><option value="pve_battles">PvE · Battles played</option><option value="trades">Completed player trades</option><option value="party_level">Highest combined party level</option></select></div><div class="leaderboard-list"></div><footer class="leaderboard-pages"><button type="button" data-page="previous">‹ Previous</button><span></span><button type="button" data-page="next">Next ›</button></footer><p class="leaderboard-note">Completed PvP and trades confirmed by both players. PvE totals come from game saves. Tracking starts with this update.</p></section>
    <div id="community-chat" class="chat-log" role="log" aria-label="Chat messages" aria-live="polite" aria-relevant="additions"></div>
    <div class="chat-requests"></div><div class="chat-actions" hidden></div><div class="chat-players" hidden></div>
    <form hidden><label class="sr-only" for="chat-channel">Channel</label><select id="chat-channel"></select><label class="sr-only" for="chat-message">Message</label><input id="chat-message" autocomplete="off" placeholder="Say something…" enterkeyhint="send" maxlength="600"><button type="submit" aria-label="Send message">Send</button></form>`;
  document.body.append(panel);
  const log = panel.querySelector('.chat-log');
  const input = panel.querySelector('#chat-message');
  const select = panel.querySelector('#chat-channel');
  const form = panel.querySelector('form');
  const toggle = panel.querySelector('.chat-toggle');
  const close = panel.querySelector('.chat-close');
  const header = panel.querySelector('.chat-header');
  const nearby = panel.querySelector('.chat-nearby');
  let rosterOpen = false;
  let activeTab = 'chat';
  const tabs = panel.querySelector('.community-tabs');
  const socialPanel = panel.querySelector('#community-social');
  const boardPanel = panel.querySelector('#community-leaderboard');
  const search = panel.querySelector('#social-search');
  const metric = panel.querySelector('#leaderboard-metric');
  let directory = [], directoryTotal = 0, directoryOffset = 0, directoryQuery = '';
  let rankings = [], rankingTotal = 0, rankingOffset = 0;
  let lastCommunityRequest = 0, searchTimer = null;
  let directoryLoaded = false, rankingsLoaded = false;
  const status = panel.querySelector('.chat-status');
  const actions = panel.querySelector('.chat-actions');
  const roster = panel.querySelector('.chat-players');
  const requests = panel.querySelector('.chat-requests');
  const controller = new window.AbortController();
  const listen = (target, event, handler, options = {}) => target.addEventListener(event, handler, { ...options, signal: controller.signal });
  const updateViewport = () => {
    const viewport = window.visualViewport;
    const height = viewport?.height ?? window.innerHeight;
    const top = viewport?.offsetTop ?? 0;
    panel.classList.toggle('compact', height < 300);
    panel.style.setProperty('--chat-viewport-height', height + 'px');
    panel.style.setProperty('--chat-viewport-bottom', Math.max(0, window.innerHeight - height - top) + 'px');
  };
  listen(window, 'resize', updateViewport);
  if (window.visualViewport) {
    listen(window.visualViewport, 'resize', updateViewport);
    listen(window.visualViewport, 'scroll', updateViewport);
  }
  updateViewport();
  let channels = ['general', 'trade', 'lfg'];
  let players = [];
  let selectedPlayer = null;
  let replyTarget = null;
  let connected = false;
  let open = false;
  let rosterSignature = '';
  const requestCards = new Map();
  const swallowed = new Set();
  const selfUserId = `player-${playerId}`;
  const speech = mountSpeechBubbles({ document, window, canvas: document.querySelector('canvas'), selfUserId });
  listen(window, 'pagehide', () => speech.clear());
  const help = '/s say · /1 general · /2 trade · /3 LFG · /w trainer-ID message · /r reply · /battle trainer-ID · /tradewith trainer-ID · /cancel · /join name · /leave name';
  const updateChannels = () => {
    const previous = select.value;
    select.replaceChildren();
    for (const name of ['say', ...channels]) {
      const option = document.createElement('option');
      option.value = name;
      option.textContent = channelLabels[name] ?? name.replace(/^custom:/, '');
      select.append(option);
    }
    select.value = ['say', ...channels].includes(previous) ? previous : 'say';
  };
  updateChannels();
  const setOpen = value => {
    open = value;
    panel.classList.toggle('editing', open);
    document.body.classList.toggle('chat-open', open);
    updateViewport();
    form.hidden = !open || activeTab !== 'chat';
    tabs.hidden = !open;
    socialPanel.hidden = !open || activeTab !== 'social';
    boardPanel.hidden = !open || activeTab !== 'leaderboard';
    log.hidden = open && activeTab !== 'chat';
    nearby.hidden = activeTab !== 'chat';
    close.hidden = !open;
    header.hidden = !open;
    for (const control of log.querySelectorAll('button')) control.tabIndex = open ? 0 : -1;
    if (!open) log.scrollTop = log.scrollHeight;
    roster.hidden = !open || activeTab !== 'chat' || !rosterOpen;
    toggle.setAttribute('aria-expanded', String(open));
    toggle.setAttribute('aria-label', open ? 'Close chat' : 'Open chat');
    if (open) { (activeTab === 'social' ? search : activeTab === 'leaderboard' ? metric : input).focus(); log.scrollTop = log.scrollHeight; requestCommunity(true); }
    else { actions.hidden = true; document.querySelector('canvas')?.focus(); wasm.crystal_social_focus(false); }
  };
  const button = (label, action, handler) => {
    const el = document.createElement('button');
    el.type = 'button'; el.textContent = label; el.dataset.action = action;
    el.onclick = handler;
    return el;
  };
  const requestCommunity = (force = false) => {
    if (!open || activeTab === 'chat' || !connected) return;
    const now = window.Date.now();
    if (!force && now - lastCommunityRequest < 5000) return;
    lastCommunityRequest = now;
    attempt(() => send(activeTab === 'social'
      ? { type: 'social_list', query: directoryQuery, offset: directoryOffset }
      : { type: 'leaderboard', metric: metric.value, offset: rankingOffset }));
  };
  const setTab = (name, focus = true) => {
    activeTab = name;
    for (const tab of tabs.children) {
      tab.setAttribute('aria-selected', String(tab.dataset.tab === name));
      tab.tabIndex = tab.dataset.tab === name ? 0 : -1;
    }
    actions.hidden = true;
    setOpen(true);
    if (focus) panel.querySelector(`[data-tab="${name}"]`).focus();
  };
  const renderCommunity = () => {
    const renderRows = (container, rows, loaded, board = false) => {
      const focusedId = container.contains(document.activeElement) ? document.activeElement.dataset.userId : null;
      const scrollTop = container.scrollTop;
      container.replaceChildren();
      if (!loaded || !rows.length) {
        const empty = document.createElement('p'); empty.className = 'community-empty';
        empty.textContent = !connected ? 'Reconnecting… Status will update when connected.' : !loaded ? 'Loading…' : board ? 'No scores yet. Be the first!' : directoryQuery ? 'No trainers match your search.' : 'No trainers yet.';
        container.append(empty); return;
      }
      for (const person of rows) {
        const row = button('', 'community-player', () => showPlayer(person.user_id, person.display_name));
        row.className = 'community-player'; row.dataset.userId = person.user_id;
        row.disabled = person.user_id === selfUserId;
        if (board) { const rank = document.createElement('span'); rank.className = 'player-rank'; rank.textContent = '#' + person.rank; row.append(rank); }
        const dot = document.createElement('i'); dot.className = 'presence-dot'; dot.dataset.online = connected ? String(person.online) : 'unknown'; dot.setAttribute('aria-hidden', 'true');
        const name = document.createElement('span'); name.className = 'player-name'; name.textContent = person.display_name + (person.user_id === selfUserId ? ' (You)' : '');
        const presence = document.createElement('span'); presence.className = 'player-presence'; presence.textContent = !connected ? 'Unknown' : person.online ? 'Online' : 'Offline';
        row.append(dot, name, presence);
        if (board) { const score = document.createElement('strong'); score.className = 'player-score'; score.textContent = person.value.toLocaleString(); row.append(score); }
        container.append(row);
        if (person.user_id === focusedId) row.focus({ preventScroll: true });
      }
      container.scrollTop = scrollTop;
    };
    renderRows(panel.querySelector('.directory-list'), directory, directoryLoaded);
    renderRows(panel.querySelector('.leaderboard-list'), rankings, rankingsLoaded, true);
    if (selectedPlayer) {
      const selected = directory.find(p => p.user_id === selectedPlayer) ?? rankings.find(p => p.user_id === selectedPlayer);
      const online = connected && selected?.online !== false;
      for (const control of actions.querySelectorAll('button')) {
        control.disabled = !online || control.dataset.action !== 'whisper' && !players.some(p => p.user_id === selectedPlayer);
      }
    }
    panel.querySelector('.directory-count').textContent = `${directoryTotal} ${directoryQuery ? 'found' : 'users'}`;
    for (const [selector, offset, total] of [['.directory-pages', directoryOffset, directoryTotal], ['.leaderboard-pages', rankingOffset, rankingTotal]]) {
      const footer = panel.querySelector(selector);
      footer.hidden = total <= 100;
      footer.querySelector('span').textContent = `${offset + 1}–${Math.min(offset + 100, total)} of ${total}`;
      footer.querySelector('[data-page="previous"]').disabled = !connected || offset === 0;
      footer.querySelector('[data-page="next"]').disabled = !connected || offset + 100 >= total;
    }
  };
  const append = (text, channel = 'system', user = null, name = null) => {
    const atBottom = log.scrollHeight - log.scrollTop - log.clientHeight < 32;
    const line = document.createElement('div');
    line.className = `chat-line chat-${Object.hasOwn(channelLabels, channel) ? channel : 'system'}`;
    line.dataset.receivedAt = String(window.Date.now());
    if (user) {
      const sender = button(name, 'player', () => showPlayer(user, name));
      sender.tabIndex = open ? 0 : -1;
      line.append('[', sender, channel === 'say' ? '] says: ' : channel === 'whisper' ? '] whispers: ' : ']: ');
    }
    line.append(text);
    log.append(line);
    while (log.childElementCount > 200) log.firstElementChild.remove();
    if (!open || atBottom) log.scrollTop = log.scrollHeight;
  };
  const send = message => {
    if (!connected) throw new Error('Reconnecting… Your draft is kept.');
    wasm.crystal_social_send(JSON.stringify(message));
  };
  const attempt = fn => { try { fn(); } catch (error) { append(String(error.message ?? error)); } };
  const showPlayer = (id, name) => {
    selectedPlayer = id;
    if (!open) setOpen(true);
    actions.replaceChildren();
    const title = document.createElement('span'); title.textContent = name;
    const person = directory.find(p => p.user_id === id) ?? rankings.find(p => p.user_id === id);
    const isNearby = players.some(p => p.user_id === id);
    const online = connected && (isNearby || person?.online === true || !person);
    const whisper = button('Whisper', 'whisper', () => { setTab('chat', false); input.value = `/w ${id} `; input.focus(); });
    whisper.disabled = !online; actions.append(title, whisper);
    for (const kind of isNearby && online ? ['battle', 'trade'] : []) actions.append(button(kind === 'battle' ? 'Battle' : 'Trade', kind, () => attempt(() => {
      send({ type: 'interaction_request', target_user_id: id, kind });
      append(`${kind === 'battle' ? 'Battle' : 'Trade'} request sent to ${name}.`);
      actions.replaceChildren(button('Cancel request', 'cancel', () => attempt(() => { send({ type: 'interaction_cancel' }); actions.hidden = true; })));
      input.focus();
    })));
    if (!online) { const note = document.createElement('span'); note.textContent = 'Offline'; actions.append(note); }
    actions.hidden = false;
  };
  const submit = () => attempt(() => {
    if (!input.value.trim()) return;
    const message = parseChat(input.value, select.value, replyTarget);
    if (message.help) append(help);
    else if (message.select) {
      if (!['say', ...channels].includes(message.select)) throw new Error('Join that channel first.');
      select.value = message.select;
    } else {
      send(message);
      if (message.type === 'interaction_request') append(`${message.kind === 'battle' ? 'Battle' : 'Trade'} request sent.`);
    }
    input.value = '';
  });
  for (const tab of tabs.children) listen(tab, 'click', () => setTab(tab.dataset.tab));
  listen(search, 'input', () => {
    window.clearTimeout(searchTimer);
    searchTimer = window.setTimeout(() => { directoryQuery = search.value.trim(); directoryOffset = 0; directoryLoaded = false; renderCommunity(); requestCommunity(true); }, 250);
  });
  listen(metric, 'change', () => { rankingOffset = 0; rankingsLoaded = false; renderCommunity(); requestCommunity(true); });
  for (const [selector, kind] of [['.directory-pages', 'social'], ['.leaderboard-pages', 'leaderboard']]) {
    for (const control of panel.querySelectorAll(`${selector} button`)) listen(control, 'click', () => {
      const delta = control.dataset.page === 'next' ? 100 : -100;
      if (kind === 'social') directoryOffset = Math.max(0, directoryOffset + delta);
      else rankingOffset = Math.max(0, rankingOffset + delta);
      requestCommunity(true);
    });
  }
  listen(toggle, 'click', () => setOpen(!open));
  listen(close, 'click', () => setOpen(false));
  listen(nearby, 'click', () => {
    rosterOpen = !rosterOpen;
    roster.hidden = !rosterOpen;
    nearby.setAttribute('aria-expanded', String(rosterOpen));
  });
  listen(form, 'submit', event => { event.preventDefault(); submit(); });
  const captureKeys = event => {
    if (event.crystalGameControl) return;
    if (event.type === 'keydown' && event.repeat && swallowed.has(event.code || event.key)) {
      event.preventDefault(); event.stopImmediatePropagation(); return;
    }
    if (event.type === 'keyup' && swallowed.delete(event.code || event.key)) {
      event.preventDefault(); event.stopImmediatePropagation(); return;
    }
    if (panel.contains(document.activeElement)) {
      event.stopImmediatePropagation();
      if (event.type === 'keydown') {
        swallowed.add(event.code || event.key);
        if (document.activeElement?.dataset.tab && ['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) {
          event.preventDefault();
          const names = ['chat', 'social', 'leaderboard'];
          const index = event.key === 'Home' ? 0 : event.key === 'End' ? 2 : (names.indexOf(activeTab) + (event.key === 'ArrowRight' ? 1 : 2)) % 3;
          setTab(names[index]);
        } else if (event.key === 'Escape') { event.preventDefault(); setOpen(false); }
        else if (event.key === 'Enter' && document.activeElement === input && !event.isComposing) {
          event.preventDefault(); if (!event.repeat) submit();
        }
      }
      return;
    }
    if (event.type !== 'keydown' || event.ctrlKey || event.metaKey || event.altKey || event.repeat ||
        document.querySelector('dialog[open]') || document.activeElement?.isContentEditable ||
        /^(INPUT|TEXTAREA|SELECT|BUTTON)$/.test(document.activeElement?.tagName)) return;
    const bindings = loadKeyBindings(window);
    const code = event.code || (event.key.length === 1 ? `Key${event.key.toUpperCase()}` : event.key);
    if (code === bindings.chat || event.key === '/') {
      event.preventDefault(); event.stopImmediatePropagation(); swallowed.add(event.code || event.key);
      activeTab = 'chat'; setTab('chat', false); if (event.key === '/') input.value = '/';
    } else if (code === bindings.start || code === bindings.select) {
      event.preventDefault(); event.stopImmediatePropagation(); swallowed.add(event.code);
      const canvas = document.querySelector('canvas');
      const isSelect = code === bindings.select;
      for (const type of ['keydown', 'keyup']) {
        const start = new window.KeyboardEvent(type, { key: isSelect ? 'Shift' : 'Enter', code: isSelect ? 'ShiftRight' : 'Enter', bubbles: true });
        Object.defineProperty(start, 'crystalGameControl', { value: true });
        // Give the game a frame to observe Start before releasing it.
        if (type === 'keydown') canvas?.dispatchEvent(start);
        else window.setTimeout(() => canvas?.dispatchEvent(start), 60);
      }
    } else if (event.key === 'Enter' || code === 'ShiftRight') {
      // Only configured physical bindings may forward the engine's internal
      // Start and Select keys. Touch/controller events bypass this physical-key path.
      event.preventDefault(); event.stopImmediatePropagation(); swallowed.add(event.code || event.key);
    }
  };
  const updateBindingLabels = () => {
    const bindings = loadKeyBindings(window);
    panel.querySelector('.chat-toggle').title = `Chat (${keyLabel(bindings.chat)})`;
    for (const element of document.querySelectorAll('[data-chat-key]')) element.textContent = keyLabel(bindings.chat);
    for (const element of document.querySelectorAll('[data-start-key]')) element.textContent = keyLabel(bindings.start);
    for (const element of document.querySelectorAll('[data-select-key]')) element.textContent = keyLabel(bindings.select);
    document.querySelector('canvas')?.setAttribute('aria-label', `Game screen. Arrow keys or WASD move; Z confirms; X goes back; ${keyLabel(bindings.start)} opens the game menu; ${keyLabel(bindings.chat)} opens chat; ${keyLabel(bindings.select)} is Select.`);
  };
  listen(window, 'geothite-key-bindings-changed', updateBindingLabels);
  listen(window, 'storage', updateBindingLabels);
  updateBindingLabels();
  listen(window, 'keydown', captureKeys, { capture: true });
  listen(window, 'keyup', captureKeys, { capture: true });
  listen(panel, 'focusin', () => wasm.crystal_social_focus(true));
  listen(panel, 'focusout', () => queueMicrotask(() => wasm.crystal_social_focus(panel.contains(document.activeElement))));
  listen(window, 'blur', () => { swallowed.clear(); wasm.crystal_social_focus(false); });
  const poll = () => {
    for (const line of log.children) line.classList.toggle('chat-faded', window.Date.now() - Number(line.dataset.receivedAt) >= 10000);
    try {
      const state = JSON.parse(wasm.crystal_social_poll());
      speech.update(state);
      if (connected && !state.connected) { requests.replaceChildren(); requestCards.clear(); actions.hidden = true; }
      const connectionChanged = connected !== state.connected;
      connected = state.connected;
      if (connectionChanged) {
        lastCommunityRequest = 0;
        if (connected) { directoryLoaded = false; rankingsLoaded = false; }
        renderCommunity();
      }
      status.textContent = connected ? 'Connected' : 'Reconnecting…';
      status.dataset.connected = String(connected);
      players = state.players ?? [];
      nearby.textContent = 'Nearby · ' + players.filter(p => p.user_id !== selfUserId).length;
      const signature = JSON.stringify(players);
      if (signature !== rosterSignature) {
        rosterSignature = signature;
        roster.replaceChildren(...players.filter(p => p.user_id !== selfUserId).map(p => button(p.display_name, 'player', () => showPlayer(p.user_id, p.display_name))));
        if (selectedPlayer && activeTab === 'chat' && !players.some(p => p.user_id === selectedPlayer)) { actions.hidden = true; selectedPlayer = null; }
      }
      if (state.selected_player) {
        const player = players.find(p => p.user_id === state.selected_player);
        if (player) showPlayer(player.user_id, player.display_name);
      }
      for (const event of state.events) {
        if (event.type === 'social_users' && event.query === directoryQuery) {
          directory = event.users; directoryTotal = event.total; directoryOffset = event.offset; directoryLoaded = true; renderCommunity();
        } else if (event.type === 'leaderboard' && event.metric === metric.value) {
          rankings = event.entries; rankingTotal = event.total; rankingOffset = event.offset; rankingsLoaded = true; renderCommunity();
        } else if (event.type === 'chat') {
          if (event.channel === 'whisper' && event.from_user_id !== selfUserId) replyTarget = event.from_user_id;
          const channel = event.channel === 'say' ? '' : `[${channelLabels[event.channel] ?? event.channel.replace(/^custom:/, '')}] `;
          append(`${channel}${event.text}`, event.channel, event.from_user_id, event.from_display_name);
        } else if (event.type === 'chat_channels') {
          channels = event.channels; updateChannels();
        } else if (event.type === 'welcome') {
          for (const channel of channels.filter(name => !['general', 'trade', 'lfg'].includes(name))) send({ type: 'chat_join', channel });
          for (const channel of ['general', 'trade', 'lfg'].filter(name => !channels.includes(name))) send({ type: 'chat_leave', channel });
        } else if (event.type === 'error') {
          append(event.message);
          for (const control of requests.querySelectorAll('button')) control.disabled = false;
          if (event.code === 'social_error' || event.code === 'invalid_request') actions.hidden = true;
        } else if (event.type === 'interaction_request') {
          const card = document.createElement('div');
          card.append(`${event.from_display_name} · ${event.kind === 'battle' ? 'Battle' : 'Trade'} `);
          for (const accepted of [true, false]) card.append(button(accepted ? 'Accept' : 'Decline', accepted ? 'accept' : 'decline', () => attempt(() => {
            send({ type: 'interaction_response', request_id: event.request_id, target_user_id: event.from_user_id, accepted });
            for (const control of card.querySelectorAll('button')) control.disabled = true;
          })));
          requestCards.get(event.request_id)?.remove();
          requestCards.set(event.request_id, card); requests.append(card);
        } else if (event.type === 'interaction_response') {
          requestCards.get(event.request_id)?.remove(); requestCards.delete(event.request_id);
          actions.hidden = true;
          if (!event.accepted) append('Request declined or cancelled.');
          else setOpen(false);
        } else if (event.type === 'match_found') {
          requests.replaceChildren(); requestCards.clear(); setOpen(false);
        }
      }
      requestCommunity();
    } catch (error) { status.textContent = 'Chat unavailable'; status.dataset.connected = 'false'; connected = false; renderCommunity(); }
  };
  const timer = window.setInterval(poll, 150);
  return () => { speech.destroy(); controller.abort(); document.body.classList.remove('chat-open'); window.clearInterval(timer); window.clearTimeout(searchTimer); wasm.crystal_social_focus(false); panel.remove(); };
}
