"""Run planner-grounded local Laya decisions in a visible Geothite browser."""

import argparse
import importlib
import json
from pathlib import Path
import time

from playwright.sync_api import Error as BrowserError, sync_playwright
import laya_policy

INSTALL = r"""async () => {
    const scripts = [...document.scripts].map(s => s.textContent).join('\n');
    const entry = scripts.match(/import\(['"]([^'"]*crystal-bevy[^'"]*\.js)['"]\)/)?.[1];
    if (!entry) throw Error('Geothite WASM entry not found');
    const wasm = await import(new URL(entry, location.href).href);
    const {createGameBridge} = await import(new URL('./webmcp.js', location.href).href);
    window.layaBridge = createGameBridge(wasm);
    window.layaPaused = false;
    const panel = document.createElement('section');
    panel.id = 'laya-panel';
    const style = document.createElement('style');
    style.textContent = `#game-stage {right:340px !important;bottom:0 !important;left:0 !important;top:0 !important}
      #laya-panel {position:fixed;z-index:2147483647;right:0;top:0;bottom:0;width:340px;overflow:auto;padding:18px;background:#101b29;color:#e9f4ff;border-left:1px solid #56758c;font:14px/1.5 system-ui;box-sizing:border-box}
      #laya-dialogue {white-space:pre-wrap;font:18px/1.5 system-ui;background:#1b2d40;padding:12px;border-radius:8px;margin:14px 0}
      #laya-scores {white-space:pre-wrap;font-size:12px}
      @media(max-width:850px) {#game-stage{right:0 !important;bottom:220px !important} #laya-panel{top:auto;height:220px;width:100%;border-left:0;border-top:1px solid #56758c}}`;
    document.head.append(style);
    panel.innerHTML = '<strong style="font-size:20px">Laya · Core ML</strong><div>Planner → Laya → Game Boy buttons</div><p id="laya-status">Loading model…</p><div id="laya-dialogue" aria-live="polite">Current game dialogue will appear here.</div><button id="laya-pause" style="padding:8px 16px;cursor:pointer">Pause</button><details style="margin-top:14px"><summary>Model decisions</summary><pre id="laya-scores"></pre></details><div style="margin-top:8px;font-size:12px">Any game key pauses Laya. Close this window to stop.</div>';
    document.body.append(panel);
    window.layaSetPaused = value => {
        window.layaPaused = value;
        document.querySelector('#laya-pause').textContent = value ? 'Resume' : 'Pause';
        if (value) window.layaBridge.cancel();
    };
    panel.querySelector('button').onclick = () => window.layaSetPaused(!window.layaPaused);
    document.addEventListener('keydown', () => window.layaSetPaused(true), true);
}"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default="https://geothite.ryanculligan.com/?multiplayer=off&flygon=1")
    parser.add_argument("--model", default=".local/laya-model")
    parser.add_argument("--profile", default=".local/laya-browser")
    parser.add_argument("--steps", type=int, default=0)
    parser.add_argument("--interval", type=float, default=0.35)
    parser.add_argument("--channel", default="chrome")
    parser.add_argument("--unassisted", action="store_true", help="Execute raw model top-1 without planner guard")
    parser.add_argument("--diagnostics", default=".local/laya-observation.json")
    args = parser.parse_args()
    with sync_playwright() as pw:
        context = pw.chromium.launch_persistent_context(
            str(Path(args.profile).resolve()), channel=args.channel, headless=False,
            no_viewport=True,
            args=["--disable-background-timer-throttling", "--window-size=1280,800"],
        )
        page = context.pages[0] if context.pages else context.new_page()
        try:
            page.goto(args.url, wait_until="domcontentloaded", timeout=120000)
            page.wait_for_function("document.querySelector('#loading')?.hidden", timeout=180000)
            # A trusted pointer gesture unlocks browser audio. Item fanfares can
            # otherwise leave the game waiting even though joypad input works.
            await_audio = page.get_by_role("button", name="Enable sound", exact=True)
            if await_audio.count():
                await_audio.click()
            page.evaluate(INSTALL)
            print("Browser ready; loading local Core ML model", flush=True)
            import laya_coreml as laya
            model = laya.load(args.model, local_files_only=Path(args.model).exists())
            planner = laya_policy.Planner()
            policy_path = Path(laya_policy.__file__)
            policy_modified = policy_path.stat().st_mtime_ns
            count = interventions = 0
            last_state = None
            unchanged = 0
            while not page.is_closed() and (not args.steps or count < args.steps):
                if page.evaluate("window.layaPaused"):
                    page.wait_for_timeout(200)
                    continue
                # Reload planner edits without losing the live game or its learned map.
                modified = policy_path.stat().st_mtime_ns
                if modified != policy_modified:
                    importlib.reload(laya_policy)
                    planner.__class__ = laya_policy.Planner
                    policy_modified = modified
                observation = page.evaluate("() => layaBridge.execute({kind:'observe'})")
                public = {key: observation[key] for key in ('status', 'observe', 'map_info', 'flow_state', 'recent_events') if key in observation}
                diagnostic = Path(args.diagnostics)
                diagnostic.parent.mkdir(parents=True, exist_ok=True)
                diagnostic.write_text(json.dumps(public, indent=2))
                if observation["flow_state"]["animating"]:
                    page.wait_for_timeout(100)
                    continue
                state_key = (observation['status']['screen'], observation['map_info']['name'],
                             str(observation['map_info']['player']), laya_policy.screen_text(observation),
                             str(observation['observe'].get('rendered_text')))
                unchanged = unchanged + 1 if state_key == last_state else 0
                last_state = state_key
                if unchanged >= 20:
                    page.evaluate("() => {document.querySelector('#laya-status').textContent='No visible progress after 20 actions. Paused for inspection.'; layaSetPaused(true);}")
                    unchanged = 0
                    continue
                plan = planner.plan(observation)
                if not plan.actions:
                    page.evaluate("message => {document.querySelector('#laya-status').textContent=message; layaSetPaused(true);}", plan.objective)
                    continue
                question = laya_policy.verify_prompt(model, plan)
                started = time.perf_counter()
                result = model.predict(plan.context, question)
                elapsed = (time.perf_counter() - started) * 1000
                answer = result["answers"]["action"]
                proposed, executed, button = plan.select(answer, guarded=not args.unassisted)
                interventions += proposed != executed
                count += 1
                data = {"decision": count, "button": button, "ms": round(elapsed),
                        "probabilities": answer["probabilities"], "objective": plan.objective,
                        "proposed": proposed, "executed": executed, "interventions": interventions,
                        "tokens": result["usage"]["input_tokens"], "context": plan.context,
                        "choices": {key: action.description for key, action in plan.actions.items()},
                        "screen": observation["status"]["screen"], "map": observation["map_info"]["name"],
                        "player": observation["map_info"]["player"],
                        "text": laya_policy.screen_text(observation)[:600]}
                print(json.dumps(data), flush=True)
                page.evaluate(r"""data => {
                    document.querySelector('#laya-status').textContent = data.objective;
                    document.querySelector('#laya-dialogue').textContent = data.text || 'Exploring ' + data.map;
                    document.querySelector('#laya-scores').textContent =
                        `Decision ${data.decision} · ${data.ms} ms · ${data.tokens} tokens\n` +
                        `Proposed: ${data.proposed}\nExecuted: ${data.executed} → ${data.button.toUpperCase()}\n` +
                        `Planner interventions: ${data.interventions}\n\n` +
                        Object.entries(data.probabilities).sort((a,b) => b[1]-a[1])
                        .map(([key,p]) => `${key}: ${(p*100).toFixed(1)}%\n${data.choices[key]}`).join('\n\n');
                }""", data)
                if page.evaluate("window.layaPaused"):
                    continue
                try:
                    outcome = page.evaluate("""button => window.layaPaused ? null :
                        layaBridge.execute({kind:'press',button,frames:8})""", button) if button else None
                except BrowserError as error:
                    if "canceled" in str(error).lower():
                        page.evaluate("() => layaSetPaused(true)")
                        continue
                    raise
                if outcome:
                    planner.feedback(observation, button, outcome)
                page.wait_for_timeout(max(0, args.interval) * 1000)
        except KeyboardInterrupt:
            pass
        except Exception:
            if not page.is_closed():
                raise
        finally:
            context.close()


if __name__ == "__main__":
    main()
