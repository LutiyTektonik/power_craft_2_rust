fn powercraft2_placement(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    context: &horizon_world::level::PlacementContext,
    answer: &mut Option<Option<horizon_world::registry::BlockStateId>>,
) {
    use crate::game::powercraft2_transport as transport;

    if answer.is_some()
        || level
            .native_state_block(state)
            .and_then(transport::Belt::from_block)
            .is_none()
    {
        return;
    }
    let Ok(direction) = transport::Direction::from_yaw(context.yaw) else {
        return;
    };
    if let Some(placed) = level.native_state_with_property(state, "facing", direction.as_str()) {
        *answer = Some(Some(placed));
    }
}
