// Shared by the standalone Cargo library and Horizon's injected module.
// No engine types, clocks, global state, IO, or allocations in the movement path.

pub const MOD_ID: &str = "powercraft2";
pub const CONVEYOR: &str = "powercraft2:conveyor";
pub const SPEEDY_CONVEYOR: &str = "powercraft2:speedy_conveyor";
pub const BRAKE_CONVEYOR: &str = "powercraft2:brake_conveyor";
pub const DETECTOR_CONVEYOR: &str = "powercraft2:detector_conveyor";
pub const REDIRECTOR_CONVEYOR: &str = "powercraft2:redirector_conveyor";
pub const SEPARATOR_CONVEYOR: &str = "powercraft2:separator_conveyor";
pub const EJECTOR_CONVEYOR: &str = "powercraft2:ejector_conveyor";
pub const SEPARATOR_SLOTS: usize = 18;
pub const BELT_HEIGHT: f64 = 0.25;
pub const DEFAULT_SPEED: f64 = 0.1;
pub const DEFAULT_CENTERING: f64 = 0.04;
const CONTACT_EPSILON: f64 = 1.0 / 64.0;
const WORLD_BORDER: f64 = 30_000_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Belt {
    Normal,
    Speedy,
    Brake,
    Detector,
    Redirector,
    Separator,
    Ejector,
}

impl Belt {
    pub fn from_block(block: &str) -> Option<Self> {
        match block {
            CONVEYOR => Some(Self::Normal),
            SPEEDY_CONVEYOR => Some(Self::Speedy),
            BRAKE_CONVEYOR => Some(Self::Brake),
            DETECTOR_CONVEYOR => Some(Self::Detector),
            REDIRECTOR_CONVEYOR => Some(Self::Redirector),
            SEPARATOR_CONVEYOR => Some(Self::Separator),
            EJECTOR_CONVEYOR => Some(Self::Ejector),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnknownDirection,
    InvalidConfiguration,
    InvalidEntity,
    NotOnSurface,
    InvalidTransfer,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UnknownDirection => "unknown conveyor direction",
            Self::InvalidConfiguration => "invalid conveyor configuration",
            Self::InvalidEntity => "invalid entity coordinates or velocity",
            Self::NotOnSurface => "item is not resting on the conveyor surface",
            Self::InvalidTransfer => "inventory returned more items than offered",
        })
    }
}

impl std::error::Error for Error {}

impl Direction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
        }
    }

    pub const fn vector(self) -> [i32; 3] {
        match self {
            Self::North => [0, 0, -1],
            Self::East => [1, 0, 0],
            Self::South => [0, 0, 1],
            Self::West => [-1, 0, 0],
        }
    }

    const fn left(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }

    const fn right(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    pub const fn opposite(self) -> Self {
        self.left().left()
    }

    pub fn neighbor(self, [x, y, z]: [i32; 3]) -> [i32; 3] {
        // Callers derive the position from support_block; horizontal bounds
        // leave room for one neighbour on either side.
        let [dx, _, dz] = self.vector();
        [x + dx, y, z + dz]
    }

    pub fn from_yaw(yaw: f32) -> Result<Self, Error> {
        if !yaw.is_finite() {
            return Err(Error::InvalidEntity);
        }
        let quadrant = ((yaw.rem_euclid(360.0) / 90.0 + 0.5).floor() as u32) % 4;
        Ok(match quadrant {
            0 => Self::South,
            1 => Self::West,
            2 => Self::North,
            _ => Self::East,
        })
    }
}

impl TryFrom<&str> for Direction {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "north" => Ok(Self::North),
            "east" => Ok(Self::East),
            "south" => Ok(Self::South),
            "west" => Ok(Self::West),
            _ => Err(Error::UnknownDirection),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    speed: f64,
    centering: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            speed: DEFAULT_SPEED,
            centering: DEFAULT_CENTERING,
        }
    }
}

impl Config {
    // Blocks per simulation tick, never blocks per rendered frame.
    pub fn new(speed: f64, centering: f64) -> Result<Self, Error> {
        if !speed.is_finite()
            || !centering.is_finite()
            || !(0.0..=0.5).contains(&speed)
            || speed == 0.0
            || !(0.0..=0.1).contains(&centering)
        {
            return Err(Error::InvalidConfiguration);
        }
        Ok(Self { speed, centering })
    }
}

pub fn next_support_block(position: [f64; 3], velocity: [f64; 3]) -> Result<[i32; 3], Error> {
    if !velocity.iter().all(|v| v.is_finite()) {
        return Err(Error::InvalidEntity);
    }
    support_block([
        position[0] + velocity[0],
        position[1],
        position[2] + velocity[2],
    ])
}

#[derive(Clone, Copy, Debug)]
pub struct ItemMotion {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub on_ground: bool,
}

// Subtracting a small offset also finds support at integer-height boundaries.
// Validate before conversion: Rust's float-to-integer casts saturate NaN/overflow.
pub fn support_block(position: [f64; 3]) -> Result<[i32; 3], Error> {
    if !position.iter().all(|n| n.is_finite())
        || position[0].abs() >= WORLD_BORDER
        || position[2].abs() >= WORLD_BORDER
        || position[1] <= f64::from(i32::MIN) + 1.0
        || position[1] >= f64::from(i32::MAX) - 1.0
    {
        return Err(Error::InvalidEntity);
    }
    Ok([
        position[0].floor() as i32,
        (position[1] - CONTACT_EPSILON).floor() as i32,
        position[2].floor() as i32,
    ])
}

pub fn velocity_for_tick(
    direction: Direction,
    block: [i32; 3],
    item: ItemMotion,
    config: Config,
) -> Result<[f64; 3], Error> {
    let sampled = support_block(item.position)?;
    if !item.velocity.iter().all(|n| n.is_finite()) {
        return Err(Error::InvalidEntity);
    }
    let surface = f64::from(block[1]) + BELT_HEIGHT;
    if sampled != block || !item.on_ground || (item.position[1] - surface).abs() > CONTACT_EPSILON {
        return Err(Error::NotOnSurface);
    }
    let [dx, _, dz] = direction.vector();
    let center_x = f64::from(block[0]) + 0.5;
    let center_z = f64::from(block[2]) + 0.5;
    let center = |offset: f64| offset.clamp(-config.centering, config.centering);
    Ok([
        if dx == 0 {
            center(center_x - item.position[0])
        } else {
            f64::from(dx) * config.speed
        },
        item.velocity[1],
        if dz == 0 {
            center(center_z - item.position[2])
        } else {
            f64::from(dz) * config.speed
        },
    ])
}

