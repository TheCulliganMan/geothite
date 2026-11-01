#!/usr/bin/env python3
"""Optional QA: execute the pinned ROM in headless PyBoy, never Geothite's renderer.

Install PyBoy 2.8.1 and Pillow in an external virtualenv. ROM, symbols and all
captures stay external; this is not a content-pack builder or audio exporter.
"""
import argparse
import hashlib
import importlib.metadata
import io
import json
import re
import subprocess
from pathlib import Path

ROM_SHA1 = "f4cd194bdee0d04ca4eac29e09b8e4e9d818c133"
ITEMS = [("POTION", 23), ("ANTIDOTE", 1), ("POKE_BALL", 12),
         ("GREAT_BALL", 2), ("ESCAPE_ROPE", 3), ("REPEL", 4)]


def constants(path):
    result, value = {}, 0
    for raw in path.read_text().splitlines():
        line = raw.split(";", 1)[0].strip()
        if line.startswith("const_def"):
            args = line.split()
            value = int(args[1].replace("$", "0x"), 0) if len(args) > 1 else 0
        elif line.startswith("const "):
            result[line.split()[1]] = value
            value += 1
    return result


class Oracle:
    def __init__(self, vendor, mode, species="CYNDAQUIL", dvs=0xff):
        from pyboy import PyBoy
        self.symbols = {}
        for line in (vendor / "pokecrystal.sym").read_text().splitlines():
            match = re.match(r"^([0-9a-fA-F]+):([0-9a-fA-F]+)\s+(\S+)", line.strip())
            if match:
                bank, address, name = match.groups()
                self.symbols[name] = int(bank, 16), int(address, 16)
        self.pyboy = PyBoy(str(vendor / "pokecrystal.gbc"), window="null",
                           symbols=str(vendor / "pokecrystal.sym"),
                           sound_volume=0, sound_emulated=False,
                           ram_file=io.BytesIO(bytes(0x8000)))
        self.pyboy.set_emulation_speed(0)
        self.pyboy.tick(600, False)
        self.pyboy.memory[0xff70] = 1
        # Fixture entry state only: use the source indoor day/text palette and
        # source fast text. No rendered menu contents, tilemap, font or palette
        # pixels are supplied; those are produced by the ROM routines below.
        for name, value in [("wEnvironment", 3), ("wMapTileset", 6 if mode == "pc" else 7),
                            ("wTimeOfDayPal", 1), ("hVBlank", 0), ("wOptions", 1),
                            ("wSpriteUpdatesEnabled", 0)]:
            self.put(name, value)
        self.items = constants(vendor / "constants/item_constants.asm")
        self.calls = []
        self.plan = ["ClearWindowData", "LoadStandardFont", "LoadFontsExtra", "LoadMapPals", "ClearScreen", "ClearSprites", "WaitBGMap2"]
        if mode == "naming":
            species_id = constants(vendor / "constants/pokemon_constants.asm")[species]
            for name, value in [("wCurPartySpecies", species_id), ("wPartyCount", 1),
                                ("wPartySpecies", species_id), ("wPartyMon1Species", species_id),
                                ("wCurPartyMon", 0), ("wMonType", 0), ("wPartyMon1DVs", dvs)]:
                self.put(name, value)
            self.pyboy.memory[self.address("wPartyMon1DVs") + 1] = 0xff
            self.plan += ["NamingScreen"]
        elif mode == "pc":
            self.put("wNumPCItems", len(ITEMS))
            at = self.address("wPCItems")
            for i, (name, quantity) in enumerate(ITEMS):
                self.pyboy.memory[at + 2*i:at + 2*i + 2] = [self.items[name], quantity]
            self.pyboy.memory[at + 2*len(ITEMS)] = 255
            self.put("wPCItemsCursor", 1)
            self.put("wPCItemsScrollPosition", 0)
            self.plan += ["PlayerWithdrawItemMenu"]
        elif mode.startswith("deposit"):
            items = [("POTION",23)] if mode == "deposit_quantity" else [
                ("POTION",23),("ANTIDOTE",1),("ESCAPE_ROPE",3),("REPEL",4),
                ("SUPER_POTION",5),("HYPER_POTION",6)]
            self.put("wNumItems",len(items))
            at=self.address("wItems")
            for i,(name,quantity) in enumerate(items):
                self.pyboy.memory[at+2*i:at+2*i+2]=[self.items[name],quantity]
            self.pyboy.memory[at+2*len(items)]=255
            self.plan += ["PlayerDepositItemMenu"]
        else:
            self.pyboy.memory[0x0000] = 0x0a  # MBC SRAM enable
            self.pyboy.memory[0x4000] = 0
            self.put("sMailboxCount", 6)
            size = self.address("sMailbox2") - self.address("sMailbox1")
            author = self.address("sMailbox1Author") - self.address("sMailbox1")
            for i in range(6):
                at = self.address("sMailbox1") + i*size
                record = [0x50] * size
                # Source charmap A..Z and 0..9; the menu reads SRAM itself.
                record[author:author+8] = [0x8c, 0x80, 0x88, 0x8b, 0xf6+i, 0x50, 0x50, 0x50]
                record[-1] = self.items["FLOWER_MAIL"]
                self.pyboy.memory[at:at+size] = record
            self.pyboy.memory[0x0000] = 0
            self.plan += ["_PlayerMailBoxMenu"]
        self.pyboy.hook_register(0, 0x100, self.enter, None)
        stack = self.address("wStackBottom")
        # Normalize interrupt enable at the fixture boundary. All following
        # graphics/menu work executes the unchanged source ROM routines.
        self.pyboy.memory[stack:stack+4] = [0xfb, 0xc3, 0x00, 0x01]  # EI; JP return hook
        self.pyboy.register_file.SP = self.address("wStackTop")
        self.pyboy.register_file.PC = stack
        self.pyboy.tick(180, True)
        if self.plan:
            raise RuntimeError(f"source initialization did not finish: {self.plan}")

    def address(self, name):
        return self.symbols[name][1]

    def put(self, name, value):
        self.pyboy.memory[self.address(name)] = value

    def read(self, name):
        return self.pyboy.memory[self.address(name)]

    def enter(self, _):
        if not self.plan:
            raise RuntimeError("menu unexpectedly returned from fixture entry")
        name = self.plan.pop(0)
        bank, address = self.symbols[name]
        self.pyboy.memory[0x2000] = bank or 1
        self.put("hROMBank", bank or 1)
        sp = self.address("wStackTop") - 2
        self.pyboy.memory[sp:sp+2] = [0, 1]
        self.pyboy.register_file.SP = sp
        self.pyboy.register_file.PC = address
        if name == "NamingScreen":
            # NAME_MON, destination points at the party nickname buffer. The
            # source initializer clears it; no screen pixels are provided.
            destination = self.address("wPartyMonNicknames")
            self.pyboy.register_file.B = 0
            self.pyboy.register_file.D = destination >> 8
            self.pyboy.register_file.E = destination & 255
        self.calls.append(name)

    def press(self, button):
        self.pyboy.button_press(button)
        self.pyboy.tick(8, True)
        self.pyboy.button_release(button)
        self.pyboy.tick(40, True)

    def image(self, path):
        self.pyboy.screen.image.save(path)

    def record(self, button=None):
        return {"button": button, "scroll": self.read("wMenuScrollPosition"),
                "cursor": self.read("wMenuCursorY"), "switch_item": self.read("wSwitchItem")}

    def close(self):
        self.pyboy.stop(save=False)


