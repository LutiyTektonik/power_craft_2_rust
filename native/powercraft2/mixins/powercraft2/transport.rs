// Shared by the standalone Cargo library and Horizon's injected module.
// No engine types, clocks, global state, IO, or allocations in the movement path.

pub const MOD_ID: &str = "powercraft2";
pub const CONVEYOR: &str = "powercraft2:conveyor";
pub const BELT_HEIGHT: f64 = 0.25;
const CONTACT_EPSILON: f64 = 1.0 / 64.0;
const WORLD_BORDER: f64 = 30_000_000.0;

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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UnknownDirection => "unknown conveyor direction",
            Self::InvalidConfiguration => "invalid conveyor configuration",
            Self::InvalidEntity => "invalid entity coordinates or velocity",
            Self::NotOnSurface => "item is not resting on the conveyor surface",
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
            speed: 0.1,
            centering: 0.04,
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
    if sampled != block
        || !item.on_ground
        || (item.position[1] - surface).abs() > CONTACT_EPSILON
    {
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
                velocity_for_tick(Direction::East, [0; 3], resting(position), Config::default()),
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

    #[test]
    fn render_frames_do_not_change_tick_results_and_reload_needs_no_cache() -> Result<(), Error> {
        let simulate = |frames: usize| -> Result<[f64; 3], Error> {
            let mut item = resting([0.5, 0.25, 0.5]);
            for _ in 0..100 {
                for _ in 0..frames {
                    // Rendering only reads the authoritative position.
                    std::hint::black_box(item.position);
                }
                let saved_facing = Direction::East.as_str();
                let loaded_direction = Direction::try_from(saved_facing)?;
                let block = support_block(item.position)?;
                item.velocity =
                    velocity_for_tick(loaded_direction, block, item, Config::default())?;
                item.position[0] += item.velocity[0];
            }
            Ok(item.position)
        };
        assert_eq!(simulate(0)?, simulate(12)?);
        Ok(())
    }
}