// Preserve the tested normal belt exactly. Variants use the same contact and
// tick rules on both sides; power is authoritative on the server and mirrored
// in the brake block's own `powered` state for client prediction.
pub fn belt_velocity_for_tick(
    belt: Belt,
    direction: Direction,
    block: [i32; 3],
    item: ItemMotion,
    config: Config,
    powered: bool,
) -> Result<[f64; 3], Error> {
    let config = match belt {
        Belt::Normal | Belt::Detector | Belt::Redirector | Belt::Separator | Belt::Ejector => {
            config
        }
        Belt::Speedy => Config::new(config.speed * 2.0, config.centering)?,
        Belt::Brake => Config::new(config.speed * 0.6, config.centering)?,
    };
    let driven = velocity_for_tick(direction, block, item, config)?;
    if belt == Belt::Brake && powered {
        // No belt thrust while powered. Vanilla friction/collisions run first;
        // damp the remaining horizontal velocity, keeping gravity/pickup intact.
        Ok([
            item.velocity[0] * 0.6,
            item.velocity[1],
            item.velocity[2] * 0.6,
        ])
    } else {
        Ok(driven)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedirectNeighbor {
    Belt,
    Air,
    Obstructed,
}

impl RedirectNeighbor {
    pub fn from_block(block: Option<&str>) -> Self {
        match block {
            Some("minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") => Self::Air,
            Some(block) if Belt::from_block(block).is_some() => Self::Belt,
            _ => Self::Obstructed,
        }
    }
}

pub fn redirect_direction(
    facing: Direction,
    powered: bool,
    mut neighbor: impl FnMut(Direction) -> RedirectNeighbor,
) -> Direction {
    // Forge's LEFT/RIGHT are defined in the placement frame. In world
    // coordinates LEFT is clockwise, and the unpowered check looks behind.
    let right = facing.right();
    let right_neighbor = neighbor(right);
    if powered && right_neighbor != RedirectNeighbor::Obstructed {
        return right;
    }
    let left = facing.left();
    let left_neighbor = neighbor(left);
    if (powered && left_neighbor != RedirectNeighbor::Obstructed)
        || (left_neighbor == RedirectNeighbor::Belt
            && right_neighbor == RedirectNeighbor::Belt
            && neighbor(facing.opposite()) != RedirectNeighbor::Belt)
    {
        return left;
    }
    facing
}

/// The menu has two 3×3 banks, displayed left/right relative to the direction of travel.
pub fn separator_weights(matches: impl Iterator<Item = bool>) -> [u32; 2] {
    let mut weights = [0; 2];
    for (slot, matched) in matches.take(SEPARATOR_SLOTS).enumerate() {
        weights[slot / (SEPARATOR_SLOTS / 2)] += u32::from(matched);
    }
    weights
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortOutput {
    pub direction: Direction,
    pub count: u32,
}

pub fn separator_outputs(
    facing: Direction,
    count: u32,
    [left, right]: [u32; 2],
    has_belt: impl Fn(Direction) -> bool,
    roll: impl FnOnce(u32) -> u32,
) -> [SortOutput; 2] {
    let single = |direction| {
        [
            SortOutput { direction, count },
            SortOutput {
                direction,
                count: 0,
            },
        ]
    };
    match (left, right) {
        (0, 0) => return single(facing),
        (_, 0) => return single(facing.left()),
        (0, _) => return single(facing.right()),
        _ => {}
    }
    let (left_exit, right_exit) = match (has_belt(facing.left()), has_belt(facing.right())) {
        (false, true) => (facing, facing.right()),
        (true, false) => (facing.left(), facing),
        _ => (facing.left(), facing.right()),
    };
    let total = left + right;
    if count == 1 {
        return single(if roll(total) < left {
            left_exit
        } else {
            right_exit
        });
    }
    let left_count =
        ((u64::from(count) * u64::from(left) + u64::from(total / 2)) / u64::from(total)) as u32;
    [
        SortOutput {
            direction: left_exit,
            count: left_count,
        },
        SortOutput {
            direction: right_exit,
            count: count - left_count,
        },
    ]
}

/// Place split stacks in separate lanes on the same belt. Their centres
/// start 0.8 blocks apart on the cross axis, outside vanilla merge reach.
pub fn separator_split_positions(
    block: [i32; 3],
    facing: Direction,
    first: Direction,
    second: Direction,
) -> Option<[[f64; 3]; 2]> {
    let left = facing.left();
    let right = facing.right();
    let lane = |route, other| {
        if route == facing {
            if other == left { right } else { left }
        } else {
            route
        }
    };
    if first == second {
        return None;
    }
    let place = |route, other| {
        let [dx, _, dz] = lane(route, other).vector();
        [
            f64::from(block[0]) + 0.5 + 0.4 * f64::from(dx),
            f64::from(block[1]) + BELT_HEIGHT,
            f64::from(block[2]) + 0.5 + 0.4 * f64::from(dz),
        ]
    };
    Some([place(first, second), place(second, first)])
}

pub const EJECTOR_DEFAULTS: [i32; 4] = [0, 0, 1, 1];
pub const EJECTOR_LIMITS: [i32; 4] = [2, 2, 54, 5_346];

pub fn ejector_sources(facing: Direction) -> impl Iterator<Item = Direction> {
    let rear = facing.opposite();
    std::iter::once(rear).chain(
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
        .into_iter()
        .filter(move |side| *side != rear),
    )
}

pub fn ejector_plan(
    mut slots: Vec<(usize, u32)>,
    [mode, order, stacks, items]: [i32; 4],
    mut roll: impl FnMut(usize) -> usize,
) -> Vec<(usize, u32)> {
    slots.retain(|(_, count)| *count > 0);
    slots.sort_unstable_by_key(|(slot, _)| *slot);
    if mode == 2 {
        return slots;
    }
    match order {
        1 => slots.reverse(),
        2 => {
            for end in (1..slots.len()).rev() {
                let index = roll(end + 1) % (end + 1);
                slots.swap(end, index);
            }
        }
        _ => {}
    }
    if mode == 0 {
        slots.truncate(stacks.clamp(0, EJECTOR_LIMITS[2]) as usize);
        return slots;
    }
    let mut remaining = items.clamp(0, EJECTOR_LIMITS[3]) as u32;
    slots.retain_mut(|(_, count)| {
        *count = (*count).min(remaining);
        remaining -= *count;
        *count > 0
    });
    slots
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageFace {
    Down,
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageTarget {
    pub position: [i32; 3],
    pub face: StorageFace,
}

// These ordinary 1.7.10-era containers have a synchronous insert path in the
// pinned engine: no mutable Game callback can remove the source item between
// insert_item and set_item_entity. Do not broaden this without re-auditing
// view_changed/game events. The engine still owns slot/face/component rules.
pub fn supports_storage(block: &str) -> bool {
    matches!(
        block,
        "minecraft:chest"
            | "minecraft:trapped_chest"
            | "minecraft:furnace"
            | "minecraft:hopper"
            | "minecraft:dispenser"
            | "minecraft:dropper"
            | "minecraft:brewing_stand"
    )
}

pub fn storage_targets(
    belt: Belt,
    direction: Direction,
    block: [i32; 3],
    item: ItemMotion,
    powered: bool,
) -> Result<[Option<StorageTarget>; 4], Error> {
    // Reuse the tested contact guard, without changing any motion.
    velocity_for_tick(direction, block, item, Config::default())?;
    // Speedy unloading is an explicitly requested extension of the published
    // behavior. It shares the same targets, face rules and transfer budget.
    // The original Detector does not unload. Do not extend other variants
    // implicitly when adding their movement behavior.
    if matches!(belt, Belt::Detector | Belt::Ejector) {
        return Ok([None; 4]);
    }
    let [dx, _, dz] = direction.vector();
    let past_center = if dx != 0 {
        (item.position[0] - f64::from(block[0]) - 0.5) * f64::from(dx) >= 0.0
    } else {
        (item.position[2] - f64::from(block[2]) - 0.5) * f64::from(dz) >= 0.0
    };
    if !(past_center || belt == Belt::Brake && powered || belt == Belt::Separator) {
        return Ok([None; 4]);
    }
    let offset = |delta: [i32; 3], face| -> Result<_, Error> {
        let mut position = block;
        for axis in 0..3 {
            position[axis] = position[axis]
                .checked_add(delta[axis])
                .ok_or(Error::InvalidEntity)?;
        }
        support_block([
            f64::from(position[0]) + 0.5,
            f64::from(position[1]) + BELT_HEIGHT,
            f64::from(position[2]) + 0.5,
        ])?;
        Ok(Some(StorageTarget { position, face }))
    };
    let front_face = match direction {
        Direction::North => StorageFace::South,
        Direction::East => StorageFace::West,
        Direction::South => StorageFace::North,
        Direction::West => StorageFace::East,
    };
    let front = offset([dx, 0, dz], front_face)?;
    if belt == Belt::Separator {
        return Ok([front, None, None, None]);
    }
    let (first, second) = if dx != 0 {
        (
            offset([0, 0, -1], StorageFace::South)?,
            offset([0, 0, 1], StorageFace::North)?,
        )
    } else {
        (
            offset([1, 0, 0], StorageFace::West)?,
            offset([-1, 0, 0], StorageFace::East)?,
        )
    };
    Ok([front, first, second, offset([0, 1, 0], StorageFace::Down)?])
}

// One bounded budget shared by all destinations in this entity's Post.
// Even corrupt oversized world stacks cannot create an unbounded insert loop.
pub const MAX_TRANSFER_PER_TICK: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transfer {
    source: u32,
    moved: u32,
}

impl Transfer {
    pub fn new(source: u32) -> Self {
        Self { source, moved: 0 }
    }

    pub fn offer(self) -> u32 {
        self.remaining().min(MAX_TRANSFER_PER_TICK - self.moved)
    }

    pub fn accept(&mut self, returned: u32) -> Result<(), Error> {
        let accepted = self
            .offer()
            .checked_sub(returned)
            .ok_or(Error::InvalidTransfer)?;
        self.moved += accepted;
        Ok(())
    }

    pub fn remaining(self) -> u32 {
        self.source - self.moved
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DetectorUpdate {
    pub powered: bool,
    pub notify_neighbors: bool,
    pub click: bool,
}

pub fn detector_update(
    previous: bool,
    occupied: bool,
    enabled: bool,
    first_tick: bool,
) -> DetectorUpdate {
    let powered = enabled && occupied;
    let changed = previous != powered;
    DetectorUpdate {
        powered,
        notify_neighbors: first_tick || changed,
        click: changed,
    }
}

pub fn detector_signal(powered: bool) -> i32 {
    if powered { 15 } else { 0 }
}

pub fn detector_area(block: [i32; 3]) -> Result<([f64; 3], [f64; 3]), Error> {
    support_block([
        f64::from(block[0]) + 0.5,
        f64::from(block[1]) + BELT_HEIGHT,
        f64::from(block[2]) + 0.5,
    ])?;
    let min = block.map(f64::from);
    Ok((min, min.map(|n| n + 1.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIRECTIONS: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    fn resting(position: [f64; 3]) -> ItemMotion {
        ItemMotion {
            position,
            velocity: [0.0, -0.02, 0.0],
            on_ground: true,
        }
    }

    #[test]
    fn direction_state_round_trip_and_unknown_values() -> Result<(), Error> {
        for direction in DIRECTIONS {
            assert_eq!(Direction::try_from(direction.as_str())?, direction);
        }
        for invalid in ["", "North", "up", "south\0", "powercraft2:east"] {
            assert_eq!(Direction::try_from(invalid), Err(Error::UnknownDirection));
        }
        Ok(())
    }

    #[test]
    fn yaw_matches_minecraft_axes_and_wraps() -> Result<(), Error> {
        for (yaw, direction) in [
            (0.0, Direction::South),
            (90.0, Direction::West),
            (180.0, Direction::North),
            (270.0, Direction::East),
            (-90.0, Direction::East),
            (720.0, Direction::South),
            (45.0, Direction::West),
        ] {
            assert_eq!(Direction::from_yaw(yaw)?, direction);
        }
        assert!(Direction::from_yaw(f32::NAN).is_err());
        Ok(())
    }

    #[test]
    fn all_directions_preserve_vertical_motion() -> Result<(), Error> {
        let item = resting([0.5, 0.25, 0.5]);
        for direction in DIRECTIONS {
            let v = velocity_for_tick(direction, [0; 3], item, Config::default())?;
            let d = direction.vector();
            assert_eq!(v, [f64::from(d[0]) * 0.1, -0.02, f64::from(d[2]) * 0.1]);
        }
        Ok(())
    }

    #[test]
    fn centering_is_bounded_and_stops_at_center() -> Result<(), Error> {
        let left = velocity_for_tick(
            Direction::North,
            [0; 3],
            resting([0.1, 0.25, 0.5]),
            Config::default(),
        )?;
        assert_eq!(left[0], 0.04);
        let right = velocity_for_tick(
            Direction::North,
            [0; 3],
            resting([0.9, 0.25, 0.5]),
            Config::default(),
        )?;
        assert_eq!(right[0], -0.04);
        Ok(())
    }

    #[test]
    fn negative_coordinates_and_chunk_seams() -> Result<(), Error> {
        assert_eq!(support_block([-0.1, -63.75, -16.01])?, [-1, -64, -17]);
        assert_eq!(support_block([16.0, 64.25, 0.5])?, [16, 64, 0]);
        assert_eq!(support_block([0.5, 1.0, 0.5])?, [0, 0, 0]);
        Ok(())
    }

    #[test]
    fn invalid_world_data_and_airborne_items_are_rejected() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1.0e100] {
            assert!(support_block([value, 0.25, 0.5]).is_err());
            assert!(support_block([0.5, value, 0.5]).is_err());
        }
        let mut item = resting([0.5, 0.25, 0.5]);
        item.on_ground = false;
        assert_eq!(
            velocity_for_tick(Direction::East, [0; 3], item, Config::default()),
            Err(Error::NotOnSurface)
        );
        item.on_ground = true;
        item.velocity[0] = f64::NAN;
        assert_eq!(
            velocity_for_tick(Direction::East, [0; 3], item, Config::default()),
            Err(Error::InvalidEntity)
        );
    }

    #[test]
    fn foreign_support_and_wrong_surface_are_rejected() {
        for position in [[1.5, 0.25, 0.5], [0.5, 0.5, 0.5], [0.5, 0.0, 0.5]] {
            assert_eq!(
                velocity_for_tick(
                    Direction::East,
                    [0; 3],
                    resting(position),
                    Config::default()
                ),
                Err(Error::NotOnSurface)
            );
        }
    }

    #[test]
    fn configuration_rejects_nonfinite_and_unbounded_values() {
        for speed in [0.0, -0.1, 0.51, f64::NAN, f64::INFINITY] {
            assert!(Config::new(speed, 0.04).is_err());
        }
        for centering in [-0.1, 0.11, f64::NAN, f64::INFINITY] {
            assert!(Config::new(0.1, centering).is_err());
        }
        assert!(Config::new(0.5, 0.0).is_ok());
    }

    #[test]
    fn velocity_is_bounded_without_accumulation() -> Result<(), Error> {
        let mut item = resting([0.5, 0.25, 0.5]);
        item.velocity = [1000.0, -0.02, -1000.0];
        for _ in 0..10000 {
            item.velocity = velocity_for_tick(Direction::East, [0; 3], item, Config::default())?;
            assert_eq!(item.velocity, [0.1, -0.02, 0.0]);
        }
        Ok(())
    }

    #[test]
    fn a_line_and_a_turn_cross_block_and_chunk_boundaries() -> Result<(), Error> {
        let mut item = resting([14.5, 0.25, 0.5]);
        for _ in 0..40 {
            let block = support_block(item.position)?;
            item.velocity = velocity_for_tick(Direction::East, block, item, Config::default())?;
            item.position[0] += item.velocity[0];
        }
        assert!((item.position[0] - 18.5).abs() < 1.0e-10);
        let block = support_block(item.position)?;
        let turned = velocity_for_tick(Direction::South, block, item, Config::default())?;
        assert_eq!(turned[2], 0.1);
        assert!(turned[0].abs() <= 0.04);
        Ok(())
    }

    // Reduced flat-floor prediction fixture, not a replacement for Horizon's
    // collision tests: tick_move integrates, tick_settle applies f32 drag,
    // then the native Post restores belt velocity. Taken from the inspected
    // ItemEntity tick order; there is no position setter in production code.
    fn flat_physics_tick(item: &mut ItemMotion) {
        item.position[0] += item.velocity[0];
        item.position[2] += item.velocity[2];
        let friction = f64::from(0.98_f32 * 0.6_f32);
        item.velocity[0] *= friction;
        item.velocity[2] *= friction;
    }

    fn drive(item: &mut ItemMotion, direction: Direction) -> Result<(), Error> {
        let block = support_block(item.position)?;
        item.velocity = velocity_for_tick(direction, block, *item, Config::default())?;
        Ok(())
    }

    fn moving(direction: Direction) -> Result<ItemMotion, Error> {
        let mut item = resting([0.5, 0.25, 0.5]);
        item.velocity[1] = 0.0;
        drive(&mut item, direction)?;
        Ok(item)
    }

    #[test]
    fn server_only_drive_reproduces_drift_before_twenty_tick_correction() -> Result<(), Error> {
        let mut server = moving(Direction::East)?;
        let mut old_client = server;
        for _ in 0..20 {
            flat_physics_tick(&mut server);
            drive(&mut server, Direction::East)?;
            flat_physics_tick(&mut old_client);
        }
        // This is the original defect, not an expected new behavior: server
        // travels 2 blocks, ordinary client friction almost stops the copy.
        assert!((server.position[0] - 2.5).abs() < 1.0e-10);
        assert!(server.position[0] - old_client.position[0] > 1.7);
        assert!(old_client.velocity[0] < 0.00001);
        Ok(())
    }

    #[test]
    fn shared_post_drive_removes_drift_without_extra_position_packets() -> Result<(), Error> {
        for direction in DIRECTIONS {
            let mut server = moving(direction)?;
            let mut client = server;
            let d = direction.vector();
            for _ in 0..400 {
                let before = client.position;
                flat_physics_tick(&mut server);
                drive(&mut server, direction)?;
                flat_physics_tick(&mut client);
                drive(&mut client, direction)?;
                assert_eq!(client.position, server.position);
                assert_eq!(client.velocity, server.velocity);
                for axis in [0, 2] {
                    assert!(
                        (client.position[axis] - before[axis] - f64::from(d[axis]) * 0.1).abs()
                            < 1.0e-10
                    );
                }
                assert_eq!(client.velocity[1], 0.0);
            }
        }
        Ok(())
    }

    #[test]
    fn normal_packet_rounding_stays_small_between_updates() -> Result<(), Error> {
        let mut server = moving(Direction::East)?;
        let mut client = server;
        for tick in 1..=400 {
            flat_physics_tick(&mut server);
            drive(&mut server, Direction::East)?;
            flat_physics_tick(&mut client);
            drive(&mut client, Direction::East)?;
            if tick % 20 == 0 {
                // Bound the standard 1/4096 coordinate encoding error. The
                // fix does not introduce a packet or a teleport of its own.
                let correction = (server.position[0] * 4096.0).round() / 4096.0;
                assert!((client.position[0] - correction).abs() <= 1.0 / 4096.0);
                client.position[0] = correction;
            }
            assert!((server.position[0] - client.position[0]).abs() <= 1.0 / 4096.0);
        }
        Ok(())
    }

    #[test]
    fn low_fps_and_catch_up_batches_keep_the_same_game_tick_trajectory() -> Result<(), Error> {
        for fps in [5, 17, 30, 60, 144] {
            let mut item = moving(Direction::East)?;
            let mut elapsed_ticks = 0;
            for frame in 1..=fps * 10 {
                let due = frame * 20 / fps;
                while elapsed_ticks < due {
                    flat_physics_tick(&mut item);
                    drive(&mut item, Direction::East)?;
                    elapsed_ticks += 1;
                }
            }
            assert_eq!(elapsed_ticks, 200);
            assert!((item.position[0] - 20.5).abs() < 1.0e-10);
        }
        Ok(())
    }

    #[test]
    fn many_items_have_the_same_per_tick_displacement() -> Result<(), Error> {
        for count in [1, 256, 4096] {
            let mut items = vec![moving(Direction::East)?; count];
            for _ in 0..200 {
                for item in &mut items {
                    flat_physics_tick(item);
                    drive(item, Direction::East)?;
                }
            }
            for item in items {
                assert!((item.position[0] - 20.5).abs() < 1.0e-10);
            }
        }
        Ok(())
    }

    #[test]
    fn prediction_during_tick_freeze_cannot_move_a_position() -> Result<(), Error> {
        let mut item = moving(Direction::East)?;
        let frozen = item.position;
        for _ in 0..100 {
            // The window may tick while the world is frozen; no physics.
            drive(&mut item, Direction::East)?;
        }
        assert_eq!(item.position, frozen);
        flat_physics_tick(&mut item);
        assert!((item.position[0] - frozen[0] - 0.1).abs() < 1.0e-10);
        Ok(())
    }

    #[test]
    fn both_adapters_project_the_same_chunk_boundary_without_moving_y() -> Result<(), Error> {
        assert_eq!(
            next_support_block([15.95, 64.25, 0.5], [0.1, 50.0, 0.0])?,
            [16, 64, 0]
        );
        assert_eq!(
            next_support_block([-16.0, -63.75, 0.5], [-0.1, -50.0, 0.0])?,
            [-17, -64, 0]
        );
        assert!(next_support_block([0.5, 0.25, 0.5], [f64::NAN, 0.0, 0.0]).is_err());
        Ok(())
    }

    #[test]
    fn variants_accept_only_our_registered_block_ids() {
        assert_eq!(Belt::from_block(CONVEYOR), Some(Belt::Normal));
        assert_eq!(Belt::from_block(SPEEDY_CONVEYOR), Some(Belt::Speedy));
        assert_eq!(Belt::from_block(BRAKE_CONVEYOR), Some(Belt::Brake));
        for foreign in [
            "minecraft:conveyor",
            "other:speedy_conveyor",
            "",
            "powercraft2:conveyor_extra",
        ] {
            assert_eq!(Belt::from_block(foreign), None);
        }
    }

    #[test]
    fn normal_variant_preserves_tested_calculation_exactly() -> Result<(), Error> {
        for x in [-16.01, -0.01, 0.0, 0.1, 0.5, 0.99, 16.0] {
            for direction in DIRECTIONS {
                for powered in [false, true] {
                    let item = resting([x, 0.25, 0.4]);
                    let block = support_block(item.position)?;
                    assert_eq!(
                        belt_velocity_for_tick(
                            Belt::Normal,
                            direction,
                            block,
                            item,
                            Config::default(),
                            powered
                        )?,
                        velocity_for_tick(direction, block, item, Config::default())?
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn speedy_and_unpowered_brake_match_original_speed_ratios() -> Result<(), Error> {
        for direction in DIRECTIONS {
            let item = resting([0.5, 0.25, 0.5]);
            let d = direction.vector();
            for (belt, ratio) in [(Belt::Speedy, 2.0), (Belt::Brake, 0.6)] {
                let v = belt_velocity_for_tick(
                    belt,
                    direction,
                    [0; 3],
                    item,
                    Config::default(),
                    false,
                )?;
                assert_eq!(
                    v,
                    [
                        f64::from(d[0]) * 0.1 * ratio,
                        item.velocity[1],
                        f64::from(d[2]) * 0.1 * ratio
                    ]
                );
            }
        }
        Ok(())
    }

    #[test]
    fn powered_brake_damps_both_axes_without_belt_thrust_or_vertical_changes() -> Result<(), Error>
    {
        let mut item = resting([0.5, 0.25, 0.5]);
        item.velocity = [-0.2, -0.03, 0.1];
        for direction in DIRECTIONS {
            assert_eq!(
                belt_velocity_for_tick(
                    Belt::Brake,
                    direction,
                    [0; 3],
                    item,
                    Config::default(),
                    true
                )?,
                [-0.12, -0.03, 0.06]
            );
        }
        item.velocity = [0.0; 3];
        assert_eq!(
            belt_velocity_for_tick(
                Belt::Brake,
                Direction::East,
                [0; 3],
                item,
                Config::default(),
                true
            )?,
            [0.0; 3]
        );
        Ok(())
    }

    #[test]
    fn powered_brake_stops_incoming_item_and_restarts_after_signal_removal() -> Result<(), Error> {
        let mut item = moving(Direction::East)?;
        let start = item.position[0];
        for _ in 0..100 {
            flat_physics_tick(&mut item);
            let block = support_block(item.position)?;
            item.velocity = belt_velocity_for_tick(
                Belt::Brake,
                Direction::East,
                block,
                item,
                Config::default(),
                true,
            )?;
        }
        assert!(item.velocity[0].abs() < 1.0e-20);
        assert!(item.position[0] - start > 0.1 && item.position[0] - start < 0.16);
        let block = support_block(item.position)?;
        item.velocity = belt_velocity_for_tick(
            Belt::Brake,
            Direction::East,
            block,
            item,
            Config::default(),
            false,
        )?;
        let before = item.position[0];
        flat_physics_tick(&mut item);
        assert!((item.position[0] - before - 0.06).abs() < 1.0e-10);
        Ok(())
    }

    #[test]
    fn variants_share_contact_validation_and_reject_corrupt_motion() {
        for belt in [Belt::Normal, Belt::Speedy, Belt::Brake] {
            for powered in [false, true] {
                let mut item = resting([0.5, 0.25, 0.5]);
                item.on_ground = false;
                assert_eq!(
                    belt_velocity_for_tick(
                        belt,
                        Direction::East,
                        [0; 3],
                        item,
                        Config::default(),
                        powered
                    ),
                    Err(Error::NotOnSurface)
                );
                item.on_ground = true;
                item.velocity[2] = f64::NAN;
                assert_eq!(
                    belt_velocity_for_tick(
                        belt,
                        Direction::East,
                        [0; 3],
                        item,
                        Config::default(),
                        powered
                    ),
                    Err(Error::InvalidEntity)
                );
            }
        }
    }

    #[test]
    fn variants_predict_identically_through_power_transitions_and_tick_batches() -> Result<(), Error>
    {
        for belt in [Belt::Normal, Belt::Speedy, Belt::Brake] {
            for direction in DIRECTIONS {
                for fps in [5, 17, 30, 60, 144] {
                    let mut server = moving(direction)?;
                    let mut client = server;
                    let mut tick = 0;
                    for frame in 1..=fps * 10 {
                        while tick < frame * 20 / fps {
                            let powered = (40..120).contains(&tick);
                            for item in [&mut server, &mut client] {
                                flat_physics_tick(item);
                                let block = support_block(item.position)?;
                                item.velocity = belt_velocity_for_tick(
                                    belt,
                                    direction,
                                    block,
                                    *item,
                                    Config::default(),
                                    powered,
                                )?;
                            }
                            assert_eq!(server.position, client.position);
                            assert_eq!(server.velocity, client.velocity);
                            tick += 1;
                        }
                    }
                    assert_eq!(tick, 200);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn mixed_line_changes_speed_without_position_setters() -> Result<(), Error> {
        let mut item = moving(Direction::East)?;
        let mut seen = [false; 3];
        for _ in 0..150 {
            flat_physics_tick(&mut item);
            let block = support_block(item.position)?;
            let (index, belt, speed) = match block[0] {
                i32::MIN..=3 => (0, Belt::Normal, 0.1),
                4..=7 => (1, Belt::Speedy, 0.2),
                _ => (2, Belt::Brake, 0.06),
            };
            seen[index] = true;
            item.velocity = belt_velocity_for_tick(
                belt,
                Direction::East,
                block,
                item,
                Config::default(),
                false,
            )?;
            assert_eq!(item.velocity[0], speed);
        }
        assert_eq!(seen, [true; 3]);
        Ok(())
    }

    #[test]
    fn storage_priority_and_receiver_faces_match_original_in_all_directions() -> Result<(), Error> {
        let at = resting([0.5, 0.25, 0.5]);
        for (direction, front, face, side1, side2) in [
            (
                Direction::North,
                [0, 0, -1],
                StorageFace::South,
                [1, 0, 0],
                [-1, 0, 0],
            ),
            (
                Direction::East,
                [1, 0, 0],
                StorageFace::West,
                [0, 0, -1],
                [0, 0, 1],
            ),
            (
                Direction::South,
                [0, 0, 1],
                StorageFace::North,
                [1, 0, 0],
                [-1, 0, 0],
            ),
            (
                Direction::West,
                [-1, 0, 0],
                StorageFace::East,
                [0, 0, -1],
                [0, 0, 1],
            ),
        ] {
            let targets = storage_targets(Belt::Normal, direction, [0; 3], at, false)?;
            assert_eq!(
                targets[0],
                Some(StorageTarget {
                    position: front,
                    face
                })
            );
            assert_eq!(targets[1].map(|t| t.position), Some(side1));
            assert_eq!(targets[2].map(|t| t.position), Some(side2));
            assert_eq!(
                targets[3],
                Some(StorageTarget {
                    position: [0, 1, 0],
                    face: StorageFace::Down
                })
            );
            assert!(targets.into_iter().flatten().all(|t| t.position[1] >= 0));
        }
        Ok(())
    }

    #[test]
    fn storage_starts_at_center_not_at_entry_or_back_edge() -> Result<(), Error> {
        for direction in DIRECTIONS {
            let d = direction.vector();
            for advance in [-0.1, 0.0, 0.1] {
                let item = resting([
                    0.5 + f64::from(d[0]) * advance,
                    0.25,
                    0.5 + f64::from(d[2]) * advance,
                ]);
                let targets = storage_targets(Belt::Normal, direction, [0; 3], item, false)?;
                assert_eq!(targets.iter().any(Option::is_some), advance >= 0.0);
            }
        }
        Ok(())
    }

    #[test]
    fn only_powered_brake_ignores_storage_border() -> Result<(), Error> {
        let item = resting([0.1, 0.25, 0.5]);
        assert_eq!(
            storage_targets(Belt::Brake, Direction::East, [0; 3], item, false)?,
            [None; 4]
        );
        assert!(storage_targets(Belt::Brake, Direction::East, [0; 3], item, true)?[0].is_some());
        assert_eq!(
            storage_targets(Belt::Normal, Direction::East, [0; 3], item, true)?,
            [None; 4]
        );
        Ok(())
    }

    #[test]
    fn speedy_unloads_like_normal_without_changing_speed_or_budget() -> Result<(), Error> {
        for direction in DIRECTIONS {
            for powered in [false, true] {
                assert_eq!(
                    storage_targets(
                        Belt::Speedy,
                        direction,
                        [0; 3],
                        resting([0.5, 0.25, 0.5]),
                        powered
                    )?,
                    storage_targets(
                        Belt::Normal,
                        direction,
                        [0; 3],
                        resting([0.5, 0.25, 0.5]),
                        powered
                    )?
                );
            }
        }
        Ok(())
    }

    #[test]
    fn storage_never_takes_airborne_or_invalid_entities() {
        for belt in [Belt::Normal, Belt::Brake] {
            let mut item = resting([0.5, 0.25, 0.5]);
            item.on_ground = false;
            assert_eq!(
                storage_targets(belt, Direction::East, [0; 3], item, true),
                Err(Error::NotOnSurface)
            );
            item.on_ground = true;
            item.velocity[0] = f64::NAN;
            assert_eq!(
                storage_targets(belt, Direction::East, [0; 3], item, true),
                Err(Error::InvalidEntity)
            );
        }
    }

    #[test]
    fn storage_coordinates_cross_negative_chunk_seams_safely() -> Result<(), Error> {
        let item = resting([-16.5, -63.75, -0.5]);
        let targets = storage_targets(Belt::Normal, Direction::West, [-17, -64, -1], item, false)?;
        assert_eq!(
            targets[0],
            Some(StorageTarget {
                position: [-18, -64, -1],
                face: StorageFace::East
            })
        );
        assert!(
            storage_targets(
                Belt::Normal,
                Direction::East,
                [i32::MAX, 0, 0],
                resting([0.5, 0.25, 0.5]),
                false
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn only_audited_container_paths_are_enabled() {
        for name in [
            "minecraft:chest",
            "minecraft:trapped_chest",
            "minecraft:furnace",
            "minecraft:hopper",
            "minecraft:dispenser",
            "minecraft:dropper",
            "minecraft:brewing_stand",
        ] {
            assert!(supports_storage(name));
        }
        for name in [
            "other:chest",
            "minecraft:composter",
            "minecraft:jukebox",
            "minecraft:oak_shelf",
            "minecraft:barrel",
            "minecraft:air",
            "powercraft2:conveyor",
        ] {
            assert!(!supports_storage(name));
        }
    }

    #[test]
    fn refused_and_partial_targets_keep_exact_remainder() -> Result<(), Error> {
        let mut transfer = Transfer::new(100);
        transfer.accept(64)?;
        assert_eq!(transfer.remaining(), 100);
        transfer.accept(10)?;
        assert_eq!(transfer.remaining(), 46);
        assert_eq!(transfer.offer(), 10);
        transfer.accept(0)?;
        assert_eq!(transfer.remaining(), 36);
        assert_eq!(transfer.offer(), 0);
        Ok(())
    }

    #[test]
    fn accepted_counts_are_conserved_across_three_destinations() -> Result<(), Error> {
        for count in 0..=255 {
            for first in [0, 1, 16, 64, 255] {
                for second in [0, 1, 16, 64, 255] {
                    for third in [0, 1, 16, 64, 255] {
                        let mut transfer = Transfer::new(count);
                        let mut stored = 0;
                        for capacity in [first, second, third] {
                            let offer = transfer.offer();
                            let accepted = offer.min(capacity);
                            transfer.accept(offer - accepted)?;
                            stored += accepted;
                            assert_eq!(count, stored + transfer.remaining());
                            assert!(stored <= MAX_TRANSFER_PER_TICK);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn corrupt_huge_stacks_have_a_bounded_transfer_budget() -> Result<(), Error> {
        let mut transfer = Transfer::new(u32::MAX);
        assert_eq!(transfer.offer(), 64);
        transfer.accept(0)?;
        assert_eq!(transfer.remaining(), u32::MAX - 64);
        assert_eq!(transfer.offer(), 0);
        Ok(())
    }

    #[test]
    fn invalid_backend_remainder_is_rejected_without_changing_accounting() {
        let mut transfer = Transfer::new(16);
        let before = transfer;
        assert_eq!(transfer.accept(17), Err(Error::InvalidTransfer));
        assert_eq!(transfer, before);
    }

    #[test]
    fn oversized_stack_retries_in_later_ticks_without_extra_items() -> Result<(), Error> {
        let mut remaining = 150;
        let mut stored = 0;
        for expected in [86, 22, 0] {
            let mut transfer = Transfer::new(remaining);
            stored += transfer.offer();
            transfer.accept(0)?;
            remaining = transfer.remaining();
            assert_eq!(remaining, expected);
            assert_eq!(remaining + stored, 150);
        }
        Ok(())
    }

    #[test]
    fn ejector_prefers_the_rear_then_checks_other_sides_once() {
        for facing in DIRECTIONS {
            let mut sides = ejector_sources(facing);
            assert_eq!(sides.next(), Some(facing.opposite()));
            assert!(
                sides.eq(DIRECTIONS
                    .into_iter()
                    .filter(|side| *side != facing.opposite()))
            );
        }
    }

    #[test]
    fn ejector_selects_whole_stacks_or_an_exact_item_budget() {
        let slots = vec![(5, 8), (0, 3), (3, 0), (2, 12)];
        assert_eq!(
            ejector_plan(slots.clone(), [0, 0, 2, 1], |_| 0),
            [(0, 3), (2, 12)]
        );
        assert_eq!(
            ejector_plan(slots.clone(), [1, 0, 1, 5], |_| 0),
            [(0, 3), (2, 2)]
        );
        assert_eq!(
            ejector_plan(slots.clone(), [1, 1, 1, 10], |_| 0),
            [(5, 8), (2, 2)]
        );
        assert!(ejector_plan(slots, [1, 2, 1, 0], |_| 0).is_empty());
    }

    #[test]
    fn ejector_random_order_does_not_repeat_slots_and_all_ignores_order() {
        let slots = vec![(3, 2), (1, 16), (2, 5)];
        let random = ejector_plan(slots.clone(), [0, 2, 3, 1], |_| 0);
        assert_eq!(random, [(2, 5), (3, 2), (1, 16)]);
        let all = ejector_plan(slots, [2, 2, 0, 0], |_| {
            panic!("all must not sample randomness")
        });
        assert_eq!(all, [(1, 16), (2, 5), (3, 2)]);
    }

    #[test]
    fn ejector_moves_like_a_normal_belt_without_unloading() -> Result<(), Error> {
        let item = resting([0.5, 0.25, 0.5]);
        let block = [0, 0, 0];
        let facing = Direction::East;
        assert_eq!(
            belt_velocity_for_tick(Belt::Ejector, facing, block, item, Config::default(), true)?,
            velocity_for_tick(facing, block, item, Config::default())?
        );
        assert_eq!(
            storage_targets(Belt::Ejector, facing, block, item, true)?,
            [None; 4]
        );
        Ok(())
    }

    #[test]
    fn sorter_routes_unmatched_and_single_bank_items() {
        let facing = Direction::East;
        for (weights, expected) in [
            ([0, 0], Direction::East),
            ([3, 0], Direction::North),
            ([0, 4], Direction::South),
        ] {
            let [output, empty] = separator_outputs(facing, 16, weights, |_| false, |_| 0);
            assert_eq!(
                output,
                SortOutput {
                    direction: expected,
                    count: 16
                }
            );
            assert_eq!(empty.count, 0);
        }
        assert_eq!(
            separator_weights((0..20).map(|slot| matches!(slot, 0 | 8 | 9 | 18 | 19))),
            [2, 1]
        );
    }

    #[test]
    fn sorter_preserves_stack_count_and_uses_front_if_one_side_has_no_belt() {
        for facing in DIRECTIONS {
            let side = facing.right();
            for count in 2..=99 {
                for left in 1..=9 {
                    for right in 1..=9 {
                        let [first, second] =
                            separator_outputs(facing, count, [left, right], |d| d == side, |_| 0);
                        assert_eq!(first.direction, facing);
                        assert_eq!(second.direction, side);
                        assert_eq!(
                            first.count,
                            (count * left + (left + right) / 2) / (left + right)
                        );
                        assert_eq!(first.count + second.count, count);
                    }
                }
            }
            let [left, right] = separator_outputs(facing, u32::MAX, [9, 9], |_| false, |_| 0);
            assert_eq!(left.direction, facing.left());
            assert_eq!(right.direction, side);
            assert_eq!(left.count + right.count, u32::MAX);
        }
    }

    #[test]
    fn sorter_single_item_has_weighted_odds_without_creating_a_second_stack() {
        let facing = Direction::East;
        for roll in 0..5 {
            let [first, second] = separator_outputs(
                facing,
                1,
                [2, 3],
                |_| true,
                |total| {
                    assert_eq!(total, 5);
                    roll
                },
            );
            assert_eq!(
                first.direction,
                if roll < 2 {
                    Direction::North
                } else {
                    Direction::South
                }
            );
            assert_eq!((first.count, second.count), (1, 0));
        }
    }

    #[test]
    fn sorter_split_stays_on_the_belt_outside_vanilla_merge_reach() -> Result<(), Error> {
        for facing in DIRECTIONS {
            for (first, second) in [
                (facing.left(), facing.right()),
                (facing, facing.right()),
                (facing.left(), facing),
            ] {
                let block = [-16, 1, 15];
                let [a, b] = separator_split_positions(block, facing, first, second)
                    .ok_or(Error::InvalidEntity)?;
                assert_eq!(support_block(a)?, block);
                assert_eq!(support_block(b)?, block);
                assert!((a[0] - b[0]).abs().max((a[2] - b[2]).abs()) > 0.75);
            }
        }
        Ok(())
    }

    #[test]
    fn sorter_targets_only_the_chosen_container_without_waiting_for_center() -> Result<(), Error> {
        let block = [-16, 1, 15];
        let targets = storage_targets(
            Belt::Separator,
            Direction::South,
            block,
            resting([-15.8, 1.25, 15.2]),
            false,
        )?;
        assert_eq!(
            targets,
            [
                Some(StorageTarget {
                    position: [-16, 1, 16],
                    face: StorageFace::North
                }),
                None,
                None,
                None,
            ]
        );
        Ok(())
    }

    #[test]
    fn redirector_routes_like_the_published_belt_for_all_facings() {
        use RedirectNeighbor::{Air, Belt, Obstructed};

        for (index, facing) in DIRECTIONS.into_iter().enumerate() {
            let left = DIRECTIONS[(index + 3) % 4];
            let right = DIRECTIONS[(index + 1) % 4];
            let behind = DIRECTIONS[(index + 2) % 4];
            // The published metadata table is in south/west/north/east order.
            // Its preferred powered exits are west/north/east/south.
            for (powered, sides, expected) in [
                (false, [Air, Belt, Belt], left),
                (true, [Air, Belt, Belt], right),
                (false, [Belt, Belt, Belt], facing),
                (false, [Obstructed, Air, Air], facing),
                (true, [Belt, Air, Obstructed], left),
                (true, [Belt, Obstructed, Obstructed], facing),
                (true, [Belt, Air, Belt], right),
            ] {
                assert_eq!(
                    redirect_direction(facing, powered, |side| {
                        if side == behind {
                            sides[0]
                        } else if side == left {
                            sides[1]
                        } else {
                            assert_eq!(side, right);
                            sides[2]
                        }
                    }),
                    expected,
                    "{facing:?}, {powered}, {sides:?}"
                );
            }
        }
        assert_eq!(RedirectNeighbor::from_block(None), Obstructed);
        assert_eq!(
            RedirectNeighbor::from_block(Some("minecraft:stone")),
            Obstructed
        );
        assert_eq!(
            RedirectNeighbor::from_block(Some("minecraft:cave_air")),
            Air
        );
        assert_eq!(
            RedirectNeighbor::from_block(Some(REDIRECTOR_CONVEYOR)),
            Belt
        );
    }

    #[test]
    fn redirector_uses_normal_speed_and_stores_by_placed_facing() -> Result<(), Error> {
        assert_eq!(
            Belt::from_block(REDIRECTOR_CONVEYOR),
            Some(Belt::Redirector)
        );
        let block = [-16, 1, 15];
        let item = resting([-15.5, 1.25, 15.5]);
        let route = redirect_direction(Direction::East, true, |_| RedirectNeighbor::Air);
        assert_eq!(route, Direction::South);
        let velocity = belt_velocity_for_tick(
            Belt::Redirector,
            route,
            block,
            item,
            Config::default(),
            true,
        )?;
        assert_eq!(velocity, [0.0, -0.02, DEFAULT_SPEED]);
        assert_eq!(
            storage_targets(Belt::Redirector, Direction::East, block, item, true)?[0],
            Some(StorageTarget {
                position: [-15, 1, 15],
                face: StorageFace::West
            })
        );
        let early = resting([-15.8, 1.25, 15.5]);
        assert_eq!(
            storage_targets(Belt::Redirector, Direction::East, block, early, true)?,
            [None; 4]
        );
        Ok(())
    }

    #[test]
    fn detector_registers_its_own_id_and_preserves_normal_motion() -> Result<(), Error> {
        assert_eq!(Belt::from_block(DETECTOR_CONVEYOR), Some(Belt::Detector));
        for direction in DIRECTIONS {
            let item = resting([0.4, 0.25, 0.6]);
            assert_eq!(
                belt_velocity_for_tick(
                    Belt::Detector,
                    direction,
                    [0; 3],
                    item,
                    Config::default(),
                    true
                )?,
                velocity_for_tick(direction, [0; 3], item, Config::default())?
            );
        }
        Ok(())
    }

    #[test]
    fn detector_does_not_implicitly_gain_unloading() -> Result<(), Error> {
        for direction in DIRECTIONS {
            for powered in [false, true] {
                assert_eq!(
                    storage_targets(
                        Belt::Detector,
                        direction,
                        [0; 3],
                        resting([0.5, 0.25, 0.5]),
                        powered
                    )?,
                    [None; 4]
                );
            }
        }
        Ok(())
    }

    #[test]
    fn detector_area_is_the_original_full_block_cube() -> Result<(), Error> {
        assert_eq!(
            detector_area([2, 64, 15])?,
            ([2.0, 64.0, 15.0], [3.0, 65.0, 16.0])
        );
        assert_eq!(
            detector_area([-17, -64, -1])?,
            ([-17.0, -64.0, -1.0], [-16.0, -63.0, 0.0])
        );
        assert!(detector_area([i32::MAX, 0, 0]).is_err());
        assert!(detector_area([0, i32::MIN, 0]).is_err());
        Ok(())
    }

    #[test]
    fn detector_emits_fifteen_then_resets_after_last_entity() {
        let mut powered = false;
        for (count, expected, click) in [(0, 0, false), (2, 15, true), (1, 15, false), (0, 0, true)]
        {
            let update = detector_update(powered, count > 0, true, false);
            assert_eq!(detector_signal(update.powered), expected);
            assert_eq!(update.notify_neighbors, click);
            assert_eq!(update.click, click);
            powered = update.powered;
        }
    }

    #[test]
    fn loaded_detector_notifies_once_without_false_click_when_unchanged() {
        for powered in [false, true] {
            let first = detector_update(powered, powered, true, true);
            assert!(first.notify_neighbors);
            assert!(!first.click);
            let next = detector_update(first.powered, powered, true, false);
            assert!(!next.notify_neighbors);
            assert!(!next.click);
        }
    }

    #[test]
    fn saved_power_is_recomputed_after_loading_an_empty_chunk() {
        let update = detector_update(true, false, true, true);
        assert!(!update.powered);
        assert!(update.notify_neighbors);
        assert!(update.click);
        assert_eq!(detector_signal(update.powered), 0);
    }

    #[test]
    fn disabling_detector_clears_output_even_while_occupied() {
        let update = detector_update(true, true, false, false);
        assert!(!update.powered);
        assert!(update.notify_neighbors);
        let again = detector_update(update.powered, true, false, false);
        assert!(!again.notify_neighbors);
        assert!(!again.click);
    }

    #[test]
    fn stable_detector_does_not_repeat_neighbor_updates_or_sounds() {
        for powered in [false, true] {
            for _ in 0..1000 {
                let update = detector_update(powered, powered, true, false);
                assert_eq!(update.powered, powered);
                assert!(!update.notify_neighbors);
                assert!(!update.click);
            }
        }
    }
}
