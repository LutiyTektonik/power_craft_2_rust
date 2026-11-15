fn powercraft2_ejector_screen_draw(
    game: &crate::game::Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut horizon_client::mod_api::HudBatch,
    origin: [f32; 2],
) {
    if block == super::powercraft2_transport::EJECTOR_CONVEYOR {
        super::powercraft2_ejector::draw(game, data, batch, origin);
    } else if block == super::powercraft2_transport::SPLITTER {
        super::powercraft2_splitter::draw(game, batch, origin);
    }
}
