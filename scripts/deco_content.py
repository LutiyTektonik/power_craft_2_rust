#!/usr/bin/env python3
"""Generate Deco registration, recipes and block models from shared hull dimensions."""
from itertools import product
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "content"
ASSETS = ROOT / "assets/powercraft2"
DATA = ROOT / "data/powercraft2"
NAMESPACE = "powercraft2"
FACES = ("north", "east", "south", "west", "up", "down")
HORIZONTAL = FACES[:4]

PLATFORM_FLOOR = (0, 0, 0, 16, 1, 16)
PLATFORM_RAIL = (0, 0, 0, 16, 24, 1)
PLATFORM_RAIL_MODEL = [
    (0, 8, 0, 16, 11, 1),
    (0, 1, 0, 2, 8, 1),
    (14, 1, 0, 16, 8, 1),
]
STAIR_TREADS = [(0, 15, 0, 16, 16, 8), (0, 7, 8, 16, 8, 16)]
STAIR_COLLISION = [(0, 8, 0, 16, 16, 8), (0, 0, 8, 16, 8, 16)]
STAIR_RAIL = (0, 0, 0, 1, 28.8, 16)
STAIR_RAIL_MODEL = [
    (0, 22, 0, 1, 25, 8),
    (0, 14, 8, 1, 17, 16),
    (0, 16, 0, 1, 22, 2),
    (0, 8, 14, 1, 14, 16),
]


