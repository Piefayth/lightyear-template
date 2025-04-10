use avian3d::prelude::{Position, Rotation};
use bevy::prelude::*;
use lightyear::prelude::{
    FromClients, MessageSend, NetworkTarget, ReplicateHierarchy, Replicating, ServerConnectEvent,
    ServerConnectionManager, ServerDisconnectEvent, ServerReplicate,
    server::{ControlledBy, Lifetime, ServerCommandsExt, SyncTarget},
};
use mygame_assets::CurrentLevel;
use mygame_common::REPLICATION_GROUP_PREDICTED;
use mygame_protocol::{
    component::Player,
    message::{ClientLevelLoadComplete, Level, ServerWelcome, UnorderedReliable},
};

pub struct ReplicationPlugin;
impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_client_connect_success);
        app.add_observer(on_client_disconnect);

        app.add_systems(Update, on_client_load_complete);
    }
}

fn on_client_load_complete(
    mut ev_client_load_complete: ResMut<Events<FromClients<ClientLevelLoadComplete>>>,
    mut commands: Commands,
    q_players: Query<&Player>,
) {
    for ev in ev_client_load_complete.drain() {
        let player_exists = q_players.iter().any(|player_id| player_id.0 == ev.from);
        let player_start_position = Position(Vec3::new(0.0, 6.0, 0.0));

        if !player_exists {
            commands.spawn((
                player_start_position,
                Rotation::default(),
                Player(ev.from),
                ServerReplicate {
                    group: REPLICATION_GROUP_PREDICTED,
                    controlled_by: ControlledBy {
                        target: NetworkTarget::Single(ev.from),
                        lifetime: Lifetime::SessionBased,
                    },
                    sync: SyncTarget {
                        prediction: NetworkTarget::Single(ev.from),
                        interpolation: NetworkTarget::AllExceptSingle(ev.from),
                    },
                    hierarchy: ReplicateHierarchy {
                        enabled: false,
                        ..default()
                    },
                    ..default()
                },
            ));
        } else {
            warn!(
                "Client {} reported load complete, but character already existed in world. Ignoring.",
                ev.from
            );
        }
    }
}

fn on_client_connect_success(
    trigger: Trigger<ServerConnectEvent>,
    mut commands: Commands,
    mut server: ResMut<ServerConnectionManager>,
    current_level: Res<CurrentLevel>,
) {
    let client_id = trigger.event().client_id;

    if let Err(e) = server.send_message_to_target::<UnorderedReliable, ServerWelcome>(
        &ServerWelcome {
            current_level: current_level.0,
        },
        NetworkTarget::Single(client_id),
    ) {
        error!(
            "unable to replicate level to client id {}, had error {}",
            client_id, e
        );
        commands.disconnect(client_id);

        return;
    }

    info!("connected client ${}", trigger.event().client_id);
}

fn on_client_disconnect(trigger: Trigger<ServerDisconnectEvent>) {
    info!("disconnected client ${}", trigger.event().client_id);
}
