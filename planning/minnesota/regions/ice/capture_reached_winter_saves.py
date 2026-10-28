#!/usr/bin/env python3
"""Retain five actual winter controller Saves as unmodified native GPU scenes.

The renderer receives only --pack/--save/--view/--screenshot. This tool never
decodes or changes a Save, seeds a visual fixture, or overrides its pose/time.
Root executes this after all three main-region controller cases actually pass.
"""

import argparse
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import uuid

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "planning/minnesota/tools"))
from render_archive import retain_render_file, save_render
from render_reference import room_sheet
from render_stages import render_stage

LABELS = (
    "lower-seventeen-stop",
    "hut-after-heal",
    "cache-after-pocket-pages",
    "stationary-ice-wall-stop",
    "after-both-native-trainers",
)
TESTS = (
    "winter_region_controller_both_seam_lanes_long_slides_grass_shelter_and_continue",
    "winter_region_controller_cache_requires_ice_once_wall_stop_turn_and_continue",
    "winter_region_controller_long_snow_return_two_native_battles_pp_money_once_and_continue",
)
TITLE = {
    "lower-seventeen-stop": "Reached17-step western snow stop",
    "hut-after-heal": "Hut after actual HealParty",
    "cache-after-pocket-pages": "Cache after native item/PocketText",
    "stationary-ice-wall-stop": "Reached stationary ICE wall-stop",
    "after-both-native-trainers": "After both real trainer victories",
}


def digest(path):
    return sha256(Path(path).read_bytes()).hexdigest()


def controller_evidence(raw):
    """Require exact passing harness rows, a clean summary and no failed run."""
    text = raw.decode("utf-8")
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", text)
    assert not re.search(
        r"^test result: FAILED\b|^test .+ \.\.\. FAILED\s*$|^failures:\s*$",
        text,
        re.MULTILINE,
    ), "choose a fresh controller log with no failed run"
    summaries = re.findall(
        r"^test result: ok\. (\d+) passed; 0 failed;.*$", text, re.MULTILINE
    )
    assert summaries and sum(map(int, summaries)) >= 3, "missing clean native test summary"
    # With --nocapture --test-threads=1 the harness prints a name before
    # battle diagnostics, then its terminal `ok` on a separate line. Require
    # that exact ownership, never infer a case PASS from only the final count.
    passed = []
    pending = None
    for line in text.splitlines():
        match = re.fullmatch(r"test ([A-Za-z_0-9:]+) \.\.\.(.*)", line)
        if match:
            assert pending is None, "ambiguous interleaved harness; use --test-threads=1"
            name, tail = match.groups()
            if tail.strip() == "ok":
                passed.append(name)
            elif tail.strip().startswith("ignored"):
                continue
            else:
                pending = name
        elif pending is not None and line.strip() == "ok":
            passed.append(pending)
            pending = None
        elif line.strip() == "FAILED":
            raise AssertionError("controller log contains a failed case")
    assert pending is None, "truncated controller case result"
    observed = []
    for name in TESTS:
        matches = [value for value in passed if value.split("::")[-1] == name]
        assert len(matches) == 1, f"need exactly one actual PASS result for {name}"
        observed.append(name)
    return dict(passed_cases=observed, clean_summary_pass_counts=list(map(int, summaries)))


def select_saves(directory):
    """Accept one complete fresh five-Save set, with the actual fixture names."""
    directory = directory.resolve(strict=True)
    assert directory.is_dir(), "--saves must be the fresh controller output directory"
    files = sorted(directory.glob("*.crystalsave"))
    assert len(files) == 5, "require exactly five Saves; do not mix controller attempts"
    selected = {}
    for label in LABELS:
        pattern = re.compile(rf"winter-{re.escape(label)}-([0-9]+)-([0-9]+)\.crystalsave")
        matches = [path for path in files if pattern.fullmatch(path.name)]
        assert len(matches) == 1, f"missing/duplicate exact native Save prefix: {label}"
        path = matches[0]
        assert path.is_file() and not path.is_symlink(), "use actual retained Save files"
        assert path.stat().st_size > 0, f"empty production Save: {path.name}"
        selected[label] = path
    assert set(selected.values()) == set(files), "unexpected Save names in fresh directory"
    return selected


