# Meshtastic Link

Meshtastic Link is an optional Geothite multiplayer transport for the Pokemon
Center Cable Club. It supports the Colosseum, Trade Center, and Time Capsule.
It does not publish overworld position, add multiplayer walking, use the hosted
relay, or alter the loaded game-content pack.

## Native setup

Build with the optional feature:

```sh
cargo build --release -p crystal-bevy --features meshtastic
```

Connect exactly one already-configured radio and select one of its existing
channel indices:

```sh
target/release/crystal-bevy \
  --pack /path/to/core-modular.crystalpack \
  --multiplayer-player-name KRIS \
  --meshtastic-serial /dev/ttyACM0 \
  --meshtastic-channel 0
```

Use `--meshtastic-tcp radio-host:4403` for a network-enabled radio or
`--meshtastic-ble name-or-mac` for Bluetooth LE. These three connection flags
are mutually exclusive. The local Meshtastic node number becomes the stable
multiplayer player ID.

Both players enter the same Cable Club room. Compatible peers advertise only
while waiting in that room. The client with the lower radio node number selects
the displayed peer and presses A to invite; the other client accepts with A (or
declines with B). Both clients then perform Geothite's normal exact protocol,
modpack, content-hash, and save-checkpoint handshake before gameplay.

## Radio configuration and privacy

Geothite uses Meshtastic `PRIVATE_APP` port 256 on the selected existing
channel. It never reads, displays, creates, or changes a channel PSK, region,
modem preset, hop limit, or radio role. Configure the radios beforehand with a
private PSK and matching LongFast channel. Channel 0 is the default only when
`--meshtastic-channel` is omitted.

Discovery packets include the trainer display name, node number, Cable Club
room, protocol version, and a compact compatibility digest. Battle and trade
traffic is sent directly to the selected peer. Initial party/checkpoint transfer
can take noticeably longer than an ordinary turn, especially across relays.

## Browser support

The browser build exposes a permission-gated connection panel when opened with
`?multiplayer=meshtastic`. Chromium-based browsers are the supported target for
Web Serial and Web Bluetooth. Browser APIs, operating-system Bluetooth support,
and radio firmware determine whether each button is available. A radio can
serve only one client connection at a time; disconnect the official Meshtastic
app before connecting Geothite.

## Troubleshooting

- No peers: verify both players selected the same channel index, entered the
  same Cable Club room, use compatible game packs, and are not connected to the
  radio from another app.
- Peer appears and vanishes: check mesh reachability and channel utilization.
  Discovery expires after 45 seconds without a refreshed advertisement.
- Session times out: LongFast with several hops can be slow. Keep both games in
  the Cable Club and avoid sending unrelated high-volume mesh traffic.
- Pack mismatch: both players must load the exact same compiled content pack.
  Meshtastic Link intentionally does not weaken Geothite's compatibility check.

## Licensing and repository boundaries

The optional native feature links the official `meshtastic` Rust crate, which is
GPL-3.0. Distributors of a combined binary must comply with that license. Builds
without the feature do not link the dependency.

This directory is configuration and documentation only. It contains no ROM,
disassembly, exported command dump, duplicate content catalog, generated PCM,
compiled game pack, or build product. Continue supplying game content externally
as described in `docs/game-content.md`.
