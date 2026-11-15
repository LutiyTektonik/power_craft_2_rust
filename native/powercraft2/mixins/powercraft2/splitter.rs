use super::{Game, powercraft2_separator as parts, powercraft2_transport as transport};
use horizon_client::{
    containers::BlockEntity,
    mod_api::{AttachmentType, Holder, Host},
};
use horizon_core::geometry::Vec3;
use horizon_world::{
    attachments::{AttachmentCodec, AttachmentError},
    nbt::{Compound, Tag},
};
use transport::{Belt, Direction, Elevator, SPLITTER, SPLITTER_EXITS};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Route {
    block: [i32; 3],
    pending: [u32; 6],
}

static ROUTE: AttachmentType<Option<Route>> = AttachmentType {
    namespace: "powercraft2",
    name: "splitter_route",
    default: || None,
    codec: Some(AttachmentCodec { encode, decode }),
    copy_on_death: false,
    sync: false,
};

fn encode(route: &Option<Route>) -> Result<Compound, AttachmentError> {
    let Some(route) = route else {
        return Ok(Compound::new());
    };
    let values = route
        .block
        .into_iter()
        .map(i64::from)
        .chain(route.pending.into_iter().map(i64::from))
        .collect();
    Ok([("route_v1".into(), Tag::LongArray(values))].into())
}

fn decode(data: &Compound) -> Result<Option<Route>, AttachmentError> {
    let Some(value) = data.get(&"route_v1".into()) else {
        return Ok(None);
    };
    let Tag::LongArray(values) = value else {
        return Err(AttachmentError::Decode);
    };
    let [x, y, z, down, up, north, south, west, east] = values.as_slice() else {
        return Err(AttachmentError::Decode);
    };
    let mut block = [0; 3];
    for (coordinate, value) in block.iter_mut().zip([x, y, z]) {
        *coordinate = i32::try_from(*value).map_err(|_| AttachmentError::Decode)?;
    }
    let mut pending = [0; 6];
    for (count, value) in pending.iter_mut().zip([down, up, north, south, west, east]) {
        *count = u32::try_from(*value).map_err(|_| AttachmentError::Decode)?;
    }
    Ok(Some(Route { block, pending }))
}

pub(super) fn clear_route(game: &mut Game<'_>, id: u32) {
    if let Some(map) = game
        .persist
        .uuids
        .get(&id)
        .and_then(|uuid| game.entity_attachments.get_mut(uuid))
    {
        let _previous = map.remove(&ROUTE);
    }
}

