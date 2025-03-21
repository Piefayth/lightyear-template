use bevy::prelude::*;
use lightyear::prelude::*;

use crate::component::Level;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerWelcome {
    pub current_level: Level,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientLevelLoadComplete;

#[derive(Channel)]
pub struct UnorderedReliable;

pub fn register_messages(app: &mut App) {
    app.register_message::<ServerWelcome>(ChannelDirection::ServerToClient);

    app.register_message::<ClientLevelLoadComplete>(ChannelDirection::ClientToServer);

    app.add_channel::<UnorderedReliable>(ChannelSettings {
        mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
        ..default()
    });
}
