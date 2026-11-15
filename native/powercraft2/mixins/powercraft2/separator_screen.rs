fn powercraft2_separator_screen(
    block: &str,
    screen: &mut Option<horizon_client::mod_api::NativeScreen>,
) {
    use crate::game::powercraft2_transport as transport;

    if screen.is_some() {
        return;
    }
    if block == super::powercraft2_xp_bank::XP_BANK {
        *screen = Some(super::powercraft2_xp_bank::screen());
    } else if block == transport::EJECTOR_CONVEYOR {
        *screen = Some(horizon_client::mod_api::NativeScreen {
            width: 204.0,
            height: 205.0,
            slots: vec![None],
            inventory_y: 121.0,
            inventory_x: 22.0,
            ..Default::default()
        });
    } else if block == transport::SPLITTER {
        let slots = (0..transport::SPLITTER_SLOTS)
            .map(|index| {
                let bank = index / transport::SPLITTER_BANK_SIZE;
                let row = index % transport::SPLITTER_BANK_SIZE;
                Some([(16 + bank * 34) as f32, (32 + row * 18) as f32])
            })
            .collect();
        *screen = Some(horizon_client::mod_api::NativeScreen {
            width: 212.0,
            height: 204.0,
            slots,
            inventory_y: 122.0,
            inventory_x: 26.0,
            ..Default::default()
        });
    } else if block == transport::SEPARATOR_CONVEYOR {
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