def freeze_json(path, value):
    """Keep every partial/final receipt before updating its convenient alias."""
    raw = (json.dumps(value, indent=2) + "\n").encode()
    frozen = path.with_name(path.stem + "-" + sha256(raw).hexdigest() + path.suffix)
    if frozen.exists():
        assert frozen.read_bytes() == raw
    else:
        with frozen.open("xb") as stream:
            stream.write(raw)
    path.write_bytes(raw)


def gpu_payloads(folder, png, output):
    """Retain and hash actual native raw bytes; never reinterpret their colors."""
    descriptors = sorted((folder / ".render-archive" / png.name).glob("gpu-readback-*.json"))
    assert descriptors, "native screenshot must retain its raw GPU descriptor"
    rows = []
    for descriptor in descriptors:
        data = json.loads(descriptor.read_bytes())
        assert [data["width"], data["height"]] == [640, 576]
        filename = data["raw_file"]
        assert Path(filename).name == filename, "raw GPU filename must be local to its archive"
        raw = descriptor.parent / filename
        assert raw.is_file() and raw.stat().st_size == data["byte_length"] > 0
        raw_hash = digest(raw)
        assert raw.name == f"gpu-readback-{raw_hash}.bin", "native raw-byte identity differs"
        rows.append(dict(
            descriptor=str(descriptor.relative_to(output)),
            descriptor_sha256=digest(descriptor),
            raw_file=str(raw.relative_to(output)),
            raw_sha256=raw_hash,
            byte_length=data["byte_length"],
            format=data["format"],
        ))
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("pack", "binary", "saves", "controller-log", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    pack = args.pack.resolve(strict=True)
    binary = args.binary.resolve(strict=True)
    controller_log = args.controller_log.resolve(strict=True)
    output = args.output.resolve()
    assert pack.is_file() and binary.is_file() and os.access(binary, os.X_OK)
    assert not output.exists(), "--output must be a fresh retained review epoch"
    saves = select_saves(args.saves)
    assert args.saves.resolve() not in [output, *output.parents], "keep output outside input Saves"
    controller_bytes = controller_log.read_bytes()
    evidence = controller_evidence(controller_bytes)
    save_hashes = {label: digest(path) for label, path in saves.items()}
    pack_hash = digest(pack)
    binary_hash = digest(binary)
    log_hash = sha256(controller_bytes).hexdigest()
    assert digest(controller_log) == log_hash, "controller log changed during preflight"
    output.mkdir(parents=True, exist_ok=False)
    inputs = output / "inputs"
    inputs.mkdir()
    (inputs / "controller.log").write_bytes(controller_bytes)
    assert digest(inputs / "controller.log") == log_hash
    for label, path in saves.items():
        shutil.copy2(path, inputs / path.name)
        assert digest(inputs / path.name) == save_hashes[label]
    (output / "capture_reached_winter_saves.py.snapshot").write_bytes(Path(__file__).read_bytes())
    private = ROOT / ".local" / f"winter-reached-capture-{os.getpid()}-{uuid.uuid4().hex}"
    assert not private.exists()
    report = dict(
        status="CAPTURE_PENDING",
        pack=str(pack), pack_sha256=pack_hash,
        binary=str(binary), binary_sha256=binary_hash,
        controller_log=str(controller_log), controller_log_sha256=log_hash,
        controller_evidence=evidence,
        saves=[dict(id=label, path=str(path), sha256=save_hashes[label])
               for label, path in saves.items()],
        fixture=("Controller's explicit initial local Mew80/SURF/Fog/badges/RNG/inventory; "
                 "all later gameplay inputs are production. This is not earned campaign progression."),
        state_overrides=[], saves_decoded_or_modified=False,
        clock_semantics=("Ordinary production Save/Continue RTC, which can advance naturally; "
                         "no synthetic hour or exact frozen-second claim."),
        source_pixel_comparison_performed=False, personally_visually_reviewed=False,
        captures=[], private_binary_removed=False,
    )
    receipt = output / "receipt.json"
    freeze_json(receipt, report)
    env = dict(os.environ,
               CRYSTAL_RENDER_TEST_HEADLESS="1",
               CRYSTAL_RENDER_TEST_CAPTURE_TARGET="image",
               CRYSTAL_RENDER_TEST_WINDOW_SIZE="640x576",
               CRYSTAL_PREVIEW_MUTE="1")
    # These can otherwise make a test renderer add a second view or visual
    # fixture. The command never supplies map/tile/hour/flags/party overrides.
    for key in ("CRYSTAL_RENDER_TEST_SECOND_VIEW", "CRYSTAL_RENDER_TEST_SAVE",
                "CRYSTAL_RENDER_TEST_MAP", "CRYSTAL_RENDER_TEST_X", "CRYSTAL_RENDER_TEST_Y",
                "CRYSTAL_RENDER_TEST_HOUR"):
        env.pop(key, None)
    images = []
    try:
        shutil.copy2(binary, private)
        assert digest(private) == binary_hash, "private executable differs from chosen renderer"
        for label, saved in saves.items():
            assert digest(saved) == save_hashes[label], "controller Save changed before capture"
            folder = output / label
            folder.mkdir()
            png = folder / "production.png"
            command = [str(private), "--pack", str(pack), "--save", str(saved),
                       "--view", "2d", "--screenshot", str(png)]
            row = dict(id=label, save_sha256=save_hashes[label], command=command)
            report["captures"].append(row)
            with (folder / "capture.log").open("xb") as log:
                process = subprocess.run(command, cwd=ROOT, env=env, stdout=log,
                                         stderr=subprocess.STDOUT, timeout=180)
            row["returncode"] = process.returncode
            row["capture_log_sha256"] = digest(folder / "capture.log")
            if png.is_file():
                retain_render_file(png)
                row["production_sha256"] = digest(png)
            assert process.returncode == 0, f"native reached-state renderer failed: {label}"
            assert png.is_file(), f"native screenshot missing: {label}"
            row["production_sha256"] = digest(png)
            retain_render_file(png)
            with Image.open(png) as original:
                assert original.size == (640, 576), "native capture must be640x576"
                actual = render_stage(original, "convert", folder, "review-rgb", "RGB")
            row["raw_gpu_payloads"] = gpu_payloads(folder, png, output)
            assert digest(png) == row["production_sha256"], "actual screenshot must not change"
            assert digest(saved) == save_hashes[label], "native renderer must not rewrite its Save"
            images.append((TITLE[label], actual))
            row["status"] = "NATIVE_REACHED_SAVE_CAPTURED"
            freeze_json(receipt, report)
            print(label, "captured actual production Save", flush=True)
        contact = room_sheet(images, columns=2)
        save_render(contact, output / "reached-winter-contact.png")
        save_render(contact.resize((contact.width // 2, contact.height // 2),
                                   Image.Resampling.NEAREST), output / "reached-winter-contact-preview.png")
        report["status"] = "NATIVE_REACHED_SAVES_CAPTURED_REVIEW_PENDING"
        report["contact_sha256"] = digest(output / "reached-winter-contact.png")
    except Exception as error:
        report["status"] = "FAILED_RETAINED_PARTIAL_CAPTURE"
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        if private.exists():
            private.unlink()
        report["private_binary_removed"] = not private.exists()
        report["pack_unchanged"] = digest(pack) == pack_hash
        report["controller_log_unchanged"] = digest(controller_log) == log_hash
        report["saves_unchanged"] = all(digest(path) == save_hashes[label]
                                        for label, path in saves.items())
        report["retained_saves_exact"] = all(digest(inputs / path.name) == save_hashes[label]
                                           for label, path in saves.items())
        integrity = all(report[key] for key in ("pack_unchanged", "controller_log_unchanged",
                        "saves_unchanged", "retained_saves_exact", "private_binary_removed"))
        if not integrity:
            report["status"] = "FAILED_INPUT_INTEGRITY"
        freeze_json(receipt, report)
        assert integrity, "capture changed an input or failed to remove its private executable"
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
