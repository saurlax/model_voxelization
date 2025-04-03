use crate::voxelization::{create_voxelized_mesh, VoxelizationSettings, GRID_SIZE};
use bevy::prelude::*;
use std::path::PathBuf;

// 为了能够查询包含Mesh的实体，添加组件标记
#[derive(Component)]
pub struct ModelMesh;

// 用于存储模型路径的资源
#[derive(Resource, Default)]
pub struct ModelResource {
    pub path: Option<PathBuf>,
    pub loaded: bool,
}

// 加载与体素化模型
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
            // 删除之前的模型
            for entity in model_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // 加载模型
            if let Ok(loaded_obj) = tobj::load_obj(
                path,
                &tobj::LoadOptions {
                    triangulate: true,
                    ..Default::default()
                },
            ) {
                let (mut models, _materials_maybe) = loaded_obj;

                // 首先计算所有子模型的整体包围盒
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut min_z = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                let mut max_z = f32::MIN;

                // 遍历所有子模型，找出整体的包围盒
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

                // 计算整个模型的中心点
                let center_x = (min_x + max_x) / 2.0;
                let center_y = (min_y + max_y) / 2.0;
                let center_z = (min_z + max_z) / 2.0;

                // 计算模型的整体尺寸
                let size_x = max_x - min_x;
                let size_y = max_y - min_y;
                let size_z = max_z - min_z;
                let max_dimension = size_x.max(size_y.max(size_z));

                // 计算统一的缩放因子
                let world_size = GRID_SIZE as f32 - 2.0; // 留出边界，实际使用 -31 ~ 31
                let scale_factor = if max_dimension > 0.0 {
                    world_size / max_dimension
                } else {
                    1.0
                };

                println!(
                    "整体模型信息: 尺寸 [{:.2}, {:.2}, {:.2}], 最大尺寸 {:.2}, 中心点 [{:.2}, {:.2}, {:.2}], 缩放因子 {:.4}",
                    size_x, size_y, size_z, max_dimension, center_x, center_y, center_z, scale_factor
                );

                // 直接对所有模型的顶点进行变换
                for model in &mut models {
                    let positions = &mut model.mesh.positions;

                    // 对每个顶点应用缩放和平移
                    for i in 0..positions.len() / 3 {
                        let x_idx = i * 3;
                        let y_idx = i * 3 + 1;
                        let z_idx = i * 3 + 2;

                        // 应用变换：居中并缩放
                        positions[x_idx] = (positions[x_idx] - center_x) * scale_factor;
                        positions[y_idx] = (positions[y_idx] - center_y) * scale_factor;
                        positions[z_idx] = (positions[z_idx] - center_z) * scale_factor;
                    }
                }

                // 使用变换后的模型进行体素化
                for model in models {
                    // 创建网格 - 直接使用已变换的模型
                    let mesh = create_voxelized_mesh(&model, voxel_settings.resolution);
                    let mesh_handle = meshes.add(mesh);

                    // 创建材质
                    let material_handle = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.8, 0.7, 0.6),
                        perceptual_roughness: 0.9,
                        ..default()
                    });

                    // 生成模型实体
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
