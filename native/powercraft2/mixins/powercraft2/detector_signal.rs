fn powercraft2_detector_signal(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    signal: &mut i32,
) {
    use crate::game::powercraft2_transport as transport;
    if level.native_state_block(state) == Some(transport::DETECTOR_CONVEYOR) {
        // Read state only, never query/update the world during redstone recursion.
        let powered = level.native_state_with_property(state, "powered", "true") == Some(state);
        *signal = transport::detector_signal(powered);
    }
}
