fn powercraft2_ejector_menu_button(
    game: &mut crate::game::Game<'_>,
    entity: &mut horizon_client::mod_api::ModBlockEntity,
    id: i32,
) {
    if entity.id == super::powercraft2_transport::EJECTOR_CONVEYOR {
        super::powercraft2_ejector::button(entity, id);
    } else if entity.id == super::powercraft2_xp_bank::XP_BANK {
        super::powercraft2_xp_bank::button(game, entity, id);
    }
}
