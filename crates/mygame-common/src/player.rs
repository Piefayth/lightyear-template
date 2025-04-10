use avian3d::prelude::{Collider, LinearVelocity, RigidBody, collider};
use bevy::{gltf::GltfMesh, prelude::*};
use leafwing_input_manager::prelude::{ActionState, InputMap, VirtualDPad};
use lightyear::prelude::{
    client::{Confirmed, Interpolated, Predicted},
    server::ReplicationTarget,
};
use mygame_assets::{LevelState, assets::GlobalAssets};
use mygame_protocol::{component::Player, input::NetworkedInput};

use crate::{Rendered, Simulated};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (add_player_gameplay_components).run_if(in_state(LevelState::Loaded)),
        );

        app.add_systems(FixedUpdate, move_player);
    }
}

fn add_player_gameplay_components(
    mut commands: Commands,
    q_rendered_player: Query<Entity, (Rendered, Without<RigidBody>, With<Player>)>,
    global_assets: Res<GlobalAssets>,
) {
    if q_rendered_player.is_empty() {
        return;
    }

    for player_entity in &q_rendered_player {
        commands.entity(player_entity).insert((
            RigidBody::Kinematic,
            Collider::capsule(3.0, 4.0),
            SceneRoot(global_assets.character.clone()),
        ));
    }
}

const PLAYER_MOVE_SPEED: f32 = 30.0;

fn move_player(
    mut q_player: Query<
        (&ActionState<NetworkedInput>, &mut LinearVelocity),
        (Simulated, With<Player>),
    >,
) {
    for (action_state, mut velocity) in q_player.iter_mut() {
        if let Some(movement) = action_state.dual_axis_data(&NetworkedInput::Move) {
            let move_vec = Vec3::new(movement.pair.x, 0.0, -movement.pair.y).normalize();
            velocity.0 = move_vec * PLAYER_MOVE_SPEED;
        }
    }
}
