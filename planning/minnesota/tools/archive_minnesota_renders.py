#!/usr/bin/env python3
"""Preserve render bytes and authoring records without editing any image."""

import hashlib
import json
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
ARCHIVE = ROOT / "planning/minnesota/renders/creature-archive"


def main():
    objects = ARCHIVE / "objects"
    objects.mkdir(parents=True, exist_ok=True)
    records = []
    directories = [
        # Preserve historical originals too, outside disposable build caches.
        ROOT / "target/minnesota-pokemon/archive",
        ROOT / "planning/minnesota/pokemon",
        ROOT / "target/minnesota-pokemon/native",
        ROOT / "target/minnesota-pokemon/gen2-reference",
        ROOT / "target/minnesota-pokemon/imagegen-native",
        ROOT / "target/minnesota-pokemon/rejected-restart",
        ROOT / "target/minnesota-pokemon/native-runtime",
        ROOT / "target/minnesota-pokemon/restart-native",
        ROOT / "target/minnesota-pokemon/crystal-style-reference.png",
        ROOT / "target/minnesota-pokemon/outline-review",
        ROOT / "target/minnesota-pokemon/proposals",
        ROOT / "target/agent-roster-contour/new-renders",
        ROOT / "target/agent-roster-contour/starter-refinement",
        ROOT / "target/agent-northwoods-animation/proposals",
        ROOT / "target/northwoods-wildlife-review",
    ]
    # Native battle frame captures and APNGs are reusable authored previews.
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("signatures-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("proposals-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("line076-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("starters-battle-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("roster-battles-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("roster-polish-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("taconarch-redesign-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("fairy-balance-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("moves-animation-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("sticky-note-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("new-sprite-refinement-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("flarkindle-redesign-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("mudmender-redesign-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("bear-differentiation-*")))
    directories.extend(sorted((ROOT / "planning/minnesota/renders").glob("sprite-publication-*")))
    directories.extend(sorted((ROOT / "target/agent-northwoods-moves").glob("beaver-*")))
    directories.extend(sorted((ROOT / "target/agent-roster-contour").glob("MN_F*")))
    directories.extend(sorted((ROOT / "target/agent-roster-contour").glob("low-*/MN_F*")))
    # Preserve exported palette variants as reusable artwork, without archiving
    # the generated website's HTML, scripts, or other build products.
    web_releases = ROOT / "target/minnesota-pokedex-web/pokedex/releases"
    directories.extend(sorted(web_releases.rglob("*.png")))
    directories.extend(sorted(web_releases.rglob("*.apng")))
    for directory in directories:
        sources = [directory] if directory.is_file() else sorted(directory.rglob("*"))
        for source in sources:
            if not source.is_file():
                continue
            data = source.read_bytes()
            digest = hashlib.sha256(data).hexdigest()
            destination = objects / (digest + source.suffix)
            if destination.exists():
                if destination.read_bytes() != data:
                    raise ValueError(f"archive object mismatch: {destination}")
            else:
                with destination.open("xb") as output:
                    output.write(data)
            records.append({
                "path": str(source.relative_to(ROOT)),
                "sha256": digest,
                "bytes": len(data),
                "object": str(destination.relative_to(ROOT)),
            })
    inventory = json.dumps({"schema_version": 1, "files": records}, indent=2).encode()
    digest = hashlib.sha256(inventory).hexdigest()
    snapshot = ARCHIVE / f"inventory-{digest}.json"
    if not snapshot.exists():
        with snapshot.open("xb") as output:
            output.write(inventory)
    with tempfile.NamedTemporaryFile(dir=ARCHIVE, prefix="latest-", suffix=".tmp", delete=False) as output:
        temporary = Path(output.name)
        output.write(inventory)
    try:
        temporary.replace(ARCHIVE / "latest.json")
    finally:
        temporary.unlink(missing_ok=True)
    print(f"Archived {len(records)} files; immutable inventory {snapshot.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
