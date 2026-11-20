#!/usr/bin/env python3
"""Check the public approaches and actual counter staff in a written Fair Table pack."""
import argparse
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
LAKESIDE = runpy.run_path(str(HERE.parent / "lakeside-stops-v01" / "check-pack.py"))
accessible, terrain, xy = (LAKESIDE[key] for key in ("accessible", "terrain", "xy"))


def check_fair(data):
    room = data["maps"]["MnStateFairgrounds"]
    stalls = json.loads((HERE.parents[1] / "zones" / "state-fairgrounds.json").read_text())
    vendors = [r for r in stalls["native_map"]["residents"] if r.get("service") == "pokemart"]
    assert len(vendors) == len(room["script_shop_commands"]) == 32
    # The entire crowd remains in place. Each path stays off the encounter grass.
    reached = accessible(data, room, (95, 64))
    actual_food = set()
    food_stalls = 0
    for vendor in vendors:
        name = vendor["id"]
        actor, = [a for a in room["objects"] if a["script"] == name]
        shop, = [s for s in room["script_shop_commands"] if s["source_script"] == name]
        assert room["scripts"][name][shop["command_index"]] == {
            "command": "pokemart", "args": ["MARTTYPE_STANDARD", shop["mart_id"]]
        }
        stock = data["marts"][shop["mart_id"]]
        assert stock == vendor["stock"] and all(data["items"][i]["price"] > 0 for i in stock), name
        at, counter, body = (tuple(vendor[key]) for key in ("approach", "counter_tile", "tile"))
        assert at in reached and xy(actor) == body
        assert (at[0], at[1] - 1) == counter and (at[0], at[1] - 2) == body
        assert terrain(data, room, at) == terrain(data, room, body) == "FLOOR"
        assert terrain(data, room, counter) == "COUNTER"
        foods = {i for i in stock if i.startswith("FAIR_")}
        actual_food.update(foods)
        food_stalls += bool(foods)
    assert food_stalls == 13 and actual_food == set(stalls["food_contract"]["purchasable_items"])
    assert len(actual_food) == 16
    return len(reached)


def check_city_and_lakeshore(data):
    cases = [
        ("MnStPaul", "MnStPaulWestSeventhBakery", (3, 3), (2, 3), "MN_SPX_WESTSEVENTHBAKERY_HOST",
         {"MN_BUTTER_ROLL", "MN_CARDAMOM_BUN"}),
        ("MnDuluth", "MnDuluthCommunityKitchen", (5, 6), None, "MN_DULUTH_X_COMMUNITYKITCHEN_F1_HOST",
         {"MN_WILD_RICE_SOUP"}),
        ("MnR11C00R01", "MnNorthShorePieShop", (5, 3), (6, 3), "MN_SHORE_BAKER", {"MN_BLUEBERRY_SLICE"}),
    ]
    for street_id, room_id, customer, counter, clerk, expected_food in cases:
        street, room = data["maps"][street_id], data["maps"][room_id]
        door, = [w for w in street["events"]["warps"]
                 if w["target_map_constant"] == room["attributes"]["map_constant"] and terrain(data, street, xy(w)) == "DOOR"]
        landing = room["events"]["warps"][door["target_warp_id"] - 1]
        reached = accessible(data, room, xy(landing))
        assert terrain(data, room, xy(landing)) == "FLOOR" and customer in reached
        actor, = [a for a in room["objects"] if a["script"] == clerk]
        body = xy(actor)
        assert terrain(data, room, body) == "FLOOR"
        if counter:
            assert terrain(data, room, counter) == "COUNTER"
            assert all(a + b == 2 * c for a, b, c in zip(customer, body, counter))
        else:
            assert sum(abs(a-b) for a, b in zip(customer, body)) == 1
        if room_id == "MnNorthShorePieShop":
            sold = {r["item_id"] for r in room["script_item_grants"] if r["source_script"] == clerk}
        else:
            shop, = [s for s in room["script_shop_commands"] if s["source_script"] == clerk]
            sold = set(data["marts"][shop["mart_id"]])
        assert expected_food <= sold, room_id


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    path = parser.parse_args().pack
    data = LAKESIDE["read_pack"](path)["data"]
    floor = check_fair(data)
    check_city_and_lakeshore(data)
    print(f"{path.name}: 32 reachable stalls, 13 specialty sellers, {floor} connected floor squares and 3 public food rooms")
