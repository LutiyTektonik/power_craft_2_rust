use super::{Game, powercraft2_transport as transport};
use horizon_client::mod_api::Host;
use horizon_core::geometry::Vec3;
use transport::{Belt, Direction, Elevator, ElevatorCell, ItemMotion, RedirectNeighbor};

fn player_motion(game: &Game<'_>) -> Option<ItemMotion> {
    let player = game.player.as_ref()?;
    let feet = game.armor[3].as_ref().map(|stack| stack.item.as_ref());
    if !transport::player_rides(feet, player.shift_key_down)
        || player.flying
        || player.fall_flying
        || player.is_passenger
        || player.spectator
        || player.sleeping
        || player.swimming
        || player.health.health() <= 0.0
    {
        return None;
    }
    Some(ItemMotion {
        position: [player.position.x, player.position.y, player.position.z],
        velocity: [player.velocity.x, player.velocity.y, player.velocity.z],
        on_ground: player.ground.on_ground,
    })
}

pub(super) fn drive_local(game: &mut Game<'_>) {
    let Some(motion) = player_motion(game) else {
        return;
    };
    let world = &game.world;
    let registry = &game.pack.assets.registry;
    let state_at = |[x, y, z]: [i32; 3]| {
        if !world.loaded(x, z) {
            return None;
        }
        registry.state(world.block(x, y, z))
    };
    let Ok(occupied) = transport::occupied_block(motion.position) else {
        return;
    };
    let powered = |state: &horizon_world::chunk::BlockState| {
        state
            .properties
            .get("powered")
            .is_some_and(|value| value == "true")
    };
    let velocity = if let Some(state) = state_at(occupied)
        && let Some(elevator) = Elevator::from_block(&state.name)
    {
        transport::player_elevator_velocity(elevator, occupied, motion, powered(state), |at| {
            state_at(at).map_or(ElevatorCell::Other, |state| {
                ElevatorCell::from_block(
                    &state.name,
                    state.properties.get("facing").map(String::as_str),
                    elevator,
                )
            })
        })
    } else if state_at(occupied).is_some_and(|state| state.name == transport::SPLITTER) {
        // The source gives a player, which has no filter stack, the upward output.
        // The shared column solver still checks the receiving cells and collision is vanilla.
        transport::player_elevator_velocity(Elevator::Up, occupied, motion, false, |at| {
            state_at(at).map_or(ElevatorCell::Other, |state| match ElevatorCell::from_block(
                &state.name,
                None,
                Elevator::Up,
            ) {
                ElevatorCell::Air => ElevatorCell::Elevator,
                cell => cell,
            })
        })
    } else {
        let Ok(block) = transport::support_block(motion.position) else {
            return;
        };
        let Some(state) = state_at(block) else {
            return;
        };
        let (Some(belt), Some(facing)) = (
            Belt::from_block(&state.name),
            state
                .properties
                .get("facing")
                .and_then(|facing| Direction::try_from(facing.as_str()).ok()),
        ) else {
            return;
        };
        let powered = powered(state);
        let direction = if belt == Belt::Redirector {
            transport::redirect_direction(facing, powered, |direction| {
                RedirectNeighbor::from_block(
                    state_at(direction.neighbor(block)).map(|state| state.name.as_str()),
                )
            })
        } else {
            facing
        };
        transport::player_belt_velocity(belt, direction, block, motion, powered)
    };
    if let Ok([vx, vy, vz]) = velocity
        && let Some(player) = game.player.as_mut()
    {
        player.velocity = Vec3::new(vx, vy, vz);
    }
}

pub(super) fn publish_power(game: &mut Game<'_>) {
    if !game.native_on(transport::MOD_ID) {
        return;
    }
    let Some(motion) = player_motion(game) else {
        return;
    };
    let Ok(occupied) = transport::occupied_block(motion.position) else {
        return;
    };
    let Ok(support) = transport::support_block(motion.position) else {
        return;
    };
    let registry = &game.pack.assets.registry;
    let state_at = |[x, y, z]: [i32; 3]| registry.state(game.world.block(x, y, z));
    let block =
        if state_at(occupied).is_some_and(|state| Elevator::from_block(&state.name).is_some()) {
            occupied
        } else if motion.on_ground
            && state_at(support).is_some_and(|state| {
                matches!(
                    Belt::from_block(&state.name),
                    Some(Belt::Brake | Belt::Redirector)
                )
            })
        {
            support
        } else {
            return;
        };
    let Some(state) = state_at(block) else {
        return;
    };
    let Some(previous) = state.properties.get("powered").map(|value| value == "true") else {
        return;
    };
    let host: &mut dyn Host = game;
    let powered = host.neighbor_signal(host.dimension(), block) > 0;
    if powered != previous {
        let value = if powered { "true" } else { "false" };
        let _changed = host.set_block_property_with(block, "powered", value, 2);
    }
}