def capture(vendor, output, family="pc"):
    version = importlib.metadata.version("pyboy")
    if version != "2.8.1":
        raise RuntimeError(f"expected pinned PyBoy 2.8.1, found {version}")
    rom = vendor / "pokecrystal.gbc"
    if hashlib.sha1(rom.read_bytes()).hexdigest() != ROM_SHA1:
        raise RuntimeError("ROM does not match the pinned assembled source")
    repository = Path(__file__).resolve().parent.parent
    if output == repository or repository in output.parents:
        raise RuntimeError("ROM reference captures must remain outside the repository")
    output.mkdir(parents=True, exist_ok=True)
    if family == "naming":
        capture_naming(vendor, output, version)
        return
    pc = Oracle(vendor, "pc")
    try:
        pc.image(output / "pc-items-initial.png")
        records = [pc.record()]
        for _ in range(8):
            pc.press("down")
            records.append(pc.record("down"))
        (output / "pc-items-scroll.json").write_text(json.dumps({"rom_sha1": ROM_SHA1, "records": records}, indent=2))
    finally:
        pc.close()
    pc = Oracle(vendor, "pc")
    try:
        records = []
        for i, button in enumerate([None, "select", "down", "down", "a", "select", "down", "b"]):
            if button:
                pc.press(button)
            records.append(pc.record(button))
            pc.image(output / f"pc-items-move-{i}.png")
        (output / "pc-items-move.json").write_text(json.dumps({"rom_sha1": ROM_SHA1, "records": records}, indent=2))
    finally:
        pc.close()
    for mode,stem in [("pc","pc-withdraw-quantity"),("deposit_quantity","pc-deposit-quantity")]:
        menu=Oracle(vendor,mode)
        try:
            menu.press("a")
            records=[]
            for i,button in enumerate([None,"up","right","down"]):
                if button:menu.press(button)
                records.append({"button":button,"quantity":menu.read("wItemQuantityChange")})
                menu.image(output/f"{stem}-{i}.png")
            (output/f"{stem}.json").write_text(json.dumps({"rom_sha1":ROM_SHA1,"records":records},indent=2))
        finally:menu.close()
    menu=Oracle(vendor,"deposit")
    try:
        directory=output/"pc-deposit-scroll";directory.mkdir(exist_ok=True)
        records=[]
        for i,button in enumerate([None]+["down"]*8+["up"]*3):
            if button:menu.press(button)
            records.append(menu.record(button))
            menu.image(directory/f"rom-pc-deposit-scroll-{i}.png")
        (directory/"pc-deposit-scroll.json").write_text(json.dumps({"rom_sha1":ROM_SHA1,"records":records},indent=2))
    finally:menu.close()
    mail = Oracle(vendor, "mailbox")
    try:
        directory = output / "mailbox"
        directory.mkdir(exist_ok=True)
        for i in range(7):
            if i:
                mail.press("down")
            mail.image(directory / f"rom-mailbox-{i}.png")
        mail.press("up")  # CANCEL -> sixth letter, the source submenu fixture
        mail.press("a")
        mail.image(directory / "rom-mailbox-actions.png")
        mail.press("down")
        mail.press("a")
        mail.image(directory / "rom-mailbox-confirm.png")
    finally:
        mail.close()
    revision = subprocess.check_output(["git", "-C", str(vendor), "rev-parse", "HEAD"], text=True).strip()
    provenance = {"rom_sha1": ROM_SHA1, "source_revision": revision,
                  "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  "items_constants_sha256": hashlib.sha256((vendor/"constants/item_constants.asm").read_bytes()).hexdigest(),
                  "symbols_sha256": hashlib.sha256((vendor/"pokecrystal.sym").read_bytes()).hexdigest(),
                  "emulator": "PyBoy", "version": version, "window": "null", "sound_emulated": False,
                  "method": "fixture RAM entry; original ROM font/palette/menu routines and ordinary joypad inputs",
                  "source_pixels_supplied": False, "geothite_renderer_used": False}
    (output / "menu-oracle-provenance.json").write_text(json.dumps(provenance, indent=2))
    print(f"Captured source PC and Mailbox references in {output}", flush=True)


