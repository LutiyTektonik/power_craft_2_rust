"""Check the original recipes, crystal variants and worldgen links in Core."""

import json
from pathlib import Path
import unittest


CONTENT = Path(__file__).resolve().parents[1] / "content"
DATA = CONTENT / "data/powercraft2"
ASSETS = CONTENT / "assets/powercraft2"
COLORS = ("orange", "red", "green", "blue", "white", "magenta", "cyan", "yellow")
CRYSTALS = tuple(f"power_crystal_{color}" for color in COLORS)


def read(root, path):
    return json.loads((root / f"{path}.json").read_text(encoding="utf-8"))


class CoreContentTest(unittest.TestCase):
    def test_core_recipes_and_red_cyan_crystal_requirements(self):
        recipes = {
            "annihilation_core": (["ILI", "GPG", "RRR"], {
                "I": "minecraft:iron_ingot", "L": "minecraft:glass_pane",
                "G": "minecraft:gold_ingot", "P": "minecraft:diamond_pickaxe",
                "R": "minecraft:redstone",
            }),
            "formation_core": (["ILI", "GDG", "RRR"], {
                "I": "minecraft:iron_ingot", "L": "minecraft:glass_pane",
                "G": "minecraft:gold_ingot", "D": "minecraft:diamond",
                "R": "minecraft:redstone",
            }),
            "crafting_core": (["TRT", "EXE", "TRT"], {
                "T": "minecraft:crafting_table", "R": "minecraft:redstone",
                "E": "minecraft:ender_pearl", "X": "powercraft2:power_crystal_red",
            }),
            "sensor_core": (["RGE", "IXI", "EGR"], {
                "R": "minecraft:redstone", "G": "minecraft:gold_ingot",
                "E": "minecraft:ender_pearl", "I": "minecraft:iron_ingot",
                "X": "powercraft2:power_crystal_cyan",
            }),
        }
        for item, (pattern, ingredients) in recipes.items():
            with self.subTest(item=item):
                recipe = read(DATA, f"recipe/{item}")
                self.assertEqual(recipe["type"], "minecraft:crafting_shaped")
                self.assertEqual(recipe["pattern"], pattern)
                self.assertEqual(recipe["key"], ingredients)
                self.assertEqual(recipe["result"], {"id": f"powercraft2:{item}", "count": 1})
                self.assertEqual(read(DATA, f"horizon/item/{item}")["max_stack_size"], 1)

    def test_every_variant_drops_its_color_and_makes_four_dust(self):
        ids = [f"powercraft2:{crystal}" for crystal in CRYSTALS]
        self.assertEqual(read(DATA, "tags/item/power_crystals")["values"], ids)
        dust = read(DATA, "recipe/power_dust")
        self.assertEqual(dust["type"], "minecraft:crafting_shapeless")
        self.assertEqual(dust["ingredients"], ["#powercraft2:power_crystals"])
        self.assertEqual(dust["result"], {"id": "powercraft2:power_dust", "count": 4})
        fuel = read(DATA, "horizon/item/power_dust")["components"]["minecraft:cooking_fuel"]
        self.assertEqual(fuel, {"burn_time": 3600, "speed_multiplier": 1.0})

        tab = read(DATA, "horizon/creative_tab/core")
        self.assertEqual(len(tab["items"]), 13)
        self.assertEqual(tab["items"][:len(CRYSTALS)], ids)
        for crystal in CRYSTALS:
            with self.subTest(crystal=crystal):
                block = read(DATA, f"horizon/block/{crystal}")
                self.assertEqual(block["components"]["minecraft:rarity"], "rare")
                self.assertTrue(block["components"]["minecraft:enchantment_glint_override"])
                pool, = read(DATA, f"loot_table/blocks/{crystal}")["pools"]
                self.assertEqual(pool["rolls"], 1)
                self.assertEqual(pool["entries"], [{
                    "type": "minecraft:item", "name": f"powercraft2:{crystal}",
                }])
                model = read(ASSETS, f"items/{crystal}")["model"]["model"]
                self.assertEqual(model, f"powercraft2:block/{crystal}")
                variant = read(ASSETS, f"blockstates/{crystal}")["variants"][""]
                self.assertEqual(variant["model"], model)

    def test_worldgen_selects_one_color_then_tries_twelve_stone_deposits(self):
        feature = "powercraft2:power_crystal_chunk"
        self.assertEqual(read(DATA, "neoforge/biome_modifier/add_power_crystal"), {
            "type": "neoforge:add_features", "biomes": "#minecraft:is_overworld",
            "features": feature, "step": "underground_ores",
        })
        self.assertEqual(read(DATA, "worldgen/placed_feature/power_crystal_chunk"), {
            "feature": feature, "placement": [{"type": "minecraft:biome"}],
        })
        choose = read(DATA, "worldgen/feature/power_crystal_chunk")
        self.assertEqual(choose["type"], "minecraft:simple_random_selector")
        self.assertEqual(len(choose["features"]), len(CRYSTALS))
        for crystal, selected in zip(CRYSTALS, choose["features"]):
            with self.subTest(crystal=crystal):
                vein = f"{crystal}_vein"
                self.assertEqual(selected, {
                    "feature": f"powercraft2:{vein}",
                    "placement": [
                        {"type": "minecraft:count", "count": 12},
                        {"type": "minecraft:in_square"},
                        {"type": "minecraft:height_range", "height": {
                            "type": "minecraft:uniform",
                            "min_inclusive": {"absolute": 5},
                            "max_inclusive": {"absolute": 14},
                        }},
                    ],
                })
                self.assertEqual(read(DATA, f"worldgen/feature/{vein}"), {
                    "type": "minecraft:ore",
                    "size": 4,
                    "discard_chance_on_air_exposure": 0.0,
                    "targets": [{
                        "target": {"predicate_type": "minecraft:block_match", "block": "minecraft:stone"},
                        "state": f"powercraft2:{crystal}",
                    }],
                })
