#!/usr/bin/env python3
"""Verify exact cave, Pokédex, save-policy, and hosted release preservation."""

import argparse
from base64 import b64decode
from hashlib import file_digest
from io import BytesIO
import json
from pathlib import Path
import sys

SOURCE = Path(__file__).resolve().parent
MINNESOTA = SOURCE.parents[1]
sys.path.insert(0, str(MINNESOTA / "tools"))

from render_progress_atlas import native_room  # noqa: E402
from render_reference import SourceArt, load_pack  # noqa: E402

MANIFEST = "northstar-cave-audit-v5"
POLICY = "data/northstar-save-compatibility.json"
HOSTED_FEATURES = {"server_clock", "player_customization"}
FACINGS = {
    "MnR20Cave1F": "right",
    "MnR12Cave1F": "down",
    "MnIronVentilation": "left",
    "MnIronPumpGallery": "down",
    "MnVoyageursIce1F": "right",
    "MnVoyageursIceB1F": "down",
}


def read(path):
    return json.loads(path.read_text())


def checked_input(path, sha256, content_hash):
    with path.open("rb") as archive:
        actual = file_digest(archive, "sha256").hexdigest()
    assert actual == sha256, (path, "archive hash", actual)
    pack = load_pack(path)
    assert pack["identity"]["content_hash"] == content_hash, (path, "content hash")
    return pack


def subset_overlay(parent, reviewed, keys):
    """Replace just the owned records, including any removed script labels."""
    return {key: value for key, value in parent.items() if key not in keys} | {
        key: reviewed[key] for key in keys if key in reviewed
    }


def script_labels(room):
    attrs = room["attributes"]
    sections = {attrs["map_scripts_label"], attrs["map_events_label"]}
    return sections | room["scripts"].keys() | room["script_text_bodies"].keys()


def expected_data(parent, reviewed, rooms):
    labels = {
        "map_blocks": set(),
        "map_dimensions": set(),
        "map_scripts": set(),
        "runtime_map_metadata": set(),
        "tilesets": set(),
        "map_attributes": rooms.keys(),
        "maps": rooms.keys(),
        "npcs": rooms.keys(),
    }
    for name in rooms:
        current = reviewed["maps"][name]
        attrs = current["attributes"]
        labels["map_blocks"].add(attrs["blocks_label"])
        labels["map_dimensions"].add(attrs["map_constant"])
        labels["runtime_map_metadata"].add(attrs["map_constant"])
        labels["tilesets"].add(attrs["tileset_name"])
        labels["map_scripts"].update(script_labels(current) | script_labels(parent["maps"][name]))
    assert len(labels["tilesets"]) == 7, labels["tilesets"]
    return parent | {
        table: subset_overlay(parent[table], reviewed[table], keys)
        for table, keys in labels.items()
    }, labels["tilesets"]


def expected_files(parent, reviewed, banks, policy):
    files = dict(parent)
    for name in banks:
        for path in (
            f"data/tilesets/{name}.json",
            f"data/tilesets/{name}_metatiles.bin",
            f"data/tilesets/{name}_palette_map.json",
            f"gfx/tilesets/{name}.2bpp",
            f"gfx/tilesets/{name}.png",
        ):
            if path in reviewed:
                files[path] = reviewed[path]
    files[POLICY] = policy
    return files


def assert_mapping(actual, expected, label):
    assert actual.keys() == expected.keys(), (label, actual.keys() ^ expected.keys())
    for key, value in expected.items():
        assert actual[key] == value, (label, key)


