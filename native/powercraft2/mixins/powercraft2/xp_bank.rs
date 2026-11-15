use super::{Game, powercraft2_xp_bank_logic as logic};
use horizon_client::{
    containers::ModContainer,
    entities::ExperienceOrb,
    items::ItemStack,
    mod_api::{Host, HudBatch, ModBlockEntity, NativeLootDrops, NativeScreen, Tag},
};
use logic::{Action, CAPACITY, absorb, decode_balance, encode_balance, transfer};

pub(super) const XP_BANK: &str = "powercraft2:xp_bank";
const BALANCE: &str = "xp_bank_points";
const SAVED_BANK: &str = "minecraft:block_entity_data";
const SCAN_LIMIT: usize = 32;

fn balance(entity: &ModBlockEntity) -> i32 {
    match entity.nbt.get(&BALANCE.into()) {
        Some(Tag::Int(points)) => (*points).max(0),
        _ => 0,
    }
}

fn sync(entity: &mut ModBlockEntity, points: i32) {
    let data = encode_balance(points);
    if entity.data.as_slice() != data {
        entity.data = data.to_vec();
    }
}

fn set_balance(entity: &mut ModBlockEntity, points: i32) {
    entity.nbt.insert(BALANCE.into(), Tag::Int(points));
    sync(entity, points);
}

fn intersects(orb: &ExperienceOrb, [x, y, z]: [i32; 3]) -> bool {
    let [x, y, z] = [x, y, z].map(f64::from);
    orb.position.x + 0.25 > x - 0.5
        && orb.position.x - 0.25 < x + 1.5
        && orb.position.y + 0.5 > y - 0.5
        && orb.position.y < y + 1.5
        && orb.position.z + 0.25 > z - 0.5
        && orb.position.z - 0.25 < z + 1.5
}

pub(super) fn tick(game: &mut Game<'_>, block: [i32; 3], entity: &mut ModBlockEntity) {
    let previous = balance(entity);
    sync(entity, previous);
    let host: &dyn Host = game;
    if previous == CAPACITY || !host.on("powercraft2") || game.remote.is_some() {
        return;
    }
    let [x, y, z] = block;
    let room = (CAPACITY - previous) as u32;
    let candidates: Vec<_> = game
        .orbs
        .iter()
        .enumerate()
        .filter(|(_, orb)| {
            orb.count > 0 && (1..=room).contains(&orb.points()) && intersects(orb, block)
        })
        .take(SCAN_LIMIT)
        .map(|(index, orb)| (index, orb.id))
        .collect();
    let mut points = previous;
    // Removing later entries first keeps earlier indices valid in the ordinary
    // tick. Recheck the id because an entity-left listener can change the list.
    for (index, id) in candidates.into_iter().rev() {
        let index = if game.orbs.get(index).is_some_and(|orb| orb.id == id) {
            index
        } else if let Some(index) = game.orbs.iter().position(|orb| orb.id == id) {
            index
        } else {
            continue;
        };
        let orb = &game.orbs[index];
        if !intersects(orb, block) {
            continue;
        }
        let (next, taken) = absorb(points, orb.points(), orb.count);
        if taken == 0 {
            continue;
        }
        if taken == orb.count {
            game.remove_orb(index);
        } else {
            game.orbs[index].count -= taken;
        }
        points = next;
        if points == CAPACITY {
            break;
        }
    }
    if points != previous {
        set_balance(entity, points);
        game.audio.play(
            "minecraft:entity.experience_orb.pickup",
            Some([f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5]),
            0.1,
            1.0,
        );
    }
}

fn exchange(game: &mut Game<'_>, points: i32, action: Action) -> Option<i32> {
    let host: &dyn Host = game;
    if !host.on("powercraft2")
        || game.remote.is_some()
        || game.spectator()
        || game
            .player
            .as_ref()
            .is_none_or(|p| p.health.dead_or_dying())
    {
        return None;
    }
    let snapshot = game.experience;
    let result = transfer(points, snapshot.level, snapshot.progress, action)?;
    // If a listener cancels or reprices this exchange, neither balance moves.
    if game.xp_change(result.player_delta) != Some(result.player_delta)
        || game.experience.level != snapshot.level
        || game.experience.progress != snapshot.progress
        || game.experience.total != snapshot.total
    {
        return None;
    }
    game.experience.level = result.level;
    game.experience.progress = result.progress;
    game.experience.total = snapshot.total.saturating_add(result.player_delta).max(0);
    game.native_player_level_changed(snapshot.level);
    Some(result.balance)
}

