use bevy::{color::palettes::tailwind::SLATE_800, prelude::*};

use crate::game_state::GameState;

pub struct SystemMenuPlugin;

impl Plugin for SystemMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SystemMenuState>();

        app.add_systems(OnEnter(SystemMenuState::Open), open_system_menu)
            .add_systems(OnEnter(SystemMenuState::Closed), close_system_menu)
            .add_systems(OnExit(GameState::Playing), close_system_menu);
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SystemMenuState {
    Open,
    #[default]
    Closed,
}

#[derive(Component)]
pub struct SystemMenu;

fn open_system_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            SystemMenu,
        ))
        .with_children(|child_builder| {
            child_builder
                .spawn((
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(SLATE_800.into()),
                ))
                .with_children(|child_child_builder| {
                    child_child_builder
                        .spawn((
                            Text::new("Main Menu"),
                            TextFont {
                                font_size: 30.,
                                ..default()
                            },
                            Node {
                                padding: UiRect::bottom(Val::Px(20.)),
                                ..default()
                            },
                        ))
                        .observe(|_click: Trigger<Pointer<Click>>, mut commands: Commands| {
                            commands.set_state(GameState::MainMenu);
                        });

                    #[cfg(not(target_family = "wasm"))]
                    child_child_builder.spawn((
                        Text::new("Exit"),
                        TextFont {
                            font_size: 30.,
                            ..default()
                        },
                    ));
                });
        });
}

fn close_system_menu(mut commands: Commands, q_system_menu: Query<Entity, With<SystemMenu>>) {
    for system_menu in &q_system_menu {
        commands.entity(system_menu).despawn();
    }
}
