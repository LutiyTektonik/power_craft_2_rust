fn powercraft2_detector_ticks(block: &str, ticks: &mut bool) {
    if block == crate::game::powercraft2_transport::DETECTOR_CONVEYOR {
        *ticks = true;
    }
}
