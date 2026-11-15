use horizon_core::position::BlockPos;
use horizon_world::{level::PlacementContext, registry::BlockStateId};

use super::{
    Game, GameLevel,
    powercraft2_transport::{Belt, Direction},
};

const PLATFORM: &str = "powercraft2:platform";
const STAIRS: &str = "powercraft2:stairs";
const HORIZONTAL: [[i32; 2]; 4] = [[0, -1], [1, 0], [0, 1], [-1, 0]];
const MASKS: [&str; 16] = [
    "mask_0", "mask_1", "mask_2", "mask_3", "mask_4", "mask_5", "mask_6", "mask_7", "mask_8",
    "mask_9", "mask_10", "mask_11", "mask_12", "mask_13", "mask_14", "mask_15",
];

pub(super) fn has_block_entity(block: &str) -> bool {
    matches!(block, PLATFORM | STAIRS)
}

fn is_climbable(name: &str) -> bool {
    matches!(name, "minecraft:ladder" | "minecraft:vine")
}

fn is_drop(name: &str, solid: impl FnOnce() -> bool) -> bool {
    !is_climbable(name)
        && (matches!(
            name,
            "minecraft:air"
                | "minecraft:cave_air"
                | "minecraft:void_air"
                | "minecraft:water"
                | "minecraft:lava"
                | "powercraft2:elevator_up"
                | "powercraft2:elevator_down"
                | "powercraft2:splitter"
        ) || Belt::from_block(name).is_some()
            || !solid())
}

/// Rails face the exposed sides of the two-block-deep ledge; stair rail bits
/// describe the player's left and right relative to the high tread.
fn rail_mask(at: [i32; 3], facing: Option<Direction>, drop: impl Fn([i32; 3]) -> bool) -> usize {
    let [x, y, z] = at;
    let exposed = |side| {
        let [dx, dz] = HORIZONTAL[side];
        drop([x + dx, y, z + dz]) && drop([x + dx, y - 1, z + dz])
    };
    match facing {
        Some(facing) => {
            let direction = match facing {
                Direction::North => 0,
                Direction::East => 1,
                Direction::South => 2,
                Direction::West => 3,
            };
            usize::from(exposed((direction + 3) % 4))
                | usize::from(exposed((direction + 1) % 4)) << 1
        }
        None => (0..4).map(|side| usize::from(exposed(side)) << side).sum(),
    }
}

fn level_block(
    level: &GameLevel<'_>,
    p: [i32; 3],
    changed: Option<(BlockPos, BlockStateId)>,
) -> BlockStateId {
    let [x, y, z] = p;
    match changed {
        Some((pos, state)) if [pos.x, pos.y, pos.z] == p => state,
        _ => level.world.block(x, y, z),
    }
}

fn geometry(
    level: &GameLevel<'_>,
    mut state: BlockStateId,
    pos: BlockPos,
    facing: Option<Direction>,
    changed: Option<(BlockPos, BlockStateId)>,
) -> Option<BlockStateId> {
    let at = [pos.x, pos.y, pos.z];
    let rails = rail_mask(at, facing, |p| {
        let state = level_block(level, p, changed);
        let name = level.native_state_block(state).unwrap_or("minecraft:air");
        is_drop(name, || {
            level
                .behaviours
                .collision_shape(state)
                .is_none_or(|shape| !shape.is_empty())
        })
    });
    state = level.native_state_with_property(state, "railings", MASKS[rails])?;
    if facing.is_none() {
        let below = level_block(level, [pos.x, pos.y - 1, pos.z], changed);
        let climbable = level.native_state_block(below).is_some_and(is_climbable);
        state = level.native_state_with_property(
            state,
            "floor",
            if climbable { "false" } else { "true" },
        )?;
    }
    Some(state)
}

pub(super) fn place(
    level: &GameLevel<'_>,
    mut state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    if answer.is_some() {
        return;
    }
    let facing = match level.native_state_block(state) {
        Some(PLATFORM) => None,
        Some(STAIRS) => {
            let Ok(facing) = Direction::from_yaw(context.yaw) else {
                return;
            };
            let Some(turned) = level.native_state_with_property(state, "facing", facing.as_str())
            else {
                return;
            };
            state = turned;
            Some(facing)
        }
        _ => return,
    };
    if let Some(placed) = geometry(level, state, context.pos, facing, None) {
        *answer = Some(Some(placed));
    }
}

pub(super) fn shape(
    level: &GameLevel<'_>,
    pos: BlockPos,
    state: BlockStateId,
    neighbour_pos: BlockPos,
    neighbour: BlockStateId,
    new: &mut Option<BlockStateId>,
) {
    if new.is_some() {
        return;
    }
    let facing = match level.native_state_block(state) {
        Some(PLATFORM) => None,
        Some(STAIRS) => level
            .registry
            .state(state)
            .and_then(|state| state.properties.get("facing"))
            .and_then(|facing| Direction::try_from(facing.as_str()).ok()),
        _ => return,
    };
    *new = geometry(level, state, pos, facing, Some((neighbour_pos, neighbour)));
}

pub(super) fn tick(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &horizon_client::mod_api::ModBlockEntity,
) {
    let [x, y, z] = pos;
    // Only diagonal support changes can miss updateShape. Spread this check over
    // ten ticks, only for the platforms and stairs already loaded and ticking.
    if (game.world.game_time + i64::from(x) + i64::from(y) + i64::from(z)).rem_euclid(10) != 0 {
        return;
    }
    let facing = match entity.id.as_str() {
        PLATFORM => None,
        STAIRS => game
            .native_block_property(pos, "facing")
            .as_deref()
            .and_then(|facing| Direction::try_from(facing).ok()),
        _ => return,
    };
    let rails = rail_mask(pos, facing, |p| {
        let name = game.native_block(p);
        is_drop(&name, || !game.block_collision_shape(p).is_empty())
    });
    let _changed = game.native_set_block_property_with(pos, "railings", MASKS[rails], 2);
    if facing.is_none() {
        let climbable = is_climbable(&game.native_block([x, y - 1, z]));
        let _changed = game.native_set_block_property_with(
            pos,
            "floor",
            if climbable { "false" } else { "true" },
            2,
        );
    }
}
