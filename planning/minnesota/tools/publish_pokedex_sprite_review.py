"""Publish reviewed front/back artwork without inventing candidate gameplay data.

Retain the existing website, apply authoritative archives/restorations, and add
clearly labelled art-only candidates. Does not compile or install a game pack.
"""
import argparse
import copy
import html
from html.parser import HTMLParser
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1] / "pokemon"
VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}


class Node:
    def __init__(self, tag="", attrs=()):
        self.tag, self.attrs, self.children = tag, dict(attrs), []

    def nodes(self):
        yield self
        for child in self.children:
            if isinstance(child, Node):
                yield from child.nodes()

    def has(self, cls):
        return cls in self.attrs.get("class", "").split()

    def one(self, cls):
        return next(n for n in self.nodes() if n.has(cls))

    def set_text(self, value):
        self.children = [html.escape(str(value))]

    def render(self):
        body = "".join(c.render() if isinstance(c, Node) else c for c in self.children)
        if not self.tag:
            return body
        attrs = "".join(" " + k if v is None else f' {k}="{html.escape(v, quote=True)}"' for k, v in self.attrs.items())
        return f"<{self.tag}{attrs}>" + ("" if self.tag in VOID else body + f"</{self.tag}>")


class Parser(HTMLParser):
    def __init__(self, text):
        super().__init__(convert_charrefs=False)
        self.root = Node()
        self.stack = [self.root]
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        node = Node(tag, attrs)
        self.stack[-1].children.append(node)
        if tag not in VOID:
            self.stack.append(node)

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if tag not in VOID:
            self.handle_endtag(tag)

    def handle_endtag(self, tag):
        for index in range(len(self.stack) - 1, 0, -1):
            if self.stack[index].tag == tag:
                del self.stack[index:]
                break

    def handle_data(self, text):
        self.stack[-1].children.append(text)

    def handle_entityref(self, name):
        self.handle_data("&" + name + ";")

    def handle_charref(self, name):
        self.handle_data("&#" + name + ";")

    def handle_decl(self, value):
        self.handle_data("<!" + value + ">")

    def handle_comment(self, value):
        self.handle_data("<!--" + value + "-->")


def fragment(text):
    return Parser(text).root.children


def read(path):
    return json.loads(path.read_text())


def e(value):
    return html.escape(str(value), quote=True)


def page_link(member):
    return "/pokedex/species/" + member["name"].lower() + ".html"


def remove_class(node, cls):
    for parent in list(node.nodes()):
        parent.children = [c for c in parent.children if not isinstance(c, Node) or not c.has(cls)]


def image(member, base, view="front", size=112):
    stem = member["id"].lower().replace("_", "-")
    return f'<img src="{base}/sprites/{stem}-{view}.png" alt="{e(member["name"])} {view}" width="{size}" height="{size}" loading="lazy">'


def type_badges(types, styles):
    return "".join(f'<span class="type-badge" aria-label="{e(t.title())} type" style="{e(styles[t])}">{e(t.title())}</span>' for t in types)


