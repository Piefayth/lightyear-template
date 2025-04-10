use bevy::prelude::*;
use leafwing_input_manager::{
    Actionlike,
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap, VirtualDPad},
};
use mygame_common::Simulated;
use mygame_protocol::input::NetworkedInput;
use serde::{Deserialize, Serialize};

use crate::{game_state::GameState, replication::LocalPlayer, ui::system_menu::SystemMenuState};

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<LocalInput>::default())
            .add_systems(
                Update,
                (
                    add_local_input_map,
                    handle_system_menu_or_cancel.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum LocalInput {
    #[actionlike(Button)]
    SystemMenuOrCancel,
}

fn add_local_input_map(
    mut commands: Commands,
    q_local_player: Query<Entity, (Simulated, Added<LocalPlayer>)>,
) {
    for player in &q_local_player {
        commands.entity(player).insert((
            InputMap::<NetworkedInput>::default()
                .with_dual_axis(NetworkedInput::Move, VirtualDPad::wasd()),
            InputMap::<LocalInput>::default().with(LocalInput::SystemMenuOrCancel, KeyCode::Escape),
            ActionState::<LocalInput>::default(),
        ));
    }
}

fn handle_system_menu_or_cancel(
    q_local_inputs: Query<&ActionState<LocalInput>>,
    system_menu_state: Res<State<SystemMenuState>>,
    mut next_system_menu_state: ResMut<NextState<SystemMenuState>>,
) {
    for local_input in &q_local_inputs {
        if local_input.just_pressed(&LocalInput::SystemMenuOrCancel) {
            match **system_menu_state {
                SystemMenuState::Open => next_system_menu_state.set(SystemMenuState::Closed),
                SystemMenuState::Closed => next_system_menu_state.set(SystemMenuState::Open),
            }
        }
    }
}
