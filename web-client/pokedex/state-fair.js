/* Playback controls only. All imagery comes from the native game renderer. */
(() => {
  const video = document.querySelector('[data-showcase-video]');
  const play = document.querySelector('[data-showcase-play]');
  const status = document.querySelector('[data-showcase-status]');
  if (!video || !play || !status) return;
  let pendingTime = null;
  function seek() {
    if (pendingTime === null || !Number.isFinite(video.duration)) return;
    video.currentTime = Math.min(pendingTime, Math.max(0, video.duration - 0.1));
    pendingTime = null;
  }
  async function start(seconds) {
    pendingTime = seconds;
    if (!video.getAttribute('src')) video.src = video.dataset.src;
    if (video.readyState >= 1) seek();
    video.scrollIntoView({ block: 'center', behavior: window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth' });
    try { await video.play(); }
    catch { status.textContent = 'Playback could not start. Try Play again, or open the full video below.'; play.hidden = false; }
  }
  video.addEventListener('loadedmetadata', seek);
  play.addEventListener('click', () => start(0));
  document.querySelector('[data-showcase-start]')?.addEventListener('click', () => start(0));
  document.querySelectorAll('[data-showcase-time]').forEach(button => {
    button.addEventListener('click', () => start(Number(button.dataset.showcaseTime)));
  });
  video.addEventListener('playing', () => { play.hidden = true; status.textContent = 'Playing State Fair Chaos. Use the video controls to pause or go full screen.'; });
  video.addEventListener('pause', () => { if (!video.ended) status.textContent = 'Paused. Resume with the video controls or choose a scene.'; });
  video.addEventListener('ended', () => { play.hidden = false; play.querySelector('strong').textContent = 'Replay State Fair Chaos'; status.textContent = 'That’s the fair. Pick a scene or play it all again.'; });
  video.addEventListener('error', () => { play.hidden = false; status.textContent = 'The video could not load. Try Play again, or open the full video below.'; });
  document.addEventListener('visibilitychange', () => { if (document.hidden) video.pause(); });
  window.addEventListener('pagehide', () => video.pause());
})();
