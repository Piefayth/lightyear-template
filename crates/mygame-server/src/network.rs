use bevy::prelude::*;
use lightyear::prelude::{
    FromClients, ReplicationGroup, ServerConnectionManager,
    server::{NetworkingState, ServerCommandsExt, ServerConnection},
};
use mygame_assets::{CurrentLevel, LevelState};
use mygame_protocol::message::{ClientHostRequestShutdown, Level};

use crate::app::ServerMode;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_server);
        app.add_systems(Update, on_host_request_shutdown);
        app.add_systems(
            OnExit(NetworkingState::Stopping),
            exit_on_client_host_shutdown,
        );
    }
}

fn start_server(mut commands: Commands, mut current_level: ResMut<CurrentLevel>) {
    commands.start_server();

    current_level.0 = Level::Example;
}

fn on_host_request_shutdown(
    mut commands: Commands,
    mut ev_host_request_shutdown: ResMut<Events<FromClients<ClientHostRequestShutdown>>>,
    server_mode: Res<ServerMode>,
) {
    let owner = match *server_mode {
        ServerMode::ClientHost(client_id) => client_id,
        _ => return,
    };

    for ev in ev_host_request_shutdown.drain() {
        if ev.from.to_bits() == owner.to_bits() {
            commands.stop_server();
        }
    }
}

/// When the client begins hosting, it creates a whole new server app
/// So when the server is stopped on a ClientHost, just exit the app.
fn exit_on_client_host_shutdown(mut commands: Commands, server_mode: Res<ServerMode>) {
    if let ServerMode::ClientHost(_) = *server_mode {
        commands.send_event(AppExit::Success);
    }
}
