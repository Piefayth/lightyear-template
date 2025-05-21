use std::time::Duration;

use avian3d::{
    math::AsF32,
    prelude::{LinearVelocity, Position, Rotation},
};
use bevy::prelude::*;
use lightyear::{
    client::prediction::rollback::DisableRollback,
    prelude::client::{
        Confirmed, Correction, Interpolated, InterpolationSet, Predicted, PredictionSet,
        VisualInterpolateStatus, VisualInterpolationPlugin,
    },
};

pub struct InterpolationPlugin;
impl Plugin for InterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VisualInterpolationPlugin::<Transform>::default());

        app.add_systems(
            PostUpdate,
            position_to_transform_for_interpolated.before(TransformSystem::TransformPropagate),
        );

        app.add_systems(Update, interp_loader);
    }
}

type ParentComponents = (
    &'static GlobalTransform,
    Option<&'static Position>,
    Option<&'static Rotation>,
);

type PosToTransformComponents = (
    &'static mut Transform,
    &'static Position,
    &'static Rotation,
    Option<&'static ChildOf>,
);

/// Copy the network-interpolated Position values to Transform
pub fn position_to_transform_for_interpolated(
    mut query: Query<PosToTransformComponents, With<Interpolated>>,
    parents: Query<ParentComponents, With<Children>>,
) {
    for (mut transform, pos, rot, parent) in &mut query {
        if let Some(parent) = parent {
            if let Ok((parent_transform, parent_pos, parent_rot)) = parents.get(parent.0) {
                let parent_transform = parent_transform.compute_transform();
                let parent_pos = parent_pos.map_or(parent_transform.translation, |pos| pos.f32());
                let parent_rot = parent_rot.map_or(parent_transform.rotation, |rot| rot.f32());
                let parent_scale = parent_transform.scale;
                let parent_transform = Transform::from_translation(parent_pos)
                    .with_rotation(parent_rot)
                    .with_scale(parent_scale);

                let new_transform = GlobalTransform::from(
                    Transform::from_translation(pos.f32()).with_rotation(rot.f32()),
                )
                .reparented_to(&GlobalTransform::from(parent_transform));

                transform.translation = new_transform.translation;
                transform.rotation = new_transform.rotation;
            }
        } else {
            transform.translation = pos.f32();
            transform.rotation = rot.f32();
        }
    }
}

fn interp_loader(
    mut commands: Commands,
    q_interpolated: Query<
        (Entity, &Position),
        (With<Predicted>, Without<VisualInterpolateStatus<Transform>>),
    >,
) {
    for (e, p) in &q_interpolated {
        commands
            .entity(e)
            .insert(VisualInterpolateStatus::<Transform> {
                trigger_change_detection: true,
                ..default()
            });
    }
}
