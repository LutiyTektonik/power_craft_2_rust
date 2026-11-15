use super::{Game, powercraft2_transport as transport};
use horizon_client::{
    containers::BlockEntity,
    entities::ItemEntity,
    mod_api::{AttachmentType, Holder, Host, RandomSource},
};
use horizon_core::geometry::Vec3;
use transport::{Belt, Config, Direction, ItemMotion, SortOutput};

/// Owned by the item for one crossing. Discarded by the engine on unload.
static ROUTE: AttachmentType<Option<([i32; 3], Direction)>> = AttachmentType {
    namespace: "powercraft2",
    name: "separator_route",
    default: || None,
    codec: None,
    copy_on_death: false,
    sync: false,
};

pub(super) fn clear_route(game: &mut Game<'_>, id: u32) {
    // Most items never touch a sorter. Avoid allocating an attachment for them.
    if let Some(map) = game
        .persist
        .uuids
        .get(&id)
        .and_then(|uuid| game.entity_attachments.get_mut(uuid))
    {
        let _previous = map.remove(&ROUTE);
    }
}

pub(super) fn route(
    game: &mut Game<'_>,
    id: u32,
    block: [i32; 3],
    facing: Direction,
    motion: ItemMotion,
) -> Option<(Direction, ItemMotion)> {
    transport::velocity_for_tick(facing, block, motion, Config::default()).ok()?;
    let holder = Holder::Entity(id);
    let host: &mut dyn Host = game;
    if let Some((at, direction)) = ROUTE.get_existing(host, holder).ok()?.flatten()
        && at == block
    {
        return Some((direction, motion));
    }
    let source = game.dropped.iter().find(|item| item.id == id)?;
    let Some(BlockEntity::Mod(filters)) = game
        .world
        .block_entities
        .get(&(block[0], block[1], block[2]))
    else {
        return None;
    };
    if filters.id != transport::SEPARATOR_CONVEYOR {
        return None;
    }
    let weights = transport::separator_weights(filters.items.iter().map(|slot| {
        slot.as_ref().is_some_and(|sample| {
            sample.count > 0
                && sample.item == source.stack.item
                && sample.damage == source.stack.damage
        })
    }));
    let count = source.stack.count;
    let roll = if count == 1 && weights[0] != 0 && weights[1] != 0 {
        game.world
            .random
            .next_int_bound((weights[0] + weights[1]) as i32)
            .ok()? as u32
    } else {
        0
    };
    let has_belt = |direction: Direction| {
        let [x, y, z] = direction.neighbor(block);
        game.world.loaded(x, z)
            && game
                .pack
                .assets
                .registry
                .state(game.world.block(x, y, z))
                .is_some_and(|state| Belt::from_block(&state.name).is_some())
    };
    let [first, second] = transport::separator_outputs(facing, count, weights, has_belt, |_| roll);
    let (primary, secondary) = if first.count == 0 {
        (second, first)
    } else {
        (first, second)
    };
    let host: &mut dyn Host = game;
    ROUTE
        .set(host, holder, Some((block, primary.direction)))
        .ok()?;
    if secondary.count > 0 {
        split(game, id, block, facing, primary, secondary, motion);
    }
    let item = game.dropped.iter().find(|item| item.id == id)?;
    let adjusted = ItemMotion {
        position: [item.position.x, item.position.y, item.position.z],
        ..motion
    };
    Some((primary.direction, adjusted))
}

fn split(
    game: &mut Game<'_>,
    id: u32,
    block: [i32; 3],
    facing: Direction,
    primary: SortOutput,
    secondary: SortOutput,
    motion: ItemMotion,
) {
    let Some(source) = game.dropped.iter().find(|item| item.id == id).cloned() else {
        return;
    };
    let Some([primary_at, secondary_at]) =
        transport::separator_split_positions(block, facing, primary.direction, secondary.direction)
    else {
        return;
    };
    let config = Config::default();
    let Ok(child_velocity) = transport::velocity_for_tick(
        secondary.direction,
        block,
        ItemMotion {
            position: secondary_at,
            ..motion
        },
        config,
    ) else {
        return;
    };
    let Some(child_id) = spawn_part(game, &source, secondary.count, secondary_at, child_velocity)
    else {
        return;
    };
    let Some(child_index) = game.dropped.iter().position(|item| item.id == child_id) else {
        return;
    };
    let Some(index) = game.dropped.iter().position(|item| {
        item.id == id && item.stack == source.stack && item.position == source.position
    }) else {
        game.remove_dropped(child_index);
        return;
    };
    let host: &mut dyn Host = game;
    if ROUTE
        .set(
            host,
            Holder::Entity(child_id),
            Some((block, secondary.direction)),
        )
        .is_err()
    {
        game.remove_dropped(child_index);
        return;
    }
    let original = &mut game.dropped[index];
    original.stack.count = primary.count;
    original.position = Vec3::new(primary_at[0], primary_at[1], primary_at[2]);
    original.previous = original.position;
    original.needs_sync = true;
}

/// Creates an item with the source's components and lifetime. A join hook can refuse the spawn.
pub(super) fn spawn_part(
    game: &mut Game<'_>,
    source: &ItemEntity,
    count: u32,
    position: [f64; 3],
    velocity: [f64; 3],
) -> Option<u32> {
    let duration = source.pickup_delay.clamp(0, i64::from(u32::MAX)) as u32;
    let stack = horizon_client::items::ItemStack {
        count,
        ..source.stack.clone()
    };
    let id = game.next_entity.checked_add(1)?;
    game.spawn_item(stack.clone(), position, velocity, duration);
    let index = game.dropped.iter().position(|item| item.id == id)?;
    if game.dropped[index].stack != stack
        || game.dropped[index].position != Vec3::new(position[0], position[1], position[2])
    {
        game.remove_dropped(index);
        return None;
    }
    let child = &mut game.dropped[index];
    child.age = source.age;
    child.pickup_delay = source.pickup_delay;
    child.thrower = source.thrower;
    child.target = source.target;
    child.health = source.health;
    child.fire_ticks = source.fire_ticks;
    child.no_gravity = source.no_gravity;
    child.on_ground = source.on_ground;
    Some(id)
}