pub(super) fn tick(game: &mut Game<'_>, id: u32, block: [i32; 3], position: [f64; 3]) {
    let Ok((centering, ready)) = transport::splitter_centering(block, position) else {
        return;
    };
    let Some(index) = game.dropped.iter().position(|item| item.id == id) else {
        return;
    };
    let item = &mut game.dropped[index];
    item.velocity = Vec3::new(centering[0], centering[1], centering[2]);
    if !ready {
        return;
    }
    let source_count = item.stack.count;
    let holder = Holder::Entity(id);
    let host: &mut dyn Host = game;
    let mut route = if let Ok(Some(Some(route))) = ROUTE.get_existing(host, holder)
        && route.block == block
        && route.pending.iter().copied().map(u64::from).sum::<u64>() == u64::from(source_count)
    {
        route
    } else {
        let source = &game.dropped[index].stack;
        let [x, y, z] = block;
        let Some(BlockEntity::Mod(filters)) = game.world.block_entities.get(&(x, y, z)) else {
            return;
        };
        if filters.id != SPLITTER {
            return;
        }
        let pending = transport::splitter_amounts(
            source_count,
            filters.items.iter().map(|slot| {
                slot.as_ref()
                    .filter(|sample| sample.item == source.item && sample.damage == source.damage)
                    .map_or(0, |sample| sample.count)
            }),
        );
        let route = Route { block, pending };
        // Save allocations before dispatch so a blocked share can survive a reload.
        let host: &mut dyn Host = game;
        if ROUTE.set(host, holder, Some(route)).is_err() {
            return;
        }
        route
    };
    let before = route;

    let mut remaining = source_count;
    let mut transfer_budget = 64;
    for ((offset, face), pending) in SPLITTER_EXITS.into_iter().zip(&mut route.pending) {
        let amount = *pending;
        if amount == 0 {
            continue;
        }
        let target = std::array::from_fn(|axis| block[axis] + offset[axis]);
        let [x, y, z] = target;
        if !(game.world.shape.min_y..game.world.shape.max_y()).contains(&y)
            || !game.world.loaded(x, z)
        {
            continue;
        }
        let Some(state) = game.pack.assets.registry.state(game.world.block(x, y, z)) else {
            continue;
        };
        let offered = amount.min(transfer_budget);
        if transport::supports_storage(&state.name) {
            if offered == 0 {
                continue;
            }
            let Some(index) = game.dropped.iter().position(|item| item.id == id) else {
                return;
            };
            let stack = horizon_client::items::ItemStack {
                count: offered,
                ..game.dropped[index].stack.clone()
            };
            let host: &mut dyn Host = game;
            let returned = host
                .insert_item(
                    target,
                    super::powercraft2_elevator::insertion_face(face),
                    stack,
                )
                .map_or(0, |stack| stack.count);
            let Some(accepted) = offered.checked_sub(returned) else {
                log::error!("PowerCraft: splitter received an invalid container remainder");
                break;
            };
            if accepted == 0 {
                continue;
            }
            let Some(index) = game.dropped.iter().position(|item| item.id == id) else {
                return;
            };
            remaining -= accepted;
            if remaining == 0 {
                game.remove_dropped(index);
                return;
            }
            let source = &mut game.dropped[index];
            source.stack.count = remaining;
            source.needs_sync = true;
            *pending -= accepted;
            transfer_budget -= accepted;
            continue;
        }
        let vertical = offset[1];
        let belt = vertical == 0
            && Belt::from_block(&state.name).is_some()
            && state
                .properties
                .get("facing")
                .and_then(|name| Direction::try_from(name.as_str()).ok())
                .is_some_and(|facing| facing.neighbor(target) != block);
        let aligned_elevator = matches!(
            (vertical, Elevator::from_block(&state.name)),
            (1, Some(Elevator::Up)) | (-1, Some(Elevator::Down))
        );
        if !(matches!(
            state.name.as_str(),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ) || (vertical == 0 && belt)
            || aligned_elevator
            || state.name == SPLITTER)
        {
            continue;
        }
        let (at, velocity) = transport::splitter_exit_motion(target, offset, belt);
        if transport::occupied_block(at) != Ok(target) {
            continue;
        }
        if amount == remaining {
            let Some(source) = game.dropped.iter_mut().find(|item| item.id == id) else {
                return;
            };
            source.position = Vec3::new(at[0], at[1], at[2]);
            source.previous = source.position;
            source.velocity = Vec3::new(velocity[0], velocity[1], velocity[2]);
            source.on_ground = false;
            source.needs_sync = true;
            clear_route(game, id);
            return;
        }

        let Some(source) = game.dropped.iter().find(|item| item.id == id).cloned() else {
            return;
        };
        let Some(child) = parts::spawn_part(game, &source, amount, at, velocity) else {
            continue;
        };
        let Some(original) = game.dropped.iter_mut().find(|item| {
            item.id == id && item.stack == source.stack && item.position == source.position
        }) else {
            if let Some(index) = game.dropped.iter().position(|item| item.id == child) {
                game.remove_dropped(index);
            }
            return;
        };
        remaining -= amount;
        original.stack.count = remaining;
        original.needs_sync = true;
        *pending = 0;
    }
    if route != before {
        let host: &mut dyn Host = game;
        let _stored = ROUTE.set(host, holder, Some(route));
    }
}

pub(super) fn draw(
    game: &Game<'_>,
    batch: &mut horizon_client::mod_api::HudBatch,
    [x, y]: [f32; 2],
) {
    let faces = [
        ("powercraft2.splitter.down", "Down", 0x005278),
        ("powercraft2.splitter.up", "Up", 0x971737),
        ("powercraft2.splitter.north", "North", 0x944400),
        ("powercraft2.splitter.south", "South", 0x495800),
        ("powercraft2.splitter.west", "West", 0x006632),
        ("powercraft2.splitter.east", "East", 0x382c91),
    ];
    for (column, (key, fallback, color)) in faces.into_iter().enumerate() {
        let label = game.translate(key, fallback);
        let left = x + 25.0 + column as f32 * 34.0 - game.native_text_width(&label) * 0.5;
        game.native_text(batch, &label, [left, y + 19.0], color, false);
    }
}
