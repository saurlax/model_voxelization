mod camera;
mod model;
mod ui;
mod voxelization;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use camera::{camera_controller_system, setup_camera};
use model::{load_model_system, ModelResource};
use ui::ui_system;
use voxelization::VoxelizationSettings;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .init_resource::<ModelResource>()
        .init_resource::<VoxelizationSettings>()
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (ui_system, load_model_system, camera_controller_system),
        )
        .run();
}
