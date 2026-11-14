fn powercraft2_detector_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::mod_api::ModBlockEntity,
) {
    use crate::game::powercraft2_transport as transport;
    use horizon_core::{
        geometry::{Aabb, Vec3},
        position::BlockPos,
    };
    use horizon_world::level::{EntityQuery, LevelAccess};

    if entity.id == transport::EJECTOR_CONVEYOR {
        super::powercraft2_ejector::tick(game, pos, entity);
        return;
    }
    if entity.id != transport::DETECTOR_CONVEYOR {
        return;
    }
    let Ok((min, max)) = transport::detector_area(pos) else {
        return;
    };
    let (dimension, previous, enabled) = {
        let host: &mut dyn horizon_client::mod_api::Host = game;
        let dimension = host.dimension();
        let Some(level) = host.level(dimension) else {
            return;
        };
        if !level.is_loaded(host, pos)
            || level.block(host, pos).as_deref() != Some(transport::DETECTOR_CONVEYOR)
        {
            return;
        }
        let previous = match level.block_property(host, pos, "powered").as_deref() {
            Some("true") => true,
            Some("false") => false,
            _ => return,
        };
        (dimension, previous, host.on(transport::MOD_ID))
    };
    // The Host entities_in helper filters entity centres and is documented
    // for occasional use. The existing level query uses actual AABBs, includes
    // server/LAN players and honors physical-trigger cancellation. with_level
    // is a minimal, read-only, pin-specific bridge; no engine code is patched.
    let occupied = enabled
        && game.with_level(|_, level, _| {
            level.count_entities_at(
                BlockPos::new(pos[0], pos[1], pos[2]),
                transport::DETECTOR_CONVEYOR,
                Aabb::new(
                    Vec3::new(min[0], min[1], min[2]),
                    Vec3::new(max[0], max[1], max[2]),
                ),
                EntityQuery::Everything,
            ) > 0
        });
    // data is explicitly transient (not saved). Re-notify neighbours once
    // after each load, even when saved powered already equals occupancy.
    let first_tick = entity.data.as_slice() != [1];
    let update = transport::detector_update(previous, occupied, enabled, first_tick);
    let host: &mut dyn horizon_client::mod_api::Host = game;
    if update.powered != previous
        && !host.set_block_property_with(
            pos,
            "powered",
            if update.powered { "true" } else { "false" },
            2,
        )
    {
        return;
    }
    if update.notify_neighbors && !host.update_signal_neighbors(dimension, pos) {
        return;
    }
    if first_tick {
        entity.data.clear();
        entity.data.push(1);
    }
    if update.click {
        let _heard = host.play_sound(
            "minecraft:block.lever.click",
            [
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + 0.125,
                f64::from(pos[2]) + 0.5,
            ],
            4,
            0.15,
            0.5,
        );
    }
}
