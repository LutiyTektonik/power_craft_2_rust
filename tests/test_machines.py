import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1] / "content"
ASSETS = ROOT / "assets/powercraft2"
DATA = ROOT / "data/powercraft2"


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


class ExperienceBankContent(unittest.TestCase):
    def test_published_recipe_and_bank_loot(self):
        bank = "powercraft2:xp_bank"
        recipe = read(DATA / "recipe/xp_bank.json")
        self.assertEqual(recipe["pattern"], ["ODO", "OGO", "O O"])
        self.assertEqual(recipe["key"], {
            "O": "minecraft:obsidian",
            "D": "minecraft:diamond_block",
            "G": "minecraft:ghast_tear",
        })
        self.assertEqual(recipe["result"], {"id": bank, "count": 1})
        loot = read(DATA / "loot_table/blocks/xp_bank.json")
        entry = loot["pools"][0]["entries"][0]
        self.assertEqual(entry["name"], bank)
        self.assertEqual(entry["functions"], [{"function": "powercraft2:retain_bank_points"}])

    def test_menu_transport_and_shape_match_the_content_registration(self):
        bank = read(DATA / "horizon/block/xp_bank.json")
        self.assertTrue(bank["item"])
        self.assertEqual(bank["properties"]["light"], 7)
        self.assertEqual(bank["shapes"], bank["collision_shapes"])
        self.assertEqual(read(DATA / "horizon/menu/xp_bank.json"), {"slots": 1, "data_slots": 2})
        self.assertIn("powercraft2:xp_bank", read(DATA / "horizon/creative_tab/machines.json")["items"])
        self.assertEqual(read(ASSETS / "items/xp_bank.json")["model"]["model"], "powercraft2:block/xp_bank")
        self.assertEqual(read(ASSETS / "blockstates/xp_bank.json")["variants"][""]["model"], "powercraft2:block/xp_bank")
        model = read(ASSETS / "models/block/xp_bank.json")
        for actual, expected in zip(model["elements"], bank["shapes"]["default"], strict=True):
            self.assertEqual([p / 16 for p in actual["from"] + actual["to"]], expected)


if __name__ == "__main__":
    unittest.main()
