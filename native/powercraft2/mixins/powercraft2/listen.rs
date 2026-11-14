fn powercraft2_listen(game: &mut crate::game::Game<'_>, listened: &mut bool) {
    let host: &mut dyn horizon_client::mod_api::Host = game;
    if host.on(crate::game::powercraft2_transport::MOD_ID) {
        *listened = true;
    }
}
