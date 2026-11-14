fn powercraft2_placement(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    context: &horizon_world::level::PlacementContext,
    answer: &mut Option<Option<horizon_world::registry::BlockStateId>>,
) {
    use crate::game::powercraft2_transport as transport;

    if answer.is_some() {
        return;
    }
    let Some(name) = level.native_state_block(state) else {
        return;
    };
    if name == transport::ELEVATOR_DOWN {
        let at = context.pos;
        let below = level.world.block(at.x, at.y - 1, at.z);
        let base = level.native_state_block(below) != Some(transport::ELEVATOR_DOWN);
        if let Some(placed) =
            level.native_state_with_property(state, "base", if base { "true" } else { "false" })
        {
            *answer = Some(Some(placed));
        }
        return;
    }
    if transport::Belt::from_block(name).is_none() {
        return;
    }
    let Ok(direction) = transport::Direction::from_yaw(context.yaw) else {
        return;
    };
    if let Some(placed) = level.native_state_with_property(state, "facing", direction.as_str()) {
        *answer = Some(Some(placed));
    }
}
