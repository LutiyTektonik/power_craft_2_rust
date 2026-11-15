import json
from pathlib import Path
import unittest

CONTENT = Path(__file__).resolve().parents[1] / "content"
DATA = CONTENT / "data/powercraft2"
ASSETS = CONTENT / "assets/powercraft2"


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


class DecoTest(unittest.TestCase):
    def test_published_deco_recipes(self):
        for name, rows, count in (
            ("platform", ["X  ", "X  ", "XXX"], 5),
            ("stairs", ["X  ", "XX ", " XX"], 4),
        ):
            with self.subTest(name=name):
                recipe = read(DATA / f"recipe/{name}.json")
                self.assertEqual(recipe["pattern"], rows)
                self.assertEqual(recipe["key"], {"X": "minecraft:iron_ingot"})
                self.assertEqual(recipe["result"], {"id": f"powercraft2:{name}", "count": count})

    def test_redstone_storage_does_not_clone_vanilla_power(self):
        block = read(DATA / "horizon/block/redstone_storage.json")
        self.assertEqual(block["template"], "minecraft:iron_block")
        self.assertFalse((DATA / "recipe/redstone_storage.json").exists())

    def test_platform_sides_and_ladder_opening(self):
        definition = read(DATA / "horizon/block/platform.json")
        shapes = definition["collision_shapes"]["values"]
        self.assertEqual(len(shapes), 32)
        self.assertEqual(shapes["mask_0,true"], [[0, 0, 0, 1, 0.0625, 1]])
        self.assertEqual(shapes["mask_0,false"], [])
        self.assertEqual(shapes["mask_1,false"], [[0, 0, 0, 1, 1.5, 0.0625]])
        self.assertEqual(shapes["mask_2,false"], [[0.9375, 0, 0, 1, 1.5, 1]])
        self.assertEqual(shapes["mask_4,false"], [[0, 0, 0.9375, 1, 1.5, 1]])
        self.assertEqual(shapes["mask_8,false"], [[0, 0, 0, 0.0625, 1.5, 1]])

    def test_stairs_rotate_and_only_guard_lateral_edges(self):
        states = read(DATA / "horizon/block/stairs.json")
        shapes = states["shapes"]["values"]
        self.assertEqual(len(shapes), 16)
        self.assertEqual(shapes["north,mask_0"], [[0, 0.5, 0, 1, 1, 0.5], [0, 0, 0.5, 1, 0.5, 1]])
        self.assertEqual(shapes["east,mask_0"][0], [0.5, 0.5, 0, 1, 1, 1])
        self.assertEqual(shapes["south,mask_0"][0], [0, 0.5, 0.5, 1, 1, 1])
        self.assertEqual(shapes["west,mask_0"][0], [0, 0.5, 0, 0.5, 1, 1])
        for facing in ("north", "east", "south", "west"):
            with self.subTest(facing=facing):
                self.assertEqual(len(shapes[f"{facing},mask_1"]), 3)
                self.assertEqual(len(shapes[f"{facing},mask_2"]), 3)
                self.assertEqual(len(shapes[f"{facing},mask_3"]), 4)

    def test_creative_tab_entries_have_models_and_loot(self):
        tab = read(DATA / "horizon/creative_tab/deco.json")
        self.assertIn("powercraft2:platform", tab["items"])
        self.assertIn("powercraft2:stairs", tab["items"])
        self.assertIn("powercraft2:redstone_storage", tab["items"])
        for block in tab["items"]:
            with self.subTest(block=block):
                name = block.removeprefix("powercraft2:")
                item = read(ASSETS / f"items/{name}.json")
                self.assertEqual(item["model"]["model"], f"powercraft2:block/{name}")
                self.assertTrue((ASSETS / f"models/block/{name}.json").exists())
                self.assertTrue((ASSETS / f"blockstates/{name}.json").exists())
                loot = read(DATA / f"loot_table/blocks/{name}.json")
                self.assertEqual(loot["pools"][0]["entries"][0]["name"], block)
