fn powercraft2_elevator_shape(
    pos: horizon_core::position::BlockPos,
    neighbour_pos: horizon_core::position::BlockPos,
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    direction: horizon_core::geometry::Direction,
    neighbour: horizon_world::registry::BlockStateId,
    new: &mut Option<horizon_world::registry::BlockStateId>,
) {
    use crate::game::powercraft2_transport::{ELEVATOR_DOWN, elevator_down_has_base};

    super::powercraft2_deco::shape(level, pos, state, neighbour_pos, neighbour, new);
    if new.is_none()
        && direction == horizon_core::geometry::Direction::Down
        && level.native_state_block(state) == Some(ELEVATOR_DOWN)
    {
        let base = elevator_down_has_base(level.native_state_block(neighbour));
        *new = level.native_state_with_property(state, "base", if base { "true" } else { "false" });
    }
}