def verify_geometry(pack, rooms):
    data = pack["data"]
    maps = data["maps"]
    constants = {room["attributes"]["map_constant"]: name for name, room in maps.items()}
    warps = 0
    for name, layout in rooms.items():
        room = maps[name]
        attrs = room["attributes"]
        width, height = layout["dimensions_metatiles"]
        assert len(room["blocks"]) == width * height, name
        assert [attrs["width"], attrs["height"]] == [width, height], name
        assert room["blocks"] == [int(cell, 16) for row in layout["rows"] for cell in row.split()], name
        assert b64decode(data["map_blocks"][attrs["blocks_label"]]) == bytes(room["blocks"]), name
        x, y = layout["arrivals"][0]
        assert 0 <= x < width * 2 and 0 <= y < height * 2, (name, x, y)
        assert all((obj["x"], obj["y"]) != (x, y) for obj in room["objects"]), (name, x, y)
        tile = room["blocks"][y // 2 * width + x // 2]
        bank = data["tilesets"][attrs["tileset_name"]]
        assert bank["collision"][f"{tile:02x}"][y % 2 * 2 + x % 2] == "FLOOR", (name, x, y)
        for warp in room["events"]["warps"]:
            peer = constants.get(warp["target_map_constant"], warp["target_map"])
            destinations = maps[peer]["events"]["warps"]
            assert sum(dest["index"] == warp["target_warp_id"] for dest in destinations) == 1, (name, warp)
            warps += 1
    assert warps == 64, warps


def png(pack, name):
    room = pack["data"]["maps"][name]
    attrs = room["attributes"]
    if attrs["time_of_day"] == "dark":
        room = dict(room, attributes=dict(attrs, time_of_day="nite"))
    output = BytesIO()
    native_room(room, pack, native_origin=True).save(output, format="PNG")
    return output.getvalue()


def verify_shared_art(parent, release, rooms, banks):
    """Prove the shared cave banks still draw other maps identically."""
    used = {bank: set() for bank in banks}
    shared_maps = 0
    for name, room in parent["data"]["maps"].items():
        attrs = room["attributes"]
        bank = attrs["tileset_name"]
        if name not in rooms and bank in banks:
            used[bank].update(room["blocks"])
            used[bank].add(attrs["border_block"])
            shared_maps += 1

    old_art, new_art = SourceArt(parent), SourceArt(release)
    for bank, blocks in used.items():
        if not blocks:
            continue
        before = parent["data"]["tilesets"][bank]["collision"]
        after = release["data"]["tilesets"][bank]["collision"]
        for block in blocks:
            key = f"{block:02x}"
            assert before.get(key) == after.get(key), ("shared cave collision", bank, key)
        for period in ("morn", "day", "nite", "dark"):
            before = old_art.blocks(bank, period)
            after = new_art.blocks(bank, period)
            for block in blocks:
                assert before[block].tobytes() == after[block].tobytes(), (
                    "shared cave pixels", bank, f"{block:02x}", period
                )
    return shared_maps


def verify_policy(parents, releases, hashes, rooms):
    before = [bytes(pack["runtime_files"][POLICY]) for pack in parents]
    after = [bytes(pack["runtime_files"][POLICY]) for pack in releases]
    assert before[0] == before[1] and after[0] == after[1], "editions disagree on save policy"
    inherited = json.loads(before[0])
    assert inherited["schema_version"] == 11, inherited["schema_version"]
    assert json.loads(after[0]) == {
        "schema_version": 12,
        "sources": [
            {
                "modpack_id": pack["identity"]["runtime_modpack_id"],
                "modpack_hash": sha256,
                "content_hash": pack["identity"]["content_hash"],
            }
            for pack, sha256 in zip(parents, hashes, strict=True)
        ],
        "inherited": inherited,
        "reentry": {
            "policy": "minnesota-cave-audit-v5",
            "maps": [
                {
                    "map_id": name,
                    "arrival": dict(zip(("x", "y"), layout["arrivals"][0], strict=True)),
                    "facing": FACINGS.get(name, "up"),
                }
                for name, layout in rooms.items()
            ],
        },
    }, "schema 12 approvals, inherited policy, or cave arrivals differ"
    return releases[0]["runtime_files"][POLICY]


def verify(parents, reviewed, releases, hashes, rooms):
    policy = verify_policy(parents, releases, hashes, rooms)
    raw_history, hosted_history = [pack["report"]["manifests"] for pack in parents]
    assert hosted_history == raw_history + ["realtime-clock", "player-customization"]
    raw_data, hosted_data = [pack["data"] for pack in parents]
    assert HOSTED_FEATURES <= hosted_data.keys() - raw_data.keys()
    assert_mapping(hosted_data, raw_data | {key: hosted_data[key] for key in HOSTED_FEATURES},
                   "published raw and hosted editions")
    assert MANIFEST not in hosted_history
    manifests = [raw_history + [MANIFEST], raw_history + [MANIFEST] + hosted_history[len(raw_history):]]
    for edition, parent, release, history in zip(
        ("raw", "hosted"), parents, releases, manifests, strict=True
    ):
        data, banks = expected_data(parent["data"], reviewed["data"], rooms)
        files = expected_files(parent["runtime_files"], reviewed["runtime_files"], banks, policy)
        identity = release["identity"]
        content_hash = identity["content_hash"]
        assert int(content_hash, 16) and len(content_hash) == 64, (edition, "content hash")
        assert content_hash != parent["identity"]["content_hash"], edition
        assert_mapping(release["data"], data, edition + " data")
        assert_mapping(release["runtime_files"], files, edition + " runtime files")
        assert_mapping(release, parent | {
            "data": release["data"],
            "runtime_files": release["runtime_files"],
            "report": parent["report"] | {"manifests": history},
            "identity": parent["identity"] | {
                "content_hash": content_hash,
                "runtime_modpack_id": "+".join(history),
            },
        }, edition)
        verify_geometry(release, rooms)
        print(f"{edition}: 19 exact cave modules, 64 warp records, exact published Dex and other parent content")
    shared = verify_shared_art(parents[0], releases[0], rooms, banks)
    for name in rooms:
        assert png(reviewed, name) == png(releases[0], name) == png(releases[1], name), (name, "floorplan PNG")
    print(f"{shared} other maps retain shared cave art and collision")
    print("Both editions match all 19 reviewed floorplan PNGs and preserve exact schema 11 within schema 12")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ("raw_parent", "hosted_parent", "reviewed", "raw_release", "hosted_release"):
        parser.add_argument(arg, type=Path)
    args = parser.parse_args()
    dex = read(MINNESOTA / "world.json")["pokedex_order_revision"]
    review = read(SOURCE / "build.json")
    hashes = (dex["pack_sha256"], dex["hosted_pack_sha256"])
    parents = [checked_input(path, digest, identity) for path, digest, identity in zip(
        (args.raw_parent, args.hosted_parent), hashes,
        (dex["content_identity"], dex["hosted_content_identity"]), strict=True
    )]
    reviewed = checked_input(
        args.reviewed, review["expected_pack_sha256"], review["expected_content_hash"]
    )
    releases = [load_pack(args.raw_release), load_pack(args.hosted_release)]
    layouts = [
        layout
        for group in ("south", "north", "mines", "ice")
        for layout in read(SOURCE / f"{group}.json")
    ]
    rooms = {layout["map"]: layout for layout in layouts}
    assert len(rooms) == len(layouts) == 19
    verify(parents, reviewed, releases, hashes, rooms)


if __name__ == "__main__":
    main()
