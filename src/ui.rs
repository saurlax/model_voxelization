use crate::model::ModelResource;
use crate::voxelization::VoxelizationSettings;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

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
                // Octree depth slider
                let mut depth = voxel_settings.octree_depth as i32;
                ui.add(
                    egui::Slider::new(&mut depth, 1..=10)
                        .text("Octree Depth")
                        .integer(),
                );

                // Mark model for reload when depth changes
                let old_depth = voxel_settings.octree_depth;
                voxel_settings.octree_depth = depth as usize;

                if old_depth != voxel_settings.octree_depth {
                    if model_resource.path.is_some() {
                        model_resource.loaded = false;
                    }
                }

                // Display current voxel size
                ui.label(format!("Voxel size: {:.6}", voxel_settings.voxel_size()));
            });
        });
    });

    // Handle model info window or help screen
    if let Some(path) = &model_resource.path {
        // Create a local clone of the path to avoid borrowing model_resource inside the closure
        let path_display = path.display().to_string();
        let octree_depth = voxel_settings.octree_depth;
        let voxel_size = voxel_settings.voxel_size();

        // Track if we need to reload the model
        let mut should_reload = false;

        // Show model info window when model is loaded
        egui::Window::new("Model Info").show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Loaded model: {}", path_display));
            ui.label(format!("Octree depth: {}", octree_depth));
            ui.label(format!("Voxel size: {:.6}", voxel_size));
            if ui.button("Reload").clicked() {
                should_reload = true;
            }
        });

        // Apply reload flag after the closure is done
        if should_reload {
            model_resource.loaded = false;
        }
    } else {
        // Show help screen when no model is loaded
        egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                ui.label(
                    egui::RichText::new("No Model Loaded")
                        .text_style(egui::TextStyle::Heading)
                        .size(24.0),
                );
                ui.add_space(20.0);

                ui.label(
                    egui::RichText::new("Select 'File > Open' to load a 3D model")
                        .text_style(egui::TextStyle::Body)
                        .size(18.0),
                );
                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new("Supported formats: .obj, .stl, .fbx")
                        .text_style(egui::TextStyle::Body)
                        .size(16.0),
                );
                ui.add_space(20.0);

                if ui.button("Open Model...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("3D Models", &["obj", "stl", "fbx"])
                        .pick_file()
                    {
                        model_resource.path = Some(path);
                        model_resource.loaded = false;
                    }
                }

                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("Use mouse and WASD keys to navigate")
                        .text_style(egui::TextStyle::Body)
                        .size(16.0),
                );
                ui.label(
                    egui::RichText::new("Left click: Rotate camera")
                        .text_style(egui::TextStyle::Body)
                        .size(14.0),
                );
                ui.label(
                    egui::RichText::new("Right click: Pan camera")
                        .text_style(egui::TextStyle::Body)
                        .size(14.0),
                );
                ui.label(
                    egui::RichText::new("Mouse wheel: Zoom")
                        .text_style(egui::TextStyle::Body)
                        .size(14.0),
                );
                ui.label(
                    egui::RichText::new("WASD: Move horizontally")
                        .text_style(egui::TextStyle::Body)
                        .size(14.0),
                );
                ui.label(
                    egui::RichText::new("QE: Move vertically")
                        .text_style(egui::TextStyle::Body)
                        .size(14.0),
                );
            });
        });
    }
}