def capture_naming(vendor, output, version):
    """Independent nickname backgrounds, gender, animated icon and editing."""
    cases = [("cyndaquil-male", "CYNDAQUIL", 0xff),
             ("cyndaquil-female", "CYNDAQUIL", 0x00),
             ("magnemite", "MAGNEMITE", 0xff)]
    for stem, species, dvs in cases:
        menu = Oracle(vendor, "naming", species, dvs)
        try:
            directory = output / stem
            directory.mkdir(exist_ok=True)
            records = []
            for frame in range(32):
                menu.image(directory / f"idle-{frame:02}.png")
                records.append({"frame": frame, "icon_frame_index": menu.read("wSpriteAnim1FrameIndex"),
                                "icon_duration": menu.read("wSpriteAnim1Duration")})
                menu.pyboy.tick(1, True)
            editing = []
            for index, button in enumerate([None, "a", "right", "a", "b", "select", "a", "start"]):
                if button:
                    menu.press(button)
                menu.image(directory / f"edit-{index}.png")
                editing.append({"button": button, "length": menu.read("wNamingScreenCurNameLength"),
                                "case": menu.read("wNamingScreenLetterCase"),
                                "destination": list(menu.pyboy.memory[menu.address("wPartyMonNicknames"):
                                                        menu.address("wPartyMonNicknames") + 11])})
            (directory / "trace.json").write_text(json.dumps({"species": species, "dvs": [dvs, 255],
                                                              "idle": records, "editing": editing}, indent=2))
        finally:
            menu.close()
    revision = subprocess.check_output(["git", "-C", str(vendor), "rev-parse", "HEAD"], text=True).strip()
    provenance = {"rom_sha1": ROM_SHA1, "source_revision": revision, "emulator": "PyBoy", "version": version,
                  "window": "null", "sound_emulated": False,
                  "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  "symbols_sha256": hashlib.sha256((vendor / "pokecrystal.sym").read_bytes()).hexdigest(),
                  "method": "initial party fixture, original NamingScreen and ordinary joypad input",
                  "source_pixels_supplied": False, "geothite_renderer_used": False}
    (output / "naming-oracle-provenance.json").write_text(json.dumps(provenance, indent=2))
    print(f"Captured source nickname references in {output}", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vendor", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--family", choices=["pc", "naming"], default="pc")
    args = parser.parse_args()
    capture(args.vendor.resolve(), args.output.resolve(), args.family)
