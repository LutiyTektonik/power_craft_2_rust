fn powercraft2_xp_bank_loot(
    kind: &str,
    ctx: &crate::loot::NativeLootContext<'_>,
    stack: &mut crate::loot::LootStack,
) {
    use crate::nbt::Tag;
    const XP_BANK: &str = "powercraft2:xp_bank";
    if kind != "powercraft2:retain_bank_points" || stack.item != XP_BANK || stack.count != 1 {
        return;
    }
    let Some(entity) = ctx
        .place
        .as_ref()
        .and_then(|place| place.block_entity.as_ref())
    else {
        return;
    };
    if entity.get("id") != Some(&Tag::String(XP_BANK.into())) {
        return;
    }
    let Some(Tag::Int(points)) = entity.get("xp_bank_points") else {
        return;
    };
    if *points > 0 {
        let component = "minecraft:block_entity_data";
        stack.components.retain(|(id, _)| id != component);
        stack.components.push((
            component.to_owned(),
            serde_json::json!({ "id": XP_BANK, "xp_bank_points": *points }),
        ));
    }
}
