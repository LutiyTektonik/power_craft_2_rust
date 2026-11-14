fn powercraft2_client_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::powercraft2_transport as transport;
    use horizon_core::geometry::Vec3;

    // `tick` is after tick_remote_level on a client. The server's copy is
    // driven only by entity_tick_post. Never run this a second time there.
    if game.remote.is_none() || !game.native_on(transport::MOD_ID) {
        return;
    }
    let world = &game.world;
    let registry = &game.pack.assets.registry;
    let Ok(config) = transport::Config::new(transport::DEFAULT_SPEED, transport::DEFAULT_CENTERING)
    else {
        return;
    };
    // The public EntityRef setter intentionally rejects client writes. This
    // pin-specific bridge changes only predicted item velocity, not position,
    // previous position, ground, age, stack, pickup delay or network state.
    for item in &mut game.dropped {
        let position = [item.position.x, item.position.y, item.position.z];
        let Ok(block) = transport::support_block(position) else {
            continue;
        };
        if !world.chunks.contains_key(&(block[0] >> 4, block[2] >> 4)) {
            continue;
        }
        let Some(state) = registry.state(world.block(block[0], block[1], block[2])) else {
            continue;
        };
        if state.name.as_str() != transport::CONVEYOR {
            continue;
        }
        let Some(facing) = state.properties.get("facing") else {
            continue;
        };
        let Ok(direction) = transport::Direction::try_from(facing.as_str()) else {
            continue;
        };
        let motion = transport::ItemMotion {
            position,
            velocity: [item.velocity.x, item.velocity.y, item.velocity.z],
            on_ground: item.on_ground,
        };
        let Ok(mut velocity) = transport::velocity_for_tick(direction, block, motion, config)
        else {
            continue;
        };
        let Ok(next) = transport::next_support_block(position, velocity) else {
            continue;
        };
        if !world.chunks.contains_key(&(next[0] >> 4, next[2] >> 4)) {
            velocity[0] = 0.0;
            velocity[2] = 0.0;
        }
        item.velocity = Vec3::new(velocity[0], velocity[1], velocity[2]);
    }
}
