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

Saves use the current pack identity. The browser does not migrate older saves.
The menu is available only when customization is enabled.

To add it to another verified pack:

```sh
cargo run -p crystal-assets --bin pack_player_customization -- . \
  content-packs/core-modular.browser.crystalpack \
  content-packs/player-customization.crystalpack
```

The builder preserves the source pack's maps, rules, sprites and audio, adds
the `player-customization` manifest and feature flag, and derives a new verified
pack identity. The Docker build includes the form and stylesheet automatically.

## Save games

Open **Settings → Save games** to save progress, download a `.crystalsave`
backup, restore a file, or delete the current player's save and recovery copy.
Restoring and deleting require confirmation and restart the game. Other
players' saves and multiplayer credentials are unchanged.

**Copy save link** creates a compressed snapshot of the last saved game in a
URL fragment. Share that URL or bookmark it; opening it previews the trainer
and asks before replacing progress. The fragment is not sent to the web server.
Anyone with the link can import the game progress. Links do not update as you
play, and very large saves must be shared as files instead. Rust validates the
current save format, checksum, exact pack identity, and game references before
importing. No older save formats or pack migration are accepted.
