# Watch Laya play

`tools/laya-play.py` runs the real [Laya Core ML model](https://github.com/mizorewww/laya-coreml)
locally on an Apple Silicon Mac and opens Geothite in a visible Chrome window.
It reads the public game observation bridge and asks the model to choose each
Game Boy button. The overlay shows the model's probabilities and inference time.
This is an experimental player, not a trained Crystal policy; it can get stuck.

Requires macOS 15+, Python 3.11–3.13, Chrome, and `uv`:

```sh
uv venv --python 3.13 .local/laya-venv
uv pip install --python .local/laya-venv/bin/python laya-coreml==0.1.0 playwright==1.63.0
.local/laya-venv/bin/hf download aac6fef/laya-multilingual-coreml --local-dir .local/laya-model
.local/laya-venv/bin/python tools/laya-play.py
```

The default URL uses the hosted Geothite game with multiplayer disabled.
To play a local browser build instead, pass `--url 'http://localhost:8080/?multiplayer=off&flygon=1'`.
The game and its pack must already be served at that address.
Core ML runs in Python; the browser uses Geothite's existing Rust/WASM client.
No model service, API key, or changes to audio are required.

Click **Pause** to stop decisions and **Resume** to continue. Any keyboard input
pauses the agent so you can play manually. Close its browser window or press
Ctrl-C in the terminal to stop. Use `--steps 20` for a bounded smoke run.
The game follows the actual window dimensions, with a separate responsive
sidebar that mirrors the current dialogue in readable text. Expand **Model
decisions** to see probabilities and action descriptions without covering the game.

The separate Chrome profile, model, and environment stay under ignored `.local/`.
Your usual browser profile is not used. Save through the game's normal menu
if you want to retain progress; a persistent browser profile alone does not
automatically save the game.

The player follows the structure of Laya's Snake demo: ordinary code plans,
Laya ranks context-specific actions, and an explicit guard checks execution.
`tools/laya_policy.py` maintains a goal, observed terrain, map transitions,
visited positions, and failed movement edges. Breadth-first search describes
each direction by whether it is blocked and whether it approaches the current
target. Exits and NPC positions come from the live observation bridge, including
its exposed exit destinations; they are not copied from a content pack.

Dialogue uses `visible_dialogue`, which is the current page, rather than the
`text` field's potentially stale script summary. Menus receive specific
consequences such as accepting a selected answer or closing an unrelated menu.
The model receives a compact objective and candidate descriptions, not a raw
terrain/JSON dump. Input length is checked before prediction. The general
1024-token Core ML CPU/GPU model remains the default.

Every candidate maps to an actual Game Boy button. **There is no wait action.**
The controller handles animation timing before inference and pauses if it
cannot construct an action or sees no progress after 20 repeated observations.
A trusted click enables browser audio at startup so fanfare waits can complete.

The overlay shows the objective, full candidate consequences, model
probabilities, token count, proposed action, executed action, and intervention
count. The guard restricts movement to steps that advance the planned route,
and prevents reversing a known dialogue/menu objective. `--unassisted` disables
these overrides. This is planner-assisted play, not evidence that the model
independently learned Pokemon navigation or story progression.

The current story objective covers setup, leaving home, reaching Elm, obtaining
a starter, and heading west. Other areas use exploration; battles use a basic
confirm/Fight policy. This is not a complete game walkthrough agent.

Game Boy A/B map through the existing Rust bridge to Bevy Z/X. Start maps to
Enter and Select to Right Shift internally, independently of the web page's
physical keyboard bindings. The adapter never mutates game state directly.
Model weights download once and inference remains local; the hosted game still
requires network access. The latest public observation is stored in ignored
`.local/laya-observation.json` for diagnosis. No game packs or audio assets are
created or modified.

Run the policy regression checks:

```sh
.local/laya-venv/bin/python -m unittest discover -s tools -p test_laya_policy.py
```
