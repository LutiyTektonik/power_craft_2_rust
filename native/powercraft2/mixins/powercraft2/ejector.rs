use super::{Game, powercraft2_transport as transport};
use horizon_client::mod_api::{Host, ModBlockEntity, RandomSource, Tag};
use horizon_core::geometry::Direction as Face;
use transport::{Direction, EJECTOR_DEFAULTS, EJECTOR_LIMITS};

const KEYS: [&str; 4] = ["mode", "order", "stacks", "items"];

fn settings(entity: &ModBlockEntity) -> [i32; 4] {
    std::array::from_fn(|i| match entity.nbt.get(&KEYS[i].into()) {
        Some(Tag::Int(value)) => (*value).clamp(0, EJECTOR_LIMITS[i]),
        _ => EJECTOR_DEFAULTS[i],
    })
}

pub(super) fn button(entity: &mut ModBlockEntity, id: i32) {
    let Ok(index) = usize::try_from(id / 4) else {
        return;
    };
    if id < 0 || index >= KEYS.len() {
        return;
    }
    let delta = [-1, 1, -10, 10][(id % 4) as usize];
    let mut values = settings(entity);
    let value = values[index] + delta;
    values[index] = if index < 2 {
        value.rem_euclid(EJECTOR_LIMITS[index] + 1)
    } else {
        value.clamp(0, EJECTOR_LIMITS[index])
    };
    entity
        .nbt
        .insert(KEYS[index].into(), Tag::Int(values[index]));
    entity.data = values.to_vec();
}

pub(super) fn tick(game: &mut Game<'_>, block: [i32; 3], entity: &mut ModBlockEntity) {
    let values = settings(entity);
    if entity.data.as_slice() != values {
        entity.data = values.to_vec();
    }
    let host: &mut dyn Host = game;
    if !host.on(transport::MOD_ID) {
        return;
    }
    let dimension = host.dimension();
    let Some(level) = host.level(dimension) else {
        return;
    };
    if level.block(host, block).as_deref() != Some(transport::EJECTOR_CONVEYOR) {
        return;
    }
    let powered = host.neighbor_signal(dimension, block) > 0;
    let state = if powered { "true" } else { "false" };
    if level.block_property(host, block, "powered").as_deref() == Some(state)
        || !host.set_block_property_with(block, "powered", state, 2)
        || !powered
    {
        return;
    }
    let Some(facing) = level.block_property(host, block, "facing") else {
        return;
    };
    let Ok(facing) = Direction::try_from(facing.as_str()) else {
        return;
    };
    for side in transport::ejector_sources(facing) {
        let source = side.neighbor(block);
        if !level.is_loaded(game, source)
            || !level.block(game, source).is_some_and(|name| {
                transport::supports_storage(&name) || name == "minecraft:barrel"
            })
        {
            continue;
        }
        let face = match side {
            Direction::North => Face::South,
            Direction::East => Face::West,
            Direction::South => Face::North,
            Direction::West => Face::East,
        };
        let Some(slots) = game.native_container_slots(dimension, source, face) else {
            continue;
        };
        let plan = transport::ejector_plan(
            slots
                .iter()
                .filter_map(|slot| {
                    if slot.can_take {
                        slot.stack.as_ref().map(|stack| (slot.slot, stack.count))
                    } else {
                        None
                    }
                })
                .collect(),
            values,
            |bound| game.world.random.next_int_bound(bound as i32).unwrap_or(0) as usize,
        );
        if plan.is_empty() {
            continue;
        }
        let [dx, _, dz] = side.vector();
        // Keep the item's collision box clear of the source block.
        let at = [
            f64::from(block[0]) + 0.5 + 0.35 * f64::from(dx),
            f64::from(block[1]) + transport::BELT_HEIGHT + 0.05,
            f64::from(block[2]) + 0.5 + 0.35 * f64::from(dz),
        ];
        let mut moved = false;
        for (slot, count) in plan {
            let Some(stack) = slots
                .iter()
                .find(|entry| entry.slot == slot)
                .and_then(|entry| entry.stack.as_ref())
            else {
                continue;
            };
            let mut output = stack.clone();
            output.count = count;
            let Some(id) = game.next_entity.checked_add(1) else {
                return;
            };
            game.spawn_item(output.clone(), at, [0.0; 3], 7);
            let Some(index) = game.dropped.iter().position(|item| item.id == id) else {
                continue;
            };
            // Join hooks may refuse or change a drop. Only debit the source
            // after the accepted drop and the planned source are both intact.
            let unchanged = game.dropped[index].stack == output
                && game
                    .native_container_slots(dimension, source, face)
                    .is_some_and(|current| {
                        current.iter().any(|entry| {
                            entry.slot == slot
                                && entry.can_take
                                && entry.stack.as_ref() == Some(stack)
                        })
                    });
            if !unchanged {
                game.remove_dropped(index);
                continue;
            }
            let Some(taken) = game.native_extract_item(dimension, source, face, slot, count) else {
                game.remove_dropped(index);
                continue;
            };
            // The audited vanilla extraction only shrinks the source slot.
            debug_assert_eq!(taken, output);
            moved = true;
        }
        if moved {
            break;
        }
    }
}

