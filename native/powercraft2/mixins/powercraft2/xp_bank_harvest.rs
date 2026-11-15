fn powercraft2_xp_bank_harvest(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    super::powercraft2_xp_bank::harvest(game, context, drops);
}
