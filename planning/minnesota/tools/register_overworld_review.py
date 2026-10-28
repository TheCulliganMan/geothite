#!/usr/bin/env python3
"""Record explicit human/agent visual acceptance of already compiled sprite art.

This does not generate, edit, sample, or export images. The Rust converter owns
native artwork. A selection must name reviewed poses, exact prompts, a separate
source-anatomy receipt and a passing production-context batch whose source hashes
still match the selected bank.
"""

import argparse
import collections
import hashlib
import json
from pathlib import Path
import shutil
import struct


ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "planning/minnesota/pokemon/overworld/20261027-v1"
NAMES = ("overworld.2bpp", "overworld.png", "overworld.gbcpal", "overworld-shiny.gbcpal")
POSES = ("down_idle", "up_idle", "left_idle", "down_step", "up_step", "left_step")


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def reference(path):
    data = path.read_bytes()
    return {"path": str(path.relative_to(ROOT)),
            "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


def decode_bank(data):
    if len(data) != 384:
        raise ValueError("six 16x16 frames require 384 bytes")
    frames = []
    for frame in range(6):
        rows = [[0] * 16 for _ in range(16)]
        for tile in range(4):
            for y in range(8):
                low, high = data[frame * 64 + tile * 16 + y * 2:][:2]
                for x in range(8):
                    rows[tile // 2 * 8 + y][tile % 2 * 8 + x] = (
                        (low >> (7 - x) & 1) | ((high >> (7 - x) & 1) << 1))
        frames.append(["".join(str(p) for p in row) for row in rows])
    return frames


def checked_source_review(item, species, manifest, source_dir, source):
    """Bind the separate anatomical review to the exact converted assets."""
    path = BASE / item["source_feature_review"]
    review = json.loads(path.read_text())
    if (review["species_id"] != species["species_id"]
            or review["result"] != "pass"
            or review["manifest_sha256"] != reference(manifest)["sha256"]
            or review["bank_sha256"] != reference(source_dir / NAMES[0])["sha256"]):
        raise ValueError("source feature review does not bind this manifest and bank")
    expected = {str((manifest.parent / f["source"]).relative_to(ROOT))
                for f in source["frames"]}
    if set(review["source_sha256"]) != expected:
        raise ValueError("source feature review must bind every selected source")
    for name, digest in review["source_sha256"].items():
        if reference(ROOT / name)["sha256"] != digest:
            raise ValueError("source art changed since anatomical review")
    poses = review["nonrear_poses"]
    if ({p["pose"] for p in poses} != {"down_idle", "left_idle", "down_step", "left_step"}
            or len(poses) != 4
            or any(p["result"] != "pass" or not p["source_feature_proof"] for p in poses)
            or review["rear_anatomy"]["result"] != "pass"
            or not review["canonical_eye_form"]):
        raise ValueError("source review needs all four anatomical views and both rears")
    return reference(path)


def checked_entry(item, species):
    token = item["token"].lower()
    manifest = BASE / item["manifest"]
    candidate = BASE / item["candidate"]
    source = next(s for s in json.loads(manifest.read_text())["species"]
                  if s["runtime_token"].lower() == token)
    if source["species_id"] != species["species_id"]:
        raise ValueError("manifest identity differs from active roster")
    index = next(s for s in json.loads((candidate / "compiled-index.json").read_text())["species"]
                 if s["runtime_token"].lower() == token)
    for key in ("species_id", "runtime_token", "fit", "palette_rgb555",
                "shiny_palette_rgb555", "sampling"):
        if source[key] != index[key]:
            raise ValueError("selected manifest differs from compiled species transform")
    if source.get("alpha_threshold", 128) != index["alpha_threshold"]:
        raise ValueError("selected alpha threshold differs from compiled species")
    if (len(source["frames"]) != 6
            or any(any(original[key] != compiled[key] for key in ("pose", "source", "crop"))
                   for original, compiled in zip(source["frames"], index["frames"]))):
        raise ValueError("selected source crops differ from compiled poses")
    source_dir = candidate / "gfx/pokemon" / token
    rows = decode_bank((source_dir / NAMES[0]).read_bytes())
    if tuple(f["pose"] for f in index["frames"]) != POSES:
        raise ValueError("unexpected pose order")
    if rows != [f["rows"] for f in index["frames"]]:
        raise ValueError("native bank differs from reviewed indexed pixels")
    for frame in rows:
        if any(frame[y][x] != "0" for y in range(16) for x in range(16)
               if x in (0, 15) or y in (0, 15)):
            raise ValueError("opaque pixel on native border")
    if any(rows[i] == rows[i + 3] for i in range(3)):
        raise ValueError("standing and stepping artwork must differ")
    binding = species["overworld_palette_binding"]
    for filename, key in [(NAMES[2], "normal_rgb555_words"), (NAMES[3], "shiny_rgb555_words")]:
        if (source_dir / filename).read_bytes() != struct.pack("<4H", *binding[key]):
            raise ValueError("palette differs from species' canonical RGB555 words")
    if struct.unpack(">II", (source_dir / NAMES[1]).read_bytes()[16:24]) != (16, 96):
        raise ValueError("runtime PNG must be 16x96")
    source_review = checked_source_review(item, species, manifest, source_dir, source)
    context = BASE / "diagnostics" / item["batch"]
    log = BASE / "verification" / (item["batch"] + ".log")
    verification = json.loads((context / "verification.json").read_text())
    checked = next(s for s in verification["species"] if s["token"] == token)
    if ("1 passed; 0 failed" not in log.read_text()
            or verification["scene_count"] != 8
            or verification["shiny_bank_frames_checked"] != verification["species_count"] * 6
            or len([f for f in verification["rendered_actor_frames"] if f["token"] == token]) != 8):
        raise ValueError("incomplete production context evidence")
    for filename, digest in checked["assets_sha256"].items():
        if reference(source_dir / filename)["sha256"] != digest:
            raise ValueError("candidate changed since production context test")
    entry = {
        "species_id": species["species_id"], "name": species["name"],
        "family_id": species["family_id"], "dex_number": species["dex_number"],
        "status": "accepted", "generation_tool": "built-in image_gen",
        "selected_manifest": reference(manifest),
        "sources": [reference(manifest.parent / p)
                    for p in dict.fromkeys(f["source"] for f in source["frames"])],
        "prompts": [reference(manifest.parent / p) for p in item["prompts"]],
        "candidate_directory": str(candidate.relative_to(ROOT)),
        "normal_review": reference(candidate / "review" / (token + ".normal-8x.png")),
        "shiny_review": reference(candidate / "review" / (token + ".shiny-8x.png")),
        "compiled_index": reference(candidate / "compiled-index.json"),
        "source_feature_review": source_review,
        "context_evidence": {
            "directory": str(context.relative_to(ROOT)),
            "input_manifest": reference(context / "input-manifest.json"),
            "verification": reference(context / "verification.json"),
            "test_log": reference(log), "normal_actor_poses": 8,
            "all_six_shiny_native_poses_verified": True},
        "root_visual_review": {"accepted": True, "notes": item["visual_review"]}}
    return token, source_dir, entry


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("selection", type=Path)
    parser.add_argument("--reviewed", action="store_true", required=True,
                        help="affirm all selected native/shiny/map boards were visually reviewed")
    args = parser.parse_args()
    coverage = json.loads((BASE / "coverage.json").read_text())
    accepted = json.loads((BASE / "accepted.json").read_text())
    by_name = {s["name"].lower(): s for s in coverage["species"]}
    checked = []
    for item in json.loads(args.selection.read_text())["species"]:
        species = by_name[item["token"].lower()]
        token, source, entry = checked_entry(item, species)
        checked.append((token, source, species, entry))
    for token, source, species, entry in checked:
        runtime = BASE / "native/gfx/pokemon" / token
        runtime.mkdir(parents=True, exist_ok=True)
        for name in NAMES:
            shutil.copy2(source / name, runtime / name)
        entry["runtime_files"] = [reference(runtime / name) for name in NAMES]
        accepted["species"] = [s for s in accepted["species"]
                               if s["species_id"] != species["species_id"]] + [entry]
        write(BASE / "reviews/accepted" / (species["species_id"] + ".json"), entry)
        species.update(status="accepted", selected_overworld_source=entry["sources"],
                       selected_native_candidate={"directory": entry["candidate_directory"],
                           "manifest": entry["selected_manifest"]["path"],
                           "runtime_directory": str(runtime.relative_to(ROOT))},
                       native_visual_review=entry["root_visual_review"],
                       integration_proof=entry["context_evidence"])
    accepted["species"].sort(key=lambda s: s["dex_number"])
    accepted["accepted_species"] = len(accepted["species"])
    accepted["complete"] = accepted["accepted_species"] == accepted["required_species"]
    for species in coverage["species"]:
        sources = sorted((BASE / "sources" / species["family_id"].lower()).glob(
            species["name"].lower() + ".overworld.imagegen-*.png"))
        if sources:
            species["available_generated_sources"] = [reference(p) for p in sources]
            if species["status"] == "awaiting_generation":
                species["status"] = "source_generated"
    for family in coverage["family_jobs"]:
        statuses = [s["status"] for s in coverage["species"]
                    if s["family_id"] == family["family_id"]]
        family["status"] = ("accepted" if all(s == "accepted" for s in statuses)
            else "native_reviewed" if any(s in ("accepted", "native_reviewed") for s in statuses)
            else "source_generated" if any(s == "source_generated" for s in statuses)
            else "awaiting_generation")
    count = accepted["accepted_species"]
    required = coverage["coverage"]["species"]
    families = coverage["coverage"]["families"]
    coverage["coverage"].update(
        accepted_species=count,
        status_counts=dict(collections.Counter(s["status"] for s in coverage["species"])),
        family_status_counts=dict(collections.Counter(f["status"] for f in coverage["family_jobs"])),
        source_generated_species=sum(s["status"] != "awaiting_generation" for s in coverage["species"]),
        source_generated_family_ids=[f["family_id"] for f in coverage["family_jobs"]
                                     if f["status"] != "awaiting_generation"],
        generation_progress_note=f"{count}/{required} accepted; {required-count} unfinished. Full scope remains {required}.")
    write(BASE / "coverage.json", coverage)
    write(BASE / "accepted.json", accepted)
    lines = ["# Northwoods overworld sprite coverage", "",
        f"**{required} active species · {families} families · {required*6:,} native frames required.** {count} accepted; {required-count} unfinished.",
        "", "Six 16×16 frames, three visible RGB555 inks plus transparent index 0, individual normal/shiny palettes and 384-byte 2bpp banks.",
        "All sources, prompts and failed attempts are retained. Native acceptance is separate from pack publication.", "",
        "| Dex | ID | Pokémon | Family | Types | Status |", "| ---: | --- | --- | --- | --- | --- |"]
    for species in sorted(coverage["species"], key=lambda s: s["dex_number"]):
        lines.append(f"| {species['dex_number']:03} | {species['species_id']} | {species['name']} | "
                     f"{species['family_id']} | {' / '.join(species['types'])} | {species['status']} |")
    (BASE / "ROSTER.md").write_text("\n".join(lines) + "\n")
    size_records = []
    for entry in accepted["species"]:
        index = json.loads((ROOT / entry["compiled_index"]["path"]).read_text())
        source = next(s for s in index["species"] if s["species_id"] == entry["species_id"])
        size_records.append({
            "species_id": entry["species_id"], "name": entry["name"],
            "native_frame_size": [16, 16], "transparent_border": True,
            "all_idle_step_pairs_differ": True,
            "frames": [{key: frame[key] for key in ("pose", "native_bounds", "pixel_sha256")}
                       for frame in source["frames"]],
            "production_context": entry["context_evidence"]["directory"]})
    write(BASE / "reviews/native-size-audit.json", {
        "accepted_species": count, "required_species": required,
        "frame_size": [16, 16], "sheet_size": [16, 96], "bank_bytes": 384,
        "palette_bytes": 8, "visible_inks": 3,
        "size_policy": "Inspect actual native rasters and production terrain beside the trainer; enlarged source art is insufficient.",
        "records": size_records})
    print(f"Registered {len(checked)} visually reviewed selections; accepted {count}/{required}.")


if __name__ == "__main__":
    main()
