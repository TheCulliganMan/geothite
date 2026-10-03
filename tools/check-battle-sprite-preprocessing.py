#!/usr/bin/env python3
"""Compare the Rust battle tile normalizer against original pret tools/gfx.c.

Inputs remain external. This test builds only two tiny temporary test programs,
never the application, and writes no source graphics or command dumps to Git.
Pass --inspect-only to verify pack/source inputs without compiling anything.
"""
import argparse
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

from PIL import Image


def load_pack(repository, path):
    spec = importlib.util.spec_from_file_location(
        "external_pack_reader", repository / "tools/check-ship-floor-source.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.load(path)


def png_tiles(data):
    image = Image.open(io.BytesIO(data)).convert("RGBA")
    width, height = image.size
    assert width % 8 == height % 8 == 0
    # Independent source-art conversion. rgbgfx --colors dmg maps the fixed
    # white/light-gray/dark-gray/black levels to indices 0/1/2/3.
    shades = {(255, 255, 255, 255): 0, (170, 170, 170, 255): 1,
              (85, 85, 85, 255): 2, (0, 0, 0, 255): 3}
    output = bytearray()
    for top in range(0, height, 8):
        for left in range(0, width, 8):
            for y in range(8):
                row = [shades[image.getpixel((left + x, top + y))] for x in range(8)]
                for plane in range(2):
                    output.append(sum(((shade >> plane) & 1) << (7 - x)
                                      for x, shade in enumerate(row)))
    return bytes(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--normalizer", type=Path)
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--cc", default="cc")
    parser.add_argument("--inspect-only", action="store_true")
    args = parser.parse_args()
    source = args.source_root
    normalizer = args.normalizer or args.repository / "crates/crystal-bevy/src/bevy_shell/battle_anim_graphics.rs"
    pack_hash = hashlib.sha256(args.pack.read_bytes()).hexdigest()
    pack = load_pack(args.repository, args.pack)
    bundle = json.loads(pack["data"]["battle_anim_bundle"])
    rules = {}
    for line in (source / "Makefile").read_text().splitlines():
        match = re.fullmatch(r"gfx/battle_anims/(\w+)\.2bpp: tools/gfx \+= (.+)", line)
        if match:
            rules[match[1]] = match[2].split()
    assert len(rules) == 17, "source Makefile battle graphics contract changed"
    source_counts = dict((label, int(count)) for count, label in re.findall(
        r"^\s*anim_obj_gfx\s+(\d+),\s*(\w+)",
        (source / "data/battle_anims/object_gfx.asm").read_text(), re.M))
    sheets = []
    for name, (count, label) in bundle["gfx_table"].items():
        if label not in bundle["gfx_sources"]:
            continue
        relative = bundle["gfx_sources"][label].removesuffix(".lz")
        stem = Path(relative).stem
        png_path = str(Path(relative).with_suffix(".png"))
        original_png = (source / png_path).read_bytes()
        assert bytes(pack["runtime_files"][png_path]) == original_png, stem
        raw = png_tiles(original_png)
        assert bytes(pack["runtime_files"][relative]) == raw, stem
        assert count == source_counts[label], (stem, count, source_counts[label])
        sheets.append((stem, count, raw))
    assert len(sheets) == 39
    report = {"pack_sha256": pack_hash, "source_pngs_identical": len(sheets),
              "packed_2bpp_streams_in_unprocessed_png_order": len(sheets),
              "source_declarations_verified": len(sheets), "makefile_rules": len(rules),
              "mode": "source_input_inspection_only" if args.inspect_only else "original_c_vs_rust"}
    if not args.inspect_only:
        with tempfile.TemporaryDirectory(prefix="battle-gfx-oracle-") as directory:
            temp = Path(directory)
            for name in ["gfx.c", "common.h"]:
                shutil.copyfile(source / "tools" / name, temp / name)
            subprocess.run([args.cc, "-std=c99", "-O2", str(temp / "gfx.c"),
                            "-o", str(temp / "gfx")], check=True)
            # Compile the actual production module, not a duplicate algorithm.
            harness = temp / "check.rs"
            harness.write_text("#[path = " + json.dumps(str(normalizer.resolve())) + "]\n"
                + "mod battle_anim_graphics;\nfn main() {\n"
                + "let a: Vec<String> = std::env::args().collect();\n"
                + "let raw = std::fs::read(&a[2]).unwrap();\n"
                + "let normalized = battle_anim_graphics::normalize(&a[1], raw, a[3].parse().unwrap()).unwrap();\n"
                + "std::fs::write(&a[4], normalized).unwrap();\n}\n")
            subprocess.run([args.rustc, "--edition=2024", str(harness), "-o", str(temp / "check")], check=True)
            subprocess.run([args.rustc, "--edition=2024", "--test", str(harness), "-o", str(temp / "unit-tests")], check=True)
            subprocess.run([str(temp / "unit-tests")], check=True)
            changed = []
            for stem, count, raw in sheets:
                raw_path, expected_path, actual_path = [temp / name for name in ["raw.2bpp", "expected.2bpp", "actual.2bpp"]]
                raw_path.write_bytes(raw)
                # C source is the independent expected result; no C routines
                # or generated source-art fixtures are copied into the repo.
                subprocess.run([str(temp / "gfx"), *rules.get(stem, []), "-o", str(expected_path), str(raw_path)], check=True)
                expected = expected_path.read_bytes()
                assert len(expected) == count * 16, (stem, len(expected), count)
                subprocess.run([str(temp / "check"), stem, str(raw_path), str(count), str(actual_path)], check=True)
                assert actual_path.read_bytes() == expected, stem
                # Run the already-correct stream through the same API too.
                subprocess.run([str(temp / "check"), stem, str(expected_path), str(count), str(actual_path)], check=True)
                assert actual_path.read_bytes() == expected, (stem, "double preprocessing")
                if raw != expected:
                    changed.append(stem)
            assert len(changed) == 17
            report.update(compiled_sheets_byte_identical=39, already_compiled_sheets_unchanged=39,
                          corrected_sheets=changed, rust_unit_tests=8)
    assert hashlib.sha256(args.pack.read_bytes()).hexdigest() == pack_hash
    report["pack_unchanged"] = True
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