pub(super) fn button(game: &mut Game<'_>, entity: &mut ModBlockEntity, id: i32) {
    let Ok(action) = Action::try_from(id) else {
        return;
    };
    if let Some(points) = exchange(game, balance(entity), action) {
        set_balance(entity, points);
    }
}

pub(super) fn harvest(game: &mut Game<'_>, context: &NativeLootDrops<'_>, drops: &mut [ItemStack]) {
    if context.table != "powercraft2:blocks/xp_bank"
        || context.used
        || context.explosion.is_some()
        || context.block.is_none()
        || context.player != Some(game.native_acting_id())
    {
        return;
    }
    let Some(stack) = drops.iter_mut().find(|stack| {
        stack.item.as_ref() == XP_BANK
            && stack.count == 1
            && stack
                .data()
                .and_then(|data| data.other(SAVED_BANK))
                .is_some_and(|state| state.get("id") == Some(&Tag::String(XP_BANK.into())))
    }) else {
        return;
    };
    let Some(Tag::Compound(mut saved)) = stack.data().and_then(|d| d.other(SAVED_BANK)).cloned()
    else {
        return;
    };
    let Some(Tag::Int(points)) = saved.get(&BALANCE.into()) else {
        return;
    };
    let Some(remainder) = exchange(game, *points, Action::WithdrawAll) else {
        return;
    };
    if remainder > 0 {
        saved.insert(BALANCE.into(), Tag::Int(remainder));
    } else {
        saved.remove(&BALANCE.into());
    }
    let keep_data = remainder > 0 || saved.len() > 1;
    stack
        .data_mut()
        .set_other(SAVED_BANK, keep_data.then_some(Tag::Compound(saved)));
    stack.tidy();
}

pub(super) fn container() -> ModContainer {
    ModContainer {
        slots: 1,
        menu: Some(XP_BANK.to_owned()),
        ..Default::default()
    }
}

pub(super) fn screen() -> NativeScreen {
    NativeScreen {
        width: 214.0,
        height: 170.0,
        slots: vec![None],
        inventory_y: 89.0,
        inventory_x: 26.0,
        ..Default::default()
    }
}

const BUTTONS: [(i32, f32, f32); 4] = [
    (10, 95.0, 28.0),
    (11, 151.0, 28.0),
    (20, 95.0, 49.0),
    (21, 151.0, 49.0),
];
const BUTTON_W: f32 = 52.0;
const BUTTON_H: f32 = 17.0;

pub(super) fn hit([x, y]: [f32; 2], right: bool) -> Option<i32> {
    if right {
        return None;
    }
    BUTTONS.iter().find_map(|&(id, left, top)| {
        ((left..left + BUTTON_W).contains(&x) && (top..top + BUTTON_H).contains(&y)).then_some(id)
    })
}

pub(super) fn draw(game: &Game<'_>, data: &[i32], batch: &mut HudBatch, [x, y]: [f32; 2]) {
    let stored = game.translate("powercraft2.xp_bank.stored", "Stored XP");
    let stored = format!("{stored} {}", decode_balance(data));
    game.native_text(batch, &stored, [x + 8.0, y + 16.0], 0x226025, false);
    for (key, fallback, top) in [
        ("powercraft2.xp_bank.withdraw", "Withdraw", 32.0),
        ("powercraft2.xp_bank.deposit", "Deposit", 53.0),
    ] {
        let label = game.translate(key, fallback);
        game.native_text(batch, &label, [x + 8.0, y + top], 0x303030, false);
    }
    let level = game.translate("powercraft2.xp_bank.player_level", "Your level");
    let level = format!("{level} {}", game.experience.level);
    game.native_text(batch, &level, [x + 8.0, y + 68.0], 0x303030, false);
    let one = game.translate("powercraft2.xp_bank.level", "1 level");
    let all = game.translate("powercraft2.xp_bank.all", "All");
    for (index, &(_, left, top)) in BUTTONS.iter().enumerate() {
        let label = if index % 2 == 0 { &one } else { &all };
        let left = x + left;
        let top = y + top;
        game.native_fill(batch, [left, top, BUTTON_W, BUTTON_H], 0xff535865);
        let inset = (BUTTON_W - game.native_text_width(label)) / 2.0;
        game.native_text(batch, label, [left + inset, top + 4.0], 0xffffff, false);
    }
}
