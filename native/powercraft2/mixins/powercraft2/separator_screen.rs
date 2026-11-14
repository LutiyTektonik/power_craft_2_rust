fn powercraft2_separator_screen(
    block: &str,
    screen: &mut Option<horizon_client::mod_api::NativeScreen>,
) {
    use crate::game::powercraft2_transport as transport;

    if block == transport::SEPARATOR_CONVEYOR && screen.is_none() {
        let slots = (0..transport::SEPARATOR_SLOTS)
            .map(|index| {
                let bank = index / 9;
                let slot = index % 9;
                Some([
                    (22 + bank * 78 + (slot % 3) * 18) as f32,
                    (22 + (slot / 3) * 18) as f32,
                ])
            })
            .collect();
        *screen = Some(horizon_client::mod_api::NativeScreen {
            width: 176.0,
            height: 166.0,
            slots,
            inventory_y: 84.0,
            inventory_x: 8.0,
            ..Default::default()
        });
    }
}