def export(baseline, output):
    if output.exists():
        raise ValueError("Use a fresh release directory")
    if "target" not in output.resolve().parts:
        raise ValueError("Review output must stay ignored under target")
    old_base, base = "/pokedex/releases/" + baseline.name, "/pokedex/releases/" + output.name
    shutil.copytree(baseline, output)
    for path in output.rglob("*"):
        if path.is_file() and path.suffix in {".html", ".json", ".js", ".css", ".svg"}:
            path.write_text(path.read_text().replace(old_base, base))
    catalog = read(output / "pokedex.json")
    records = {s["identity"]["id"]: s for s in catalog["species"] + catalog["cut_species"]
               if s.get("status") != "sprite-candidate"}
    roster, curation = read(ROOT / "roster.json"), read(ROOT / "roster-curation.json")
    profiles = {p["id"]: p for p in read(ROOT / "species-profiles.json")["species"]}
    palettes = read(ROOT / "palettes.json")["pokemon"]
    order = {s["id"]: s["number"] for s in read(ROOT / "pokedex-order.json")["species"]}
    retired = {f["id"] for f in curation["retired_families"]}
    families = {f["id"]: f for f in roster["families"]}
    retained = {m["id"]: (f, m) for f in families.values() if f["id"] not in retired for m in f["members"]}
    plan = read(ROOT / "replacement-225-20261104.json")
    candidates = {m["id"]: (f, m) for f in plan["families"] for m in f["members"]}
    if set(candidates) & records.keys():
        raise ValueError("Candidate IDs overwrite existing identities")
    if len(retained) + len(candidates) != 225:
        raise ValueError(f"Expected225 review designs, got{len(retained)} retained +{len(candidates)} candidates")
    document = Parser((output / "index.html").read_text()).root
    styles = {"".join(n.children).upper(): n.attrs.get("style", "") for n in document.nodes() if n.has("type-badge")}
    cards = {n.attrs["data-id"]: n for n in document.nodes() if n.has("pokemon-card")}
    rare = [sid for sid in retained if cards[sid].attrs.get("data-rarity") in {"rare", "pseudo"}]
    ordinary = sorted(set(retained) - set(rare), key=order.get)
    ids = ordinary + list(candidates) + sorted(rare, key=order.get)
    numbers = {sid: i for i, sid in enumerate(ids, 1)}
    archive = sorted(set(records) - set(retained))
    reasons = {f["id"]: f["reason"] for f in curation["retired_families"]}
    removed = {m["id"]: m["reason"] for m in curation["removed_evolutions"]}
    for sid, record in records.items():
        active = sid in retained
        if active:
            record["identity"] = copy.deepcopy(retained[sid][1])
            record["identity"]["regional_number"] = numbers[sid]
            record["normal_rgb555"] = palettes[sid]["gbc_rgb555_words"]
            record["theme"] = retained[sid][0]["theme"]
            record["profile"]["evolution"] = profiles[sid]["evolution"]
            for field in ("classification", "dex_pages"):
                record["profile"][field] = copy.deepcopy(profiles[sid][field])
        record["status"] = "active" if active else "cut"
        record["cut_reason"] = None if active else removed.get(sid, reasons.get(record["family"], "Archived design; preserved for reference."))
        card = cards[sid]
        card.attrs["data-cut"] = str(not active).lower()
        if active:
            card.attrs.pop("hidden", None)
            card.attrs["data-number"] = str(numbers[sid])
            card.attrs["data-types"] = " ".join(record["identity"]["types"])
            profile = profiles[sid]
            moves = [move for _, move in profile["level_moves"]] + profile.get("tmhm_moves", []) + profile.get("egg_moves", [])
            card.attrs["data-search"] = " ".join([record["identity"]["name"], card.attrs["data-types"], retained[sid][0]["theme"], profile["classification"], *moves])
            card.one("type-badges").children = fragment(type_badges(record["identity"]["types"], styles))
            card.one("card-classification").set_text(profiles[sid]["classification"])
        else:
            card.attrs["hidden"] = None
        card.one("card-top").children = fragment(f'<span{chr(32)+"class="+chr(34)+"cut-label"+chr(34) if not active else ""}>{"#"+str(numbers[sid]).zfill(3) if active else "Archived"}</span>')
    for sid, (family, member) in candidates.items():
        manifest = read(ROOT / family["front_back_art"]["manifest"])
        for sprite in manifest["sprites"]:
            if sprite["species_id"] == sid:
                native = ROOT.parents[2] / "target/minnesota-pokemon/native" / family["family_id"]
                shutil.copyfile(native / f'{sprite["id"]}.{sprite["view"]}.png', output / "sprites" / f'{sid.lower().replace("_", "-")}-{sprite["view"]}.png')
        identity = {k: member[k] for k in ("id", "name", "types", "stage")}
        identity["regional_number"] = numbers[sid]
        records[sid] = {"identity": identity, "family": family["family_id"], "status": "sprite-candidate", "theme": family["secondary_theme"], "front_back_only": True,
                        "normal_rgb555": manifest["pokemon_palettes"][sid]["gbc_rgb555_words"]}
        types = " ".join(member["types"])
        cards[sid] = fragment(f'<article class="pokemon-card" data-id="{sid}" data-family="{family["family_id"]}" data-cut="false" data-name="{e(member["name"])}" data-number="{numbers[sid]}" data-total="0" data-types="{types}" data-region="minnesota" data-stage="{member["stage"]}" data-rarity="ordinary" data-search="{e(member["name"]+" "+types+" "+family["base"]+" "+family["secondary_theme"])}"><div class="card-top"><span>#{numbers[sid]:03} · Sprite candidate</span></div><a class="card-link" href="{page_link(member)}"><div class="card-art">{image(member, base)}</div><h3>{e(member["name"])}</h3><p>Front / back review</p></a></article>')[0]
        write_candidate_page(output, base, family, member, numbers[sid])
        cards[sid].one("card-link").children += fragment('<div class="type-badges">' + type_badges(member["types"], styles) + '</div>')
    parent = next(n for n in document.nodes() if any(isinstance(c, Node) and c.has("pokemon-card") for c in n.children))
    parent.children = [cards[sid] for sid in ids + archive]
    family_cards = [n for n in document.nodes() if n.has("family-card") and n.attrs["data-family"] in families]
    for card in family_cards:
        fid = card.attrs["data-family"]
        active = fid in families and fid not in retired
        card.attrs["data-cut"] = str(not active).lower()
        if active:
            card.attrs.pop("hidden", None)
            ms = families[fid]["members"]
            card.attrs["data-number"] = str(numbers[ms[0]["id"]])
            card.one("family-art").children = fragment("".join(f'<a href="{page_link(m)}">{image(m,base)}<strong>{e(m["name"])}</strong></a>' for m in ms))
            remove_class(card, "cut-label")
            top = card.one("family-card-top")
            next(n for n in reversed(top.children) if isinstance(n, Node)).set_text(f"{len(ms)} STAGES")
        else:
            card.attrs["hidden"] = None
    family_parent = next(n for n in document.nodes() if any(isinstance(c, Node) and c.has("family-card") for c in n.children))
    new_family_cards = []
    for family in plan["families"]:
        ms = family["members"]
        art = "".join(f'<a href="{page_link(m)}">{image(m,base)}<strong>{e(m["name"])}</strong></a>' for m in ms)
        new_family_cards += fragment(f'<article class="family-card" data-family="{family["family_id"]}" data-cut="false" data-name="{e(ms[0]["name"])}" data-number="{numbers[ms[0]["id"]]}" data-total="0"><div class="family-card-top"><span>Sprite candidates</span><span>{len(ms)} STAGES</span></div><div class="family-art">{art}</div><p>{e(family["base"]+" + "+family["secondary_theme"])}</p></article>')
    family_parent.children = sorted(family_cards + new_family_cards, key=lambda n: (n.attrs["data-cut"] == "true", int(n.attrs["data-number"])))
    count = len(ids)
    register = document.one("dex-register")
    next(n for n in register.nodes() if n.tag == "strong").set_text(f"001—{count}")
    next(n for n in register.nodes() if n.tag == "small").set_text(f"{count} DESIGNS · FRONT / BACK REVIEW")
    heading = document.one("dex-heading")
    remove_class(heading, "sprite-review-notice")
    heading.children += fragment(f'<p class="sprite-review-notice">{len(retained)} retained designs + {len(candidates)} new front/back candidates. Archived designs are hidden by default. New candidates are artwork for review; gameplay work is pending.</p>')
    for n in document.nodes():
        if n.attrs.get("id") == "result-count":
            n.set_text(f"{count} Pokémon indexed")
        if n.tag == "meta" and n.attrs.get("name") == "description":
            n.attrs["content"] = "Northstar Pokédex:225 front/back designs for review, with requested removals archived."
        if n.has("site-footer"):
            next(x for x in n.nodes() if x.tag == "p").set_text("FRONT / BACK SPRITE REVIEW ·225 DESIGNS")
        if n.has("cut-toggle"):
            next(x for x in n.nodes() if x.tag == "span").set_text("Show archived designs")
    (output / "index.html").write_text(document.render())
    with (output / "style.css").open("a") as stylesheet:
        stylesheet.write("\n.dex-heading{flex-wrap:wrap}.dex-register{flex-shrink:0}.sprite-review-notice{flex-basis:100%;margin:0;font-size:13px;line-height:1.5;max-width:760px}\n")
    for sid in retained.keys() | set(archive):
        update_existing_page(output, base, sid, records, retained, profiles, ids, numbers, styles, palettes)
    catalog.update(species_count=count, family_count=len({records[sid]["family"] for sid in ids}),
                   species=[records[sid] for sid in ids], cut_species=[records[sid] for sid in archive],
                   cut_species_count=len(archive), cut_family_count=len(retired),
                   publication_scope="front-back-sprite-review", retained_species_count=len(retained),
                   sprite_candidate_count=len(candidates), gameplay_implementation_claim=False)
    if "acquisition_source" in catalog:
        catalog["retained_historical_acquisition_source"] = catalog.pop("acquisition_source")
    catalog["candidate_gameplay_data"] = "Not authored or deployed by this front/back review release."
    (output / "pokedex.json").write_text(json.dumps(catalog, separators=(",", ":")))
    (output / "app.js").write_text((output / "app.js").read_text().replace("cut proposal", "archived design"))
    (output / "sprite-review.js").write_text("document.querySelectorAll('[data-sprite-facing]').forEach(b=>b.addEventListener('click',()=>{const p=document.querySelector('#specimen-portrait');p.src=p.dataset[b.dataset.spriteFacing];document.querySelectorAll('[data-sprite-facing]').forEach(x=>x.setAttribute('aria-pressed',String(x===b)));}));\n")
    print(f"Exported{count} front/back review designs: {len(retained)} retained +{len(candidates)} candidates; {len(archive)} archived originals preserved.")