const BUTTONS: [f32; 4] = [132.0, 148.0, 164.0, 180.0];

pub(super) fn hit([x, y]: [f32; 2], right: bool) -> Option<i32> {
    for row in 0..4 {
        let top = 24.0 + 21.0 * row as f32;
        if y < top || y >= top + 16.0 {
            continue;
        }
        for (column, left) in BUTTONS.into_iter().enumerate() {
            if (left..left + 14.0).contains(&x) {
                let delta = [2, 0, 1, 3][column];
                if row < 2 && (column == 0 || column == 3) {
                    return None;
                }
                return Some(
                    4 * row
                        + if right && row >= 2 {
                            if column < 2 { 2 } else { 3 }
                        } else {
                            delta
                        },
                );
            }
        }
    }
    None
}

pub(super) fn draw(
    game: &Game<'_>,
    data: &[i32],
    batch: &mut horizon_client::mod_api::HudBatch,
    [x, y]: [f32; 2],
) {
    let [mode, order, stacks, items] = data.try_into().unwrap_or(EJECTOR_DEFAULTS);
    let mode = match mode {
        1 => game.translate("powercraft2.ejector.items", "Items"),
        2 => game.translate("powercraft2.ejector.all", "All"),
        _ => game.translate("powercraft2.ejector.stacks", "Stacks"),
    };
    let order = match order {
        1 => game.translate("powercraft2.ejector.last", "Last"),
        2 => game.translate("powercraft2.ejector.random", "Random"),
        _ => game.translate("powercraft2.ejector.first", "First"),
    };
    let lines = [
        (game.translate("powercraft2.ejector.mode", "Mode"), mode),
        (game.translate("powercraft2.ejector.order", "Order"), order),
        (
            game.translate("powercraft2.ejector.stacks", "Stacks"),
            stacks.to_string(),
        ),
        (
            game.translate("powercraft2.ejector.items", "Items"),
            items.to_string(),
        ),
    ];
    for (row, (label, value)) in lines.into_iter().enumerate() {
        let top = y + 24.0 + 21.0 * row as f32;
        game.native_text(batch, &label, [x + 12.0, top + 4.0], 0x303030, false);
        game.native_text(batch, &value, [x + 73.0, top + 4.0], 0x303030, false);
        for (column, (left, label)) in BUTTONS.into_iter().zip(["«", "−", "+", "»"]).enumerate()
        {
            if row < 2 && (column == 0 || column == 3) {
                continue;
            }
            let left = x + left;
            game.native_fill(batch, [left, top, left + 14.0, top + 16.0], 0xff535865);
            game.native_text(batch, label, [left + 3.0, top + 4.0], 0xffffff, false);
        }
    }
}
