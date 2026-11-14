fn powercraft2_ejector_screen_click(
    block: &str,
    at: [f32; 2],
    right: bool,
    button: &mut Option<i32>,
) {
    if block == super::powercraft2_transport::EJECTOR_CONVEYOR && button.is_none() {
        *button = super::powercraft2_ejector::hit(at, right);
    }
}
