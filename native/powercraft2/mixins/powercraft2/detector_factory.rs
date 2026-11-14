fn powercraft2_detector_factory(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    use crate::game::powercraft2_transport as transport;
    if block == transport::DETECTOR_CONVEYOR && entity.is_none() {
        *entity = Some((
            transport::DETECTOR_CONVEYOR.to_owned(),
            horizon_world::nbt::Compound::new(),
        ));
    }
}
