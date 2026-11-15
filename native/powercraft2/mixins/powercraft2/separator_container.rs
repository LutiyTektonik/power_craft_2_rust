fn powercraft2_separator_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    use crate::game::powercraft2_transport as transport;

    if container.is_some() {
        return;
    }
    if block == super::powercraft2_xp_bank::XP_BANK {
        *container = Some(super::powercraft2_xp_bank::container());
        return;
    }
    let slots = match block {
        transport::SEPARATOR_CONVEYOR => transport::SEPARATOR_SLOTS,
        transport::EJECTOR_CONVEYOR => 1,
        transport::SPLITTER => transport::SPLITTER_SLOTS,
        _ => return,
    };
    *container = Some(horizon_client::containers::ModContainer {
        slots,
        menu: Some(block.to_owned()),
        ..Default::default()
    });
}
