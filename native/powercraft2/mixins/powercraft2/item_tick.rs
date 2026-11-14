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
    if level.block(host, block).as_deref() != Some(transport::CONVEYOR) {
        return;
    }
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
    let Ok(mut next_velocity) = transport::velocity_for_tick(direction, block, motion, config)
    else {
        return;
    };
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
