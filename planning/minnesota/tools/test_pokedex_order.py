"""Regressions for family ordering and publication drift in the regional Pokédex."""
import copy
import unittest
from pathlib import Path

from audit_pokedex_roster import audit_published_order, read, regional_order


ROOT = Path(__file__).resolve().parents[1] / "pokemon"


class PokedexOrderTests(unittest.TestCase):
    def setUp(self):
        self.roster = read(ROOT / "roster.json")
        self.order = read(ROOT / "pokedex-order.json")
        retired = {family["id"] for family in read(ROOT / "roster-curation.json")["retired_families"]}
        active = {member["id"] for family in self.roster["families"] if family["id"] not in retired
                  for member in family["members"]}
        self.active_order = [species_id for species_id in regional_order(self.roster, self.order)
                             if species_id in active]
        self.catalog = {"species": [
            {"identity": {"id": species_id, "regional_number": number}}
            for number, species_id in enumerate(self.active_order, 1)
        ]}

    def test_active_starters_and_final_families(self):
        self.assertEqual(self.active_order[:9], [
            "MN_001", "MN_002", "MN_003", "MN_274", "MN_275", "MN_276",
            "MN_010", "MN_011", "MN_012",
        ])
        self.assertEqual(self.active_order[-8:], [
            "MN_151", "MN_152", "MN_153", "MN_095", "MN_096", "MN_319", "MN_098", "MN_099",
        ])
        self.assertEqual(len(self.active_order), 225)
        audit_published_order(self.catalog, self.active_order)

    def test_dense_source_numbers_cannot_disagree_with_sections(self):
        species = self.order["species"]
        legendary = next(entry for entry in species if entry["id"] == "MN_095")
        species.remove(legendary)
        replacement = next(i for i, entry in enumerate(species) if entry["id"] == "MN_320")
        species.insert(replacement, legendary)
        for number, entry in enumerate(species, 1):
            entry["number"] = number
        with self.assertRaisesRegex(AssertionError, "Species numbers must follow"):
            regional_order(self.roster, self.order)

    def test_sections_cannot_move_legendaries_before_ordinary_families(self):
        sections = self.order["sections"]
        legendary = next(section for section in sections if section["id"] == "legendary")
        sections.remove(legendary)
        replacement = next(i for i, section in enumerate(sections)
                           if section["id"] == "new_regional_families")
        sections.insert(replacement, legendary)
        with self.assertRaisesRegex(AssertionError, "families must stay in order"):
            regional_order(self.roster, self.order)

    def test_sections_cannot_repeat_or_omit_a_family(self):
        order = copy.deepcopy(self.order)
        order["sections"][1]["families"].append("MN_F001")
        with self.assertRaisesRegex(AssertionError, "multiple order sections"):
            regional_order(self.roster, order)
        self.order["sections"][1]["families"].pop()
        with self.assertRaisesRegex(AssertionError, "cover every source family"):
            regional_order(self.roster, self.order)

    def test_starter_and_evolution_orders_are_validated(self):
        order = copy.deepcopy(self.order)
        order["sections"][0]["families"].reverse()
        with self.assertRaisesRegex(AssertionError, "Grass, Fire, Water"):
            regional_order(self.roster, order)
        self.roster["families"][0]["members"].reverse()
        with self.assertRaisesRegex(AssertionError, "Evolution stages"):
            regional_order(self.roster, self.order)

    def test_dense_published_numbers_cannot_hide_the_old_legendary_order(self):
        species = self.catalog["species"]
        species[186:] = species[-5:] + species[186:-5]
        for number, entry in enumerate(species, 1):
            entry["identity"]["regional_number"] = number
        with self.assertRaisesRegex(AssertionError, "Published species sequence"):
            audit_published_order(self.catalog, self.active_order)

    def test_correct_published_sequence_cannot_keep_stale_numbers(self):
        species = self.catalog["species"]
        species[186]["identity"]["regional_number"], species[-1]["identity"]["regional_number"] = (
            species[-1]["identity"]["regional_number"], species[186]["identity"]["regional_number"]
        )
        with self.assertRaisesRegex(AssertionError, "Published species numbers"):
            audit_published_order(self.catalog, self.active_order)


if __name__ == "__main__":
    unittest.main()
