fn powercraft2_ejector_screen_click(
    block: &str,
    at: [f32; 2],
    right: bool,
    button: &mut Option<i32>,
) {
    if button.is_some() {
        return;
    }
    if block == super::powercraft2_xp_bank::XP_BANK {
        *button = super::powercraft2_xp_bank::hit(at, right);
    } else if block == super::powercraft2_transport::EJECTOR_CONVEYOR {
        *button = super::powercraft2_ejector::hit(at, right);
    }
}
