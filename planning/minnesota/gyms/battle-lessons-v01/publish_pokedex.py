#!/usr/bin/env python3
"""Build an additive Northwoods Pokédex release from its immutable predecessor."""
import argparse
from collections import Counter
import json
from pathlib import Path
import re
import shutil
import tempfile

ROOT = Path(__file__).resolve().parents[4]
PROFILE = ROOT / "planning/minnesota/pokemon/species-profiles.json"
PANEL = re.compile(r'(<div role="tabpanel" id="moves-machine"[^>]*><table[^>]*>.*?<tbody>)(.*?)(</tbody>)', re.S)
ROW = re.compile(r'<tr><td class="move-level">([^<]+)</td><td>([^<]+)</td>.*?</tr>', re.S)
COUNT = re.compile(r'(id="tab-machine"[^>]*>TM / HM <span>)(\d+)(</span>)')


def replace_one(text, old, new):
    if text.count(old) != 1:
        raise ValueError("Missing or duplicated release record: " + old[:100])
    return text.replace(old, new, 1)


def machine_rows(html):
    panel = PANEL.search(html)
    if not panel:
        raise ValueError("Missing TM/HM panel")
    return [(row[1], row[2], row[0]) for row in ROW.finditer(panel[2])]


def update_page(html, new_rows):
    old_rows = machine_rows(html)
    if len(new_rows) != len(old_rows) + 1 or COUNT.search(html)[2] != str(len(old_rows)):
        raise ValueError("Existing move tab and its count differ")
    text = PANEL.sub(lambda m: m[1] + "".join(new_rows) + m[3], html, count=1)
    return COUNT.sub(lambda m: m[1] + str(len(new_rows)) + m[3], text, count=1)


def export(base, destination):
    if destination.exists() or destination.name == base.name:
        raise ValueError("Export requires a new immutable release directory")
    records = json.loads((base / "pokedex.json").read_text())
    canonical = {r["id"]: r["tmhm_moves"] for r in json.loads(PROFILE.read_text())["species"]}
    catalog = {m["move_id"]: m for m in records["machine_catalog"]}
    order = {m["code"]: i for i, m in enumerate(records["machine_catalog"])}
    targets = []
    totals = Counter()
    for entry in records["species"]:
        identity = entry["identity"]
        before = entry["profile"]["tmhm_moves"]
        after = canonical[identity["id"]]
        if before == after:
            continue
        if after[:-1] != before or after[-1] not in {"THUNDERPUNCH", "STEEL_WING"}:
            raise ValueError(f"Unrelated publication difference for {identity['name']}")
        targets.append((entry, catalog[after[-1]]))
        totals[after[-1]] += 1
    if totals != {"THUNDERPUNCH": 7, "STEEL_WING": 10}:
        raise ValueError(f"Unexpected regional compatibility changes: {totals}")

    # The existing guide already describes both machines, including Steel Wing
    # on archived species. Reuse their captions, effects and type presentation.
    needed = {machine["code"] for _, machine in targets}
    donors = {}
    for page in sorted((base / "species").glob("*.html")):
        for code, label, row in machine_rows(page.read_text()):
            if code in needed:
                if code in donors and donors[code] != (label, row):
                    raise ValueError("Inconsistent native move presentation: " + code)
                donors[code] = (label, row)
        if donors.keys() == needed:
            break

    index = (base / "index.html").read_text()
    suffix = "/pokedex.json"
    index = replace_one(index, f"/pokedex/releases/{base.name}{suffix}",
                        f"/pokedex/releases/{destination.name}{suffix}")
    with tempfile.TemporaryDirectory(dir=destination.parent) as scratch:
        output = Path(scratch) / destination.name
        shutil.copytree(base, output)
        for entry, machine in targets:
            identity, code = entry["identity"], machine["code"]
            label, row = donors[code]
            source = output / "species" / (identity["name"].lower() + ".html")
            html = source.read_text()
            rows = {c: text for c, _, text in machine_rows(html)}
            expected = [m["code"] for m in entry["machine_learnset"]]
            if list(rows) != expected or code in rows:
                raise ValueError("Species HTML and JSON disagree: " + identity["name"])
            rows[code] = row
            ordered = sorted(rows, key=order.__getitem__)
            source.write_text(update_page(html, [rows[c] for c in ordered]))
            entry["machine_learnset"].append(machine)
            entry["machine_learnset"].sort(key=lambda m: order[m["code"]])
            entry["profile"]["tmhm_moves"].append(machine["move_id"])
            pattern = re.compile(r'(<article class="pokemon-card" data-id="' + re.escape(identity["id"])
                                 + r'"[^>]*? data-search=")([^"]*)(")')
            matches = list(pattern.finditer(index))
            if len(matches) != 1:
                raise ValueError("Expected one searchable card for " + identity["name"])
            card = matches[0]
            search = card[2]
            if label.lower() not in search.lower():
                search += " " + label
            if code not in search.split():
                search += " " + code
            index = replace_one(index, card[0], card[1] + search + card[3])
        (output / "pokedex.json").write_text(json.dumps(records, indent=2) + "\n")
        (output / "index.html").write_text(index)
        output.rename(destination)
    print(f"Updated {len(targets)} machine tabs, catalog records and search cards in {destination}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    export(args.base, args.destination)
