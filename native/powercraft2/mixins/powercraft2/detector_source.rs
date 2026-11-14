fn powercraft2_detector_source(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    source: &mut bool,
) {
    if level.native_state_block(state)
        == Some(crate::game::powercraft2_transport::DETECTOR_CONVEYOR)
    {
        *source = true;
    }
}
