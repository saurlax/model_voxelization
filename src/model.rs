use crate::voxelization::{create_voxelized_mesh, VoxelizationSettings, COORDINATE_RANGE};
use bevy::prelude::*;
use std::path::PathBuf;

// Component marker for mesh entities
#[derive(Component)]
pub struct ModelMesh;

// Resource for storing model path
#[derive(Resource, Default)]
pub struct ModelResource {
    pub path: Option<PathBuf>,
    pub loaded: bool,
}

pub fn load_model_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut model_resource: ResMut<ModelResource>,
    voxel_settings: Res<VoxelizationSettings>,
    model_query: Query<Entity, With<ModelMesh>>,
) {
    if let Some(path) = &model_resource.path.clone() {
        if !model_resource.loaded {
            // Remove previous model
            for entity in model_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Load model
            if let Ok(loaded_obj) = tobj::load_obj(
                path,
                &tobj::LoadOptions {
                    triangulate: true,
                    ..Default::default()
                },
            ) {
                let (mut models, _materials_maybe) = loaded_obj;

                // Calculate overall bounding box
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut min_z = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                let mut max_z = f32::MIN;

                // Process all submodels
                for model in &models {
                    let positions = &model.mesh.positions;

                    for i in 0..positions.len() / 3 {
                        let x = positions[i * 3];
                        let y = positions[i * 3 + 1];
                        let z = positions[i * 3 + 2];

                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        min_z = min_z.min(z);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                        max_z = max_z.max(z);
                    }
                }

                // Calculate model center
                let center_x = (min_x + max_x) / 2.0;
                let center_y = (min_y + max_y) / 2.0;
                let center_z = (min_z + max_z) / 2.0;

                // Calculate model dimensions
                let size_x = max_x - min_x;
                let size_y = max_y - min_y;
                let size_z = max_z - min_z;
                let max_dimension = size_x.max(size_y.max(size_z));

                // Calculate scaling factor to fit model in -1~1 range
                let world_size = COORDINATE_RANGE * 2.0 * 0.95; // Use -0.95~0.95 actual range
                let scale_factor = if max_dimension > 0.0 {
                    world_size / max_dimension
                } else {
                    1.0
                };

                println!(
                    "Model info: dimensions [{:.2}, {:.2}, {:.2}], max size {:.2}, center [{:.2}, {:.2}, {:.2}], scale factor {:.4}",
                    size_x, size_y, size_z, max_dimension, center_x, center_y, center_z, scale_factor
                );

                // Transform all vertices
                for model in &mut models {
                    let positions = &mut model.mesh.positions;

                    // Center and scale each vertex
                    for i in 0..positions.len() / 3 {
                        let x_idx = i * 3;
                        let y_idx = i * 3 + 1;
                        let z_idx = i * 3 + 2;

                        positions[x_idx] = (positions[x_idx] - center_x) * scale_factor;
                        positions[y_idx] = (positions[y_idx] - center_y) * scale_factor;
                        positions[z_idx] = (positions[z_idx] - center_z) * scale_factor;
                    }
                }

                // Voxelize each transformed model
                for model in models {
                    let mesh = create_voxelized_mesh(&model, voxel_settings.octree_depth);
                    let mesh_handle = meshes.add(mesh);

                    // Create material
                    let material_handle = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.8, 0.7, 0.6),
                        perceptual_roughness: 0.9,
                        ..default()
                    });

                    // Spawn model entity
                    commands.spawn((
                        Mesh3d(mesh_handle),
                        MeshMaterial3d(material_handle),
                        Transform::from_xyz(0.0, 0.0, 0.0),
                        ModelMesh,
                    ));
                }

                model_resource.loaded = true;
                println!("Model loaded and voxelized: {}", path.display());
            } else {
                println!("Failed to load model: {}", path.display());
            }
        }
    }
}
