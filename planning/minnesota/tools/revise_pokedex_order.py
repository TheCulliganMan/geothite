#!/usr/bin/env python3
"""Publish corrected regional numbers while retaining an existing guide's content."""
import argparse
import html
import json
from pathlib import Path
import re
import shutil
import subprocess

from audit_pokedex_roster import audit


ROOT = Path(__file__).resolve().parents[1] / "pokemon"


def replace_once(text, old, new):
    if text.count(old) != 1:
        raise ValueError(f"Expected exactly one occurrence: {old[:100]}")
    return text.replace(old, new, 1)


def reorder_cards(index, kind, key, numbers, update):
    pattern = re.compile(r'<article class="' + kind + r'".*?</article>', re.S)
    matches = list(pattern.finditer(index))
    if not matches or any(a.end() != b.start() for a, b in zip(matches, matches[1:])):
        raise ValueError(f"Expected a contiguous {kind} grid")
    active, archived = {}, []
    for match in matches:
        card = match.group()
        if 'data-cut="true"' in card:
            archived.append(card)
            continue
        identity = re.search(r'\b' + key + r'="([^"]+)"', card).group(1)
        if identity not in numbers or identity in active:
            raise ValueError(f"Unexpected/duplicate {kind}: {identity}")
        old = int(re.search(r'data-number="(\d+)"', card).group(1))
        new = numbers[identity]
        card = replace_once(card, f'data-number="{old}"', f'data-number="{new}"')
        active[identity] = update(card, old, new)
    if set(active) != set(numbers):
        raise ValueError(f"Missing {kind} entries")
    cards = [active[key] for key in sorted(active, key=numbers.get)] + archived
    return index[:matches[0].start()] + "".join(cards) + index[matches[-1].end():]


def publish(source, output):
    audit(ROOT)
    if output.exists() or not source.is_dir():
        raise ValueError("Provide an existing release and a fresh output directory")
    subprocess.run(["git", "check-ignore", "--quiet", "--no-index", "--", str(output)], check=True)
    if any(path.is_symlink() for path in source.rglob("*")):
        raise ValueError("Source release must not contain symlinks")
    roster = json.loads((ROOT / "roster.json").read_text())
    retired = {f["id"] for f in json.loads((ROOT / "roster-curation.json").read_text())["retired_families"]}
    active = {m["id"] for f in roster["families"] if f["id"] not in retired for m in f["members"]}
    order = json.loads((ROOT / "pokedex-order.json").read_text())["species"]
    numbers = {entry["id"]: n for n, entry in enumerate((e for e in order if e["id"] in active), 1)}
    catalog = json.loads((source / "pokedex.json").read_text())
    species = catalog["species"]
    if len(species) != len(numbers) or {s["identity"]["id"] for s in species} != active:
        raise ValueError("Source release must contain the exact active roster")
    old_numbers = {s["identity"]["id"]: s["identity"]["regional_number"] for s in species}
    family_numbers = {}
    for entry in species:
        identity = entry["identity"]
        identity["regional_number"] = numbers[identity["id"]]
        family = entry["family"]
        family_numbers[family] = min(family_numbers.get(family, len(species) + 1), identity["regional_number"])
    species.sort(key=lambda s: s["identity"]["regional_number"])
    index = (source / "index.html").read_text()

    def update_species(card, old, new):
        search = re.search(r'data-search="([^"]+)"', card).group()
        updated, count = re.subn(r'#' + f'{old:03}' + r'\b', f'#{new:03}', search)
        if count != 1:
            raise ValueError(f"Expected one searchable number: {old}")
        card = replace_once(card, search, updated)
        return replace_once(card, f'<span>#{old:03}</span>', f'<span>#{new:03}</span>')

    index = reorder_cards(index, "pokemon-card", "data-id", numbers, update_species)
    index = reorder_cards(index, "family-card", "data-family", family_numbers, lambda card, old, new: card)
    pages = {}
    for position, entry in enumerate(species):
        identity = entry["identity"]
        name = identity["name"]
        path = Path("species") / f"{name.lower()}.html"
        page = (source / path).read_text()
        old, new = old_numbers[identity["id"]], identity["regional_number"]
        page = replace_once(page, f'REGIONAL NO. {old:03}', f'REGIONAL NO. {new:03}')
        page = replace_once(page, f'POKÉDEX / {old:03}', f'POKÉDEX / {new:03}')

        def neighbor(offset, arrow):
            at = position + offset
            if not 0 <= at < len(species):
                return "<span></span>"
            other = species[at]["identity"]
            return (f'<a href="/pokedex/species/{other["name"].lower()}.html">{arrow}'
                    f'<span>NO. {other["regional_number"]:03}</span>'
                    f'<strong>{html.escape(other["name"])}</strong></a>')

        pager = ('<nav class="entry-pager" aria-label="Adjacent Pokédex entries">'
                 + neighbor(-1, "←") + '<a href="/pokedex/#guide">REGIONAL INDEX</a>'
                 + neighbor(1, "→") + '</nav>')
        page, count = re.subn(r'<nav class="entry-pager".*?</nav>', lambda _: pager, page, flags=re.S)
        if count != 1:
            raise ValueError(f"Expected one entry pager: {name}")
        pages[path] = page
    shutil.copytree(source, output)
    (output / "index.html").write_text(index)
    (output / "pokedex.json").write_text(json.dumps(catalog, ensure_ascii=False, separators=(",", ":")))
    for path, page in pages.items():
        (output / path).write_text(page)
    # Rebase every inherited immutable static link, including older retained art.
    # External cry/showcase media URLs remain unchanged.
    for path in output.rglob("*"):
        if path.is_file() and path.suffix in {".html", ".json", ".js", ".css", ".svg"}:
            text = path.read_text()
            updated = re.sub(r"/pokedex/releases/[A-Za-z0-9_-]+/", f"/pokedex/releases/{output.name}/", text)
            if updated != text:
                path.write_text(updated)
    audit(ROOT, output)
    print(f"Published {len(species)} entries; changed {sum(old_numbers[k] != v for k, v in numbers.items())} regional numbers")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    publish(args.source, args.output)


if __name__ == "__main__":
    main()
