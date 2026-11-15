fn powercraft2_detector_ticks(block: &str, ticks: &mut bool) {
    use crate::game::powercraft2_transport as transport;
    if matches!(
        block,
        transport::DETECTOR_CONVEYOR
            | transport::EJECTOR_CONVEYOR
            | super::powercraft2_xp_bank::XP_BANK
    ) || super::powercraft2_deco::has_block_entity(block)
    {
        *ticks = true;
    }
}
