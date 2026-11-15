fn powercraft2_block_entity_factory(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    use crate::game::powercraft2_transport as transport;
    let supported = matches!(
        block,
        transport::DETECTOR_CONVEYOR
            | transport::SEPARATOR_CONVEYOR
            | transport::EJECTOR_CONVEYOR
            | transport::SPLITTER
            | super::powercraft2_xp_bank::XP_BANK
    ) || super::powercraft2_deco::has_block_entity(block);
    if supported && entity.is_none() {
        *entity = Some((block.to_owned(), horizon_world::nbt::Compound::new()));
    }
}
