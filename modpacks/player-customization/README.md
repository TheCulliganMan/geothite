# Player customization

The Rust web server enables this modpack in the default hosted browser pack.
Open **Settings → Personalization** or the game's **START → PROFILE** menu
while in the overworld. Trainer name, Chris/Kris sprite, and online handle are
editable without changing the authenticated player ID or trainer ID.

Trainer names support up to 8 uppercase characters; handles support up to 24
letters, numbers, or underscores. Names and sprites are saved with the game;
the handle is stored per browser player and restored on reconnect. Nearby
players receive the selected handle and sprite, and chat uses the handle.
Edits are unavailable during dialogue, battle, and online interactions.

The browser migrates saves from the exact pack that preceded this extension,
leaving the original slot intact. Other modpacks retain their normal identity
checks and do not receive the menu unless customization is enabled.

To add it to another verified pack:

```sh
cargo run -p crystal-assets --bin pack_player_customization -- . \
  content-packs/core-modular.browser.crystalpack \
  content-packs/player-customization.crystalpack
```

The builder preserves the source pack's maps, rules, sprites and audio, adds
the `player-customization` manifest and feature flag, and derives a new verified
pack identity. The Docker build includes the form and stylesheet automatically.
