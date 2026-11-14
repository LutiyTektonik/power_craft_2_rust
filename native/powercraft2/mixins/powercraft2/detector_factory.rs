fn powercraft2_block_entity_factory(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    use crate::game::powercraft2_transport as transport;
    if matches!(
        block,
        transport::DETECTOR_CONVEYOR | transport::SEPARATOR_CONVEYOR | transport::EJECTOR_CONVEYOR
    ) && entity.is_none()
    {
        *entity = Some((block.to_owned(), horizon_world::nbt::Compound::new()));
    }
}
