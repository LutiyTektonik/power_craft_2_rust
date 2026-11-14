fn powercraft2_ejector_menu_button(entity: &mut horizon_client::mod_api::ModBlockEntity, id: i32) {
    if entity.id == super::powercraft2_transport::EJECTOR_CONVEYOR {
        super::powercraft2_ejector::button(entity, id);
    }
}