def write(path, data):
    path.parent.mkdir(exist_ok=True, parents=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def registry_id(name):
    return f"{NAMESPACE}:{name}"


def model_id(name):
    return f"{NAMESPACE}:block/{name}"


def model_file(name, **body):
    write(ASSETS / f"models/block/{name}.json", body)


def model_parts(name, pieces, surface="minecraft:block/iron_block", trim=None):
    trim = trim or surface
    model_file(
        name,
        textures={"particle": surface, "surface": surface, "trim": trim},
        elements=[
            {
                "from": list(box[:3]),
                "to": list(box[3:]),
                "faces": {face: {"texture": f"#{material}"} for face in FACES},
            }
            for box, material in pieces
        ],
    )


def register_block(name, *, template="minecraft:iron_block", resistance=10.0, **fields):
    properties = {
        "hardness": 1.5,
        "explosion_resistance": resistance,
        "requires_tool": True,
        "sounds": "minecraft:iron_block",
        "occludes": "shapes" not in fields,
    }
    write(DATA / f"horizon/block/{name}.json", {"template": template, "properties": properties, **fields, "item": True})
    write(
        DATA / f"loot_table/blocks/{name}.json",
        {
            "type": "minecraft:block",
            "pools": [
                {
                    "rolls": 1,
                    "entries": [
                        {
                            "type": "minecraft:item",
                            "name": registry_id(name),
                            "conditions": [{"condition": "minecraft:survives_explosion"}],
                        }
                    ],
                }
            ],
        },
    )
    write(
        ASSETS / f"items/{name}.json",
        {"model": {"type": "minecraft:model", "model": model_id(name)}},
    )


def shaped_recipe(name, pattern, ingredient, count):
    write(
        DATA / f"recipe/{name}.json",
        {
            "type": "minecraft:crafting_shaped",
            "category": "building",
            "pattern": pattern,
            "key": {"X": ingredient},
            "result": {"id": registry_id(name), "count": count},
        },
    )


def enum(values):
    return {"type": "enum", "values": values}


def mask_name(mask):
    return f"mask_{mask}"


def rotate(box, quarter_turns):
    x0, y0, z0, x1, y1, z1 = box
    for _ in range(quarter_turns):
        x0, z0, x1, z1 = 16 - z1, x0, 16 - z0, x1
    return x0, y0, z0, x1, y1, z1


def mirror_east(box):
    x0, y0, z0, x1, y1, z1 = box
    return 16 - x1, y0, z0, 16 - x0, y1, z1


def normalized(boxes):
    return [[coordinate / 16 for coordinate in box] for box in boxes]


def blockstates(name, **body):
    write(ASSETS / f"blockstates/{name}.json", body)


def apply_model(name, turns=0):
    return {"model": model_id(name), **({"y": 90 * turns} if turns else {})}


def platform():
    rails = [rotate(PLATFORM_RAIL, turn) for turn in range(4)]
    masks = [mask_name(mask) for mask in range(16)]
    shape_values = {
        f"{mask_name(mask)},{str(floor).lower()}": normalized(
            ([PLATFORM_FLOOR] if floor else [])
            + [rail for side, rail in enumerate(rails) if mask & (1 << side)],
        )
        for mask, floor in product(range(len(masks)), (False, True))
    }
    register_block(
        "platform",
        resistance=6.0,
        state_properties={"railings": enum(masks), "floor": {"type": "boolean"}},
        default_state={"railings": "mask_0", "floor": "true"},
        shapes={"default": normalized([PLATFORM_FLOOR])},
        collision_shapes={"properties": ["railings", "floor"], "values": shape_values},
    )
    model_parts("platform", [(PLATFORM_FLOOR, "surface")])
    model_parts("platform_rail", [(piece, "surface") for piece in PLATFORM_RAIL_MODEL])
    parts = [{"when": {"floor": "true"}, "apply": apply_model("platform")}]
    for mask in range(1, len(masks)):
        for direction in range(4):
            if mask & (1 << direction):
                parts.append({
                    "when": {"railings": mask_name(mask)},
                    "apply": apply_model("platform_rail", direction),
                })
    blockstates("platform", multipart=parts)
    shaped_recipe("platform", ["X  ", "X  ", "XXX"], "minecraft:iron_ingot", 5)


def stairs():
    masks = [mask_name(mask) for mask in range(4)]
    rails = [STAIR_RAIL, mirror_east(STAIR_RAIL)]
    shape_values = {
        f"{facing},{mask_name(mask)}": normalized(
            [rotate(box, turn) for box in STAIR_COLLISION]
            + [rotate(rail, turn) for side, rail in enumerate(rails) if mask & (1 << side)],
        )
        for turn, facing in enumerate(HORIZONTAL)
        for mask in range(len(masks))
    }
    register_block(
        "stairs",
        resistance=6.0,
        state_properties={"facing": enum(list(HORIZONTAL)), "railings": enum(masks)},
        default_state={"facing": "north", "railings": "mask_0"},
        shapes={"properties": ["facing", "railings"], "values": shape_values},
    )
    model_parts("stairs", [(piece, "surface") for piece in STAIR_TREADS])
    model_parts("stairs_rail_left", [(piece, "surface") for piece in STAIR_RAIL_MODEL])
    model_parts("stairs_rail_right", [(mirror_east(piece), "surface") for piece in STAIR_RAIL_MODEL])
    parts = []
    for turn, facing in enumerate(HORIZONTAL):
        parts.append({"when": {"facing": facing}, "apply": apply_model("stairs", turn)})
        for mask in range(1, len(masks)):
            for side, rail in enumerate(("stairs_rail_left", "stairs_rail_right")):
                if mask & (1 << side):
                    parts.append({
                        "when": {"facing": facing, "railings": mask_name(mask)},
                        "apply": apply_model(rail, turn),
                    })
    blockstates("stairs", multipart=parts)
    shaped_recipe("stairs", ["X  ", "XX ", " XX"], "minecraft:iron_ingot", 4)


def redstone_storage():
    register_block("redstone_storage")
    model_file("redstone_storage", parent="minecraft:block/cube_all", textures={"all": "minecraft:block/redstone_block"})
    blockstates("redstone_storage", variants={"": apply_model("redstone_storage")})


def deco_tab(names):
    write(
        DATA / "horizon/creative_tab/deco.json",
        {"title": "itemGroup.powercraft2.deco", "icon": registry_id("platform"), "items": [registry_id(name) for name in names]},
    )


def main():
    redstone_storage()
    platform()
    stairs()
    deco_tab(["redstone_storage", "platform", "stairs"])


if __name__ == "__main__":
    main()
