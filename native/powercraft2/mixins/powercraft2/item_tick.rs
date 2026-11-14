fn powercraft2_item_tick(
    game: &mut crate::game::Game<'_>,
    dimension: &str,
    entity: u32,
    kind: &str,
) {
    use crate::game::powercraft2_transport as transport;

    if kind != "minecraft:item" {
        return;
    }
    let host: &mut dyn horizon_client::mod_api::Host = game;
    if !host.on(transport::MOD_ID) {
        return;
    }
    // A Post may report an item removed by merging/expiry. Never retain a handle.
    let (Some(item), Some(level)) = (host.entity(entity), host.level(dimension)) else {
        return;
    };
    let Some(position) = item.position(host) else {
        return;
    };
    let Ok(block) = transport::support_block(position) else {
        return;
    };
    let Some(name) = level.block(host, block) else {
        return;
    };
    let Some(belt) = transport::Belt::from_block(&name) else {
        return;
    };
    let Some(facing) = level.block_property(host, block, "facing") else {
        return;
    };
    let Ok(direction) = transport::Direction::try_from(facing.as_str()) else {
        return;
    };
    let (Some(velocity), Some(on_ground)) = (item.velocity(host), item.on_ground(host)) else {
        return;
    };
    let motion = transport::ItemMotion {
        position,
        velocity,
        on_ground,
    };
    let Ok(config) = transport::Config::new(transport::DEFAULT_SPEED, transport::DEFAULT_CENTERING)
    else {
        return;
    };
    let powered = if matches!(belt, transport::Belt::Brake | transport::Belt::Redirector) {
        let powered = host.neighbor_signal(dimension, block) > 0;
        let value = if powered { "true" } else { "false" };
        // Client neighbor_signal deliberately returns zero. Publish only a
        // changed property on OUR brake block using UPDATE_CLIENTS (2), keeping
        // facing and all entity data intact. No neighbour-update recursion.
        if level.block_property(host, block, "powered").as_deref() != Some(value)
            && !host.set_block_property_with(block, "powered", value, 2)
        {
            return;
        }
        powered
    } else {
        false
    };
    let route = if belt == transport::Belt::Redirector {
        transport::redirect_direction(direction, powered, |side| {
            let at = side.neighbor(block);
            if !level.is_loaded(host, at) {
                return transport::RedirectNeighbor::Obstructed;
            }
            transport::RedirectNeighbor::from_block(level.block(host, at).as_deref())
        })
    } else {
        direction
    };
    let Ok(mut next_velocity) =
        transport::belt_velocity_for_tick(belt, route, block, motion, config, powered)
    else {
        return;
    };
    // In the published redirector, storage uses the placed facing before
    // redirecting any remaining items.
    if let Ok(mut targets) = transport::storage_targets(belt, direction, block, motion, powered) {
        // No entity snapshot/allocation on an ordinary line without a target.
        for target in &mut targets {
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
            let Some(snapshot) = found else { return };
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
                    return;
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
                    return;
                }
                if transfer.remaining() == 0 {
                    return;
                }
            }
        }
    }
    // Do not drive an item across a chunk edge that the server has not loaded.
    // Vanilla still owns collisions, gravity, item saves, merging and pickup.
    let Ok(next_block) = transport::next_support_block(position, next_velocity) else {
        return;
    };
    if !level.is_loaded(host, next_block) {
        next_velocity[0] = 0.0;
        next_velocity[2] = 0.0;
    }
    // False means the engine rejected a stale/non-simulated entity; do not retry.
    let _changed = item.set_velocity(host, next_velocity);
}
