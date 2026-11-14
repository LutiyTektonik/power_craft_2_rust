fn powercraft2_separator_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    use crate::game::powercraft2_transport as transport;

    if block == transport::SEPARATOR_CONVEYOR && container.is_none() {
        *container = Some(horizon_client::containers::ModContainer {
            slots: transport::SEPARATOR_SLOTS,
            menu: Some(transport::SEPARATOR_CONVEYOR.to_owned()),
            ..Default::default()
        });
    }
}
