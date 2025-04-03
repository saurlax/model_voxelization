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
                // 只保留八叉树深度调节滑块
                let mut depth = voxel_settings.octree_depth as i32;
                ui.add(
                    egui::Slider::new(&mut depth, 1..=10)
                        .text("Octree Depth")
                        .integer(),
                );
                // 当深度改变时重新加载模型
                let old_depth = voxel_settings.octree_depth;
                voxel_settings.octree_depth = depth as usize;
                
                if old_depth != voxel_settings.octree_depth {
                    // 如果深度变化，标记模型需要重新加载
                    if model_resource.path.is_some() {
                        model_resource.loaded = false;
                    }
                }
                
                // 显示当前体素大小
                ui.label(format!("Voxel size: {:.6}", voxel_settings.voxel_size()));
            });
        });
    });

    // 显示当前加载的模型路径
    if let Some(path) = &model_resource.path {
        let path_display = format!("Loaded model: {}", path.display());
        let octree_depth_display = format!("Octree depth: {}", voxel_settings.octree_depth);
        let voxel_size_display = format!("Voxel size: {:.6}", voxel_settings.voxel_size());

        egui::Window::new("Model Info").show(contexts.ctx_mut(), |ui| {
            ui.label(path_display);
            ui.label(octree_depth_display);
            ui.label(voxel_size_display);
            if ui.button("Reload").clicked() {
                model_resource.loaded = false;
            }
        });
    }
}