def write_candidate_page(output, base, family, member, number):
    stem = member["id"].lower().replace("_", "-")
    text = f'''<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{e(member["name"])} · Front/back review</title><link rel="stylesheet" href="{base}/style.css"><script defer src="{base}/sprite-review.js"></script><style>.review-art{{image-rendering:pixelated;width:280px;height:280px;object-fit:contain;max-width:100%}}.review-copy{{max-width:680px;margin:32px auto;padding:24px}}button{{min-width:64px;min-height:44px}}.portrait-stage{{background:#fff}}</style></head><body><main class="review-copy"><a href="/pokedex/">← Pokédex</a><p>REVIEW NO.{number:03} · NEW SPRITE CANDIDATE</p><h1>{e(member["name"])}</h1><div class="portrait-stage"><img class="review-art" id="specimen-portrait" src="{base}/sprites/{stem}-front.png" data-front="{base}/sprites/{stem}-front.png" data-back="{base}/sprites/{stem}-back.png" alt="{e(member["name"])} battle sprite"></div><div class="portrait-controls"><button type="button" data-sprite-facing="front" aria-pressed="true">Front</button><button type="button" data-sprite-facing="back" aria-pressed="false">Back</button></div><p>{e(family['base'])} + {e(family['secondary_theme'])}</p><p>{e(' / '.join(member['types']))}</p><p>Front/back artwork for review. Gameplay work is pending.</p></main></body></html>'''
    (output / "species" / (member["name"].lower() + ".html")).write_text(text)


