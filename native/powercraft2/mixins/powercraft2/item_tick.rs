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
    let Ok(occupied) = transport::occupied_block(position) else {
        return;
    };
    let occupied_name = level.block(host, occupied);
    if occupied_name.as_deref() == Some(transport::SPLITTER) {
        super::powercraft2_separator::clear_route(game, entity);
        super::powercraft2_splitter::tick(game, entity, occupied, position);
        return;
    }
    super::powercraft2_splitter::clear_route(game, entity);
    if let Some(elevator) = occupied_name
        .as_deref()
        .and_then(transport::Elevator::from_block)
    {
        super::powercraft2_separator::clear_route(game, entity);
        super::powercraft2_elevator::tick(game, dimension, entity, occupied, position, elevator);
        return;
    }
    let host: &mut dyn horizon_client::mod_api::Host = game;
    let Some(name) = level.block(host, block) else {
        return;
    };
    if name != transport::SEPARATOR_CONVEYOR {
        super::powercraft2_separator::clear_route(game, entity);
    }
    let host: &mut dyn horizon_client::mod_api::Host = game;
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
    let (sort_route, motion) = if belt == transport::Belt::Separator {
        let Some((route, adjusted)) =
            super::powercraft2_separator::route(game, entity, block, direction, motion)
        else {
            return;
        };
        (Some(route), adjusted)
    } else {
        (None, motion)
    };
    let host: &mut dyn horizon_client::mod_api::Host = game;
    let position = motion.position;
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
    let route = if let Some(route) = sort_route {
        route
    } else if belt == transport::Belt::Redirector {
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
    let storage_direction = sort_route.unwrap_or(direction);
    if let Ok(mut targets) =
        transport::storage_targets(belt, storage_direction, block, motion, powered)
    {
        if super::powercraft2_elevator::unload(
            host,
            level,
            dimension,
            entity,
            position,
            &mut targets,
        ) {
            return;
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
