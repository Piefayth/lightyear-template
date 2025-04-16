use crate::game_state::GameState;
use bevy::prelude::*;
use lightyear::prelude::{
    client::{ClientCommandsExt, ClientConnection, NetClient},
    *,
};
use mygame_assets::{CurrentLevel, LevelState};
use mygame_common::Simulated;
use mygame_protocol::{
    component::Player,
    message::{ClientLevelLoadComplete, ServerWelcome, UnorderedReliable},
};

pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_server_welcome.run_if(in_state(GameState::ConnectingRemote)),
                #[cfg(feature = "host")]
                on_server_welcome.run_if(in_state(GameState::ConnectingSelf)),
            ),
        );
        app.add_systems(Update, await_spawn.run_if(in_state(GameState::Spawning)));
        app.add_systems(OnEnter(LevelState::Loaded), on_assets_loaded);
    }
}

/// Tag component to identify the local player
#[derive(Component)]
pub struct LocalPlayer;

/// Once finished loading the assets that the server requested the client to load
/// Signal the completion to the server
fn on_assets_loaded(mut commands: Commands, mut client: ResMut<ClientConnectionManager>) {
    commands.set_state(GameState::Spawning);

    if let Err(e) =
        client.send_message::<UnorderedReliable, ClientLevelLoadComplete>(&ClientLevelLoadComplete)
    {
        println!("unable to signal client level load complete due to {}", e);
        commands.disconnect_client();
    }
}

/// Respond to the welcome message from the server by initiating a load of the level requested
fn on_server_welcome(
    mut server_welcome_events: ResMut<Events<ClientReceiveMessage<ServerWelcome>>>,
    game_state: Res<State<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for ev in server_welcome_events.drain() {
        next_state.set(GameState::Loading);
        current_level.0 = ev.message.current_level;
    }
}

fn await_spawn(
    mut commands: Commands,
    q_spawned_player: Query<(Entity, &Player), (Simulated, Added<Player>)>,
    client: Res<ClientConnection>,
) {
    for (entity, player) in &q_spawned_player {
        if player.0 == client.id() {
            commands.entity(entity).insert(LocalPlayer);
            commands.set_state(GameState::Playing);
        }
    }
}