def update_existing_page(output, base, sid, records, retained, profiles, ids, numbers, styles, palettes):
    member = records[sid]["identity"]
    path = output / "species" / (member["name"].lower() + ".html")
    document = Parser(path.read_text()).root
    active = sid in retained
    remove_class(document, "cut-notice")
    breadcrumb = document.one("breadcrumb")
    next(n for n in breadcrumb.children if isinstance(n, Node) and n.tag == "span").set_text(f"REVIEW NO.{numbers[sid]:03}" if active else "ARCHIVED DESIGN")
    next(n for n in document.one("portrait-meta").children if isinstance(n, Node) and n.tag == "span").set_text(f"POKÉDEX /{numbers[sid]:03}" if active else "ARCHIVED")
    if active:
        for node in document.nodes():
            if node.has("type-badges") or node.has("entry-types"):
                node.children = fragment(type_badges(member["types"], styles))
            if node.attrs.get("id") == "palette-normal":
                node.children = fragment("".join(f'<i style="--swatch:{e(color)}" title="{e(color)}"></i>' for color in palettes[sid]["hex"]))
        family = retained[sid][0]
        document.one("entry-intro").one("eyebrow").set_text(profiles[sid]["classification"] + " POKÉMON")
        document.one("dex-prose").children = fragment('<p>' + e(" ".join(" ".join(profiles[sid]["dex_pages"]).replace("@", " ").split())) + '</p>')
        next(n for n in document.one("theme-note").nodes() if n.tag == "p").set_text(family["theme"])
        steps = []
        for m in family["members"]:
            evolution = profiles[m["id"]]["evolution"]
            label = "Final evolution" if evolution is None else "Evolves at Lv." + str(evolution.get("level", "?"))
            steps.append(f'<a class="evolution-step{chr(32)+"current" if m["id"]==sid else ""}" href="{page_link(m)}">{image(m,base)}<strong>{e(m["name"])}</strong><span>{label}</span></a>')
        document.one("evolution-line").children = fragment("".join(steps))
        position = ids.index(sid)
        links = []
        for offset in [-1, 1]:
            target = position + offset
            if 0 <= target < len(ids):
                m = records[ids[target]]["identity"]
                links.append(f'<a href="{page_link(m)}"><span>NO.{numbers[m["id"]]:03}</span><strong>{e(m["name"])}</strong></a>')
        document.one("entry-pager").children = fragment("".join(links))
    else:
        main = next(n for n in document.nodes() if n.tag == "main")
        main.children = fragment(f'<aside class="cut-notice"><strong>Archived design</strong><p>Removed from the current Pokédex lineup.</p><p>{e(records[sid]["cut_reason"])}</p></aside>') + main.children
        document.one("entry-pager").children = fragment('<a href="/pokedex/?cut=1">View archived designs</a>')
    path.write_text(document.render())


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    export(args.baseline, args.output)
