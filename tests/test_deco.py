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
            ("iron_frame", ["XXX", "X X", "XXX"], 8),
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

    def test_iron_frame_covers_five_fixture_faces_and_keeps_items_passable(self):
        definition = read(DATA / "horizon/block/iron_frame.json")
        state = read(ASSETS / "blockstates/iron_frame.json")
        panel_keys = {f"panel_{side}" for side in ("north", "east", "south", "west", "up")}
        self.assertEqual(set(definition["state_properties"]), panel_keys)
        self.assertEqual(
            {next(iter(part["when"])) for part in state["multipart"] if "when" in part},
            panel_keys,
        )
        self.assertEqual(definition["collision_shapes"], {"default": [[0, 0, 0, 1, 1, 1]]})
        self.assertEqual(definition["collision_context_shapes"], {"item": {"default": []}})

    def test_chimney_variants_keep_legacy_materials_and_own_item_drops(self):
        ingredients = {
            "cobblestone": "cobblestone",
            "brick": "bricks",
            "stone_brick": "stone_bricks",
            "cracked_stone_brick": "cracked_stone_bricks",
            "chiseled_stone_brick": "chiseled_stone_bricks",
            "sandstone": "sandstone",
            "chiseled_sandstone": "chiseled_sandstone",
            "cut_sandstone": "cut_sandstone",
            "nether_brick": "nether_bricks",
            "quartz": "quartz_block",
            "chiseled_quartz": "chiseled_quartz_block",
            "quartz_pillar": "quartz_pillar",
            "mossy_cobblestone": "mossy_cobblestone",
            "mossy_stone_brick": "mossy_stone_bricks",
            "clay": "clay",
            "iron": "iron_block",
        }
        paths = sorted((DATA / "recipe").glob("chimney_*.json"))
        self.assertEqual(len(paths), len(ingredients))
        for path in paths:
            with self.subTest(name=path.stem):
                name = path.stem
                recipe = read(path)
                self.assertEqual(recipe["pattern"], ["X X", "X X", "X X"])
                self.assertEqual(recipe["key"], {"X": f"minecraft:{ingredients[name.removeprefix('chimney_')]}"})
                self.assertEqual(recipe["result"], {"id": f"powercraft2:{name}", "count": 6})
                definition = read(DATA / f"horizon/block/{name}.json")
                self.assertEqual(len(definition["shapes"]["default"]), 4)
                self.assertEqual(definition["collision_context_shapes"], {"item": {"default": []}})
                model = read(ASSETS / f"models/block/{name}.json")
                self.assertEqual(model["parent"], "powercraft2:block/chimney_shell")

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
