use std::time::Duration;

use bevy::{
    app::{PanicHandlerPlugin, ScheduleRunnerPlugin},
    asset::AssetPlugin,
    diagnostic::DiagnosticsPlugin,
    gltf::GltfPlugin,
    input::InputPlugin,
    log::LogPlugin,
    pbr::PbrPlugin,
    prelude::*,
    render::{
        RenderPlugin as BevyRenderPlugin,
        camera::CameraPlugin,
        mesh::skinning::SkinnedMeshInverseBindposes,
        settings::{RenderCreation, WgpuSettings},
    },
    scene::ScenePlugin,
    state::app::StatesPlugin,
    window::ExitCondition,
};
use lightyear::{
    prelude::*,
    server::{config::ServerConfig, plugin::ServerPlugins},
};
use mygame_common::CommonPlugin;
use mygame_render::RenderPlugin;

use crate::{network::NetworkPlugin, replication::ReplicationPlugin};

#[derive(Resource, PartialEq, Eq)]
pub enum ServerMode {
    Windowed,
    Headless,
    ClientHost(ClientId),
}

pub fn build_server_app(server_config: ServerConfig, asset_path: String, mode: ServerMode) -> App {
    let mut app = App::new();

    let asset_plugin = AssetPlugin {
        file_path: asset_path.clone(),
        ..default()
    };

    match mode {
        ServerMode::Windowed => {
            app.add_plugins((DefaultPlugins.build().set(asset_plugin), RenderPlugin));
        }
        _ => {
            app.add_plugins((
                MinimalPlugins.build().set(ScheduleRunnerPlugin::run_loop(
                    Duration::from_secs_f64(1.0 / 100.0),
                )),
                asset_plugin,
                WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                },
                BevyRenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: None,
                        ..default()
                    }),
                    ..default()
                },
                PanicHandlerPlugin,
                TransformPlugin,
                DiagnosticsPlugin,
                StatesPlugin,
                ScenePlugin,
                GltfPlugin::default(),
                PbrPlugin::default(),
            ));

            match mode {
                ServerMode::ClientHost(_) => {}
                _ => {
                    app.add_plugins(LogPlugin::default());
                }
            }

            app.init_asset::<Image>(); // or add ImagePlugin
        }
    };

    app.add_plugins(ServerPlugins {
        config: server_config,
    })
    .add_plugins((CommonPlugin, NetworkPlugin, ReplicationPlugin))
    .insert_resource(mode);

    app
}
