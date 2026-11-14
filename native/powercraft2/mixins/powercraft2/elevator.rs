use super::{Game, powercraft2_transport as transport};
use horizon_client::mod_api::{Host, LevelRef};
use transport::{Elevator, ElevatorCell, ItemMotion, StorageTarget};

pub(super) fn tick(
    game: &mut Game<'_>,
    dimension: &str,
    entity: u32,
    block: [i32; 3],
    position: [f64; 3],
    elevator: Elevator,
) {
    let host: &mut dyn Host = game;
    let (Some(item), Some(level)) = (host.entity(entity), host.level(dimension)) else {
        return;
    };
    let Some(velocity) = item.velocity(host) else {
        return;
    };
    let motion = ItemMotion {
        position,
        velocity,
        on_ground: false,
    };
    let powered = host.neighbor_signal(dimension, block) > 0;
    let value = if powered { "true" } else { "false" };
    if level.block_property(host, block, "powered").as_deref() != Some(value)
        && !host.set_block_property_with(block, "powered", value, 2)
    {
        return;
    }
    let Ok(velocity) = transport::elevator_velocity(elevator, block, motion, powered, |at| {
        if !level.is_loaded(host, at) {
            return ElevatorCell::Other;
        }
        level.block(host, at).map_or(ElevatorCell::Other, |name| {
            let facing = if transport::Belt::from_block(&name).is_some() {
                level.block_property(host, at, "facing")
            } else {
                None
            };
            ElevatorCell::from_block(&name, facing.as_deref(), elevator)
        })
    }) else {
        return;
    };
    let mut targets = transport::elevator_storage_targets(block);
    if !unload(host, level, dimension, entity, position, &mut targets) {
        let _changed = item.set_velocity(host, velocity);
    }
}

/// Returns true when the item should no longer be driven this tick.
pub(super) fn unload(
    host: &mut dyn Host,
    level: LevelRef,
    dimension: &str,
    entity: u32,
    position: [f64; 3],
    targets: &mut [Option<StorageTarget>],
) -> bool {
    for target in targets.iter_mut() {
        if let Some(at) = target
            && (!level.is_loaded(host, at.position)
                || !level
                    .block(host, at.position)
                    .is_some_and(|name| transport::supports_storage(&name)))
        {
            *target = None;
        }
    }
    if targets.iter().any(Option::is_some) {
        let found = host
            .item_entities(
                dimension,
                [
                    position[0] - 0.125,
                    position[1] - 0.125,
                    position[2] - 0.125,
                ],
                [
                    position[0] + 0.125,
                    position[1] + 0.375,
                    position[2] + 0.125,
                ],
            )
            .into_iter()
            .find(|snapshot| snapshot.id == entity);
        let Some(snapshot) = found else { return true };
        let stack = snapshot.stack;
        let mut transfer = transport::Transfer::new(stack.count);
        for target in targets.into_iter().flatten() {
            let count = transfer.offer();
            if count == 0 {
                break;
            }
            use horizon_core::geometry::Direction;
            let face = match target.face {
                transport::StorageFace::Down => Direction::Down,
                transport::StorageFace::Up => Direction::Up,
                transport::StorageFace::North => Direction::North,
                transport::StorageFace::East => Direction::East,
                transport::StorageFace::South => Direction::South,
                transport::StorageFace::West => Direction::West,
            };
            let offered = horizon_client::items::ItemStack {
                count,
                ..stack.clone()
            };
            let returned = host.insert_item(target.position, face, offered);
            if transfer
                .accept(returned.as_ref().map_or(0, |rest| rest.count))
                .is_err()
            {
                log::error!("PowerCraft: native insert_item violated its remainder contract");
                return true;
            }
        }
        if transfer.remaining() != stack.count {
            // This synchronous sequence has no await, world swap or
            // mutable Game callback for the explicitly audited containers.
            // Preserve every component: only the source count changes.
            let remaining = horizon_client::items::ItemStack {
                count: transfer.remaining(),
                ..stack
            };
            if !host.set_item_entity(dimension, entity, remaining) {
                log::error!("PowerCraft: source item vanished during synchronous insertion");
                return true;
            }
            if transfer.remaining() == 0 {
                return true;
            }
        }
    }
    false
}
