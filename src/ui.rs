use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::model::ModelResource;
use crate::voxelization::VoxelizationSettings;

pub fn ui_system(
    mut contexts: EguiContexts,
    mut model_resource: ResMut<ModelResource>,
    mut voxel_settings: ResMut<VoxelizationSettings>,
) {
    egui::TopBottomPanel::top("top_panel").show(contexts.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("3D Models", &["obj", "stl", "fbx"])
                        .pick_file()
                    {
                        model_resource.path = Some(path);
                        model_resource.loaded = false;
                    }
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("Settings", |ui| {
                ui.add(
                    egui::Slider::new(&mut voxel_settings.resolution, 0.01..=1.0)
                        .text("Voxel Size")
                        .logarithmic(true),
                );
            });
        });
    });

    // 显示当前加载的模型路径
    if let Some(path) = &model_resource.path {
        let path_display = format!("Loaded model: {}", path.display());
        let voxel_size_display = format!("Voxel size: {:.3}", voxel_settings.resolution);

        egui::Window::new("Model Info").show(contexts.ctx_mut(), |ui| {
            ui.label(path_display);
            ui.label(voxel_size_display);
            if ui.button("Reload").clicked() {
                model_resource.loaded = false;
            }
        });
    }
}
