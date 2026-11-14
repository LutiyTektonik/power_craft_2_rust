fn powercraft2_elevator_shape(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    direction: horizon_core::geometry::Direction,
    neighbour: horizon_world::registry::BlockStateId,
    new: &mut Option<horizon_world::registry::BlockStateId>,
) {
    use crate::game::powercraft2_transport::ELEVATOR_DOWN;

    if new.is_none()
        && direction == horizon_core::geometry::Direction::Down
        && level.native_state_block(state) == Some(ELEVATOR_DOWN)
    {
        let base = level.native_state_block(neighbour) != Some(ELEVATOR_DOWN);
        *new = level.native_state_with_property(state, "base", if base { "true" } else { "false" });
    }
}
