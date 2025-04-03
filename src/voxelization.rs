use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use std::collections::HashSet;

#[derive(Resource)]
pub struct VoxelizationSettings {
    pub resolution: f32, // 体素大小
}

impl Default for VoxelizationSettings {
    fn default() -> Self {
        Self { resolution: 1.0 } // 默认分辨率设为1.0
    }
}

// 体素化坐标，用于HashSet去重
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct VoxelCoord(i32, i32, i32);

// 八叉树的最大深度和边界
const MAX_DEPTH: usize = 6; // 最大深度
pub const GRID_SIZE: i32 = 64; // 总网格大小 (-32 ~ 32)

// 简化后的体素化函数，不再需要额外的变换参数
pub fn create_voxelized_mesh(model: &tobj::Model, voxel_size: f32) -> Mesh {
    let positions = &model.mesh.positions;
    let indices = &model.mesh.indices;

    // 创建体素网格 (使用HashSet以支持高效查找)
    let mut filled_voxels = HashSet::new();

    // 遍历所有三角形，进行体素化
    for i in 0..indices.len() / 3 {
        let idx1 = indices[i * 3] as usize;
        let idx2 = indices[i * 3 + 1] as usize;
        let idx3 = indices[i * 3 + 2] as usize;

        let p1 = Vec3::new(
            positions[idx1 * 3],
            positions[idx1 * 3 + 1],
            positions[idx1 * 3 + 2],
        );
        let p2 = Vec3::new(
            positions[idx2 * 3],
            positions[idx2 * 3 + 1],
            positions[idx2 * 3 + 2],
        );
        let p3 = Vec3::new(
            positions[idx3 * 3],
            positions[idx3 * 3 + 1],
            positions[idx3 * 3 + 2],
        );

        // 直接使用顶点坐标进行体素化，因为已经变换过了
        voxelize_triangle(p1, p2, p3, voxel_size, &mut filled_voxels);
    }

    println!("体素化完成，共生成 {} 个体素", filled_voxels.len());

    // 创建体素化网格
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let cube_size = voxel_size * 0.95; // 稍微缩小，留出缝隙效果

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut mesh_indices = Vec::new();

    // 渲染体素网格
    for voxel_coord in &filled_voxels {
        // 计算体素在世界空间中的中心位置
        let voxel_center = Vec3::new(
            voxel_coord.0 as f32 * voxel_size,
            voxel_coord.1 as f32 * voxel_size,
            voxel_coord.2 as f32 * voxel_size,
        );

        // 检查哪些面是可见的（相邻体素不存在的面）
        let neighbors = [
            VoxelCoord(voxel_coord.0 + 1, voxel_coord.1, voxel_coord.2), // 右
            VoxelCoord(voxel_coord.0 - 1, voxel_coord.1, voxel_coord.2), // 左
            VoxelCoord(voxel_coord.0, voxel_coord.1 + 1, voxel_coord.2), // 上
            VoxelCoord(voxel_coord.0, voxel_coord.1 - 1, voxel_coord.2), // 下
            VoxelCoord(voxel_coord.0, voxel_coord.1, voxel_coord.2 + 1), // 前
            VoxelCoord(voxel_coord.0, voxel_coord.1, voxel_coord.2 - 1), // 后
        ];

        let half = cube_size / 2.0;

        // 立方体的8个顶点
        let corners = [
            voxel_center + Vec3::new(-half, -half, -half), // 0: 左下后
            voxel_center + Vec3::new(half, -half, -half),  // 1: 右下后
            voxel_center + Vec3::new(half, -half, half),   // 2: 右下前
            voxel_center + Vec3::new(-half, -half, half),  // 3: 左下前
            voxel_center + Vec3::new(-half, half, -half),  // 4: 左上后
            voxel_center + Vec3::new(half, half, -half),   // 5: 右上后
            voxel_center + Vec3::new(half, half, half),    // 6: 右上前
            voxel_center + Vec3::new(-half, half, half),   // 7: 左上前
        ];

        // 只为可见面添加几何体
        for (i, neighbor) in neighbors.iter().enumerate() {
            if !filled_voxels.contains(neighbor) {
                // 根据面的索引添加面
                match i {
                    0 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [1, 2, 6, 5],
                        [1.0, 0.0, 0.0],
                    ), // 右面
                    1 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [0, 4, 7, 3],
                        [-1.0, 0.0, 0.0],
                    ), // 左面
                    2 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [4, 5, 6, 7],
                        [0.0, 1.0, 0.0],
                    ), // 上面
                    3 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [0, 3, 2, 1],
                        [0.0, -1.0, 0.0],
                    ), // 下面
                    4 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [3, 7, 6, 2],
                        [0.0, 0.0, 1.0],
                    ), // 前面
                    5 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [0, 1, 5, 4],
                        [0.0, 0.0, -1.0],
                    ), // 后面
                    _ => {}
                }
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(mesh_indices));

    mesh
}

// 添加立方体的一个面
fn add_cube_face(
    vertices: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    corners: &[Vec3; 8],
    face_indices: [usize; 4],
    normal: [f32; 3],
) {
    let start_idx = vertices.len() as u32;

    // 添加四个顶点
    for &idx in &face_indices {
        vertices.push([corners[idx].x, corners[idx].y, corners[idx].z]);
        normals.push(normal);
    }

    // 添加UV坐标
    uvs.push([0.0, 0.0]);
    uvs.push([1.0, 0.0]);
    uvs.push([1.0, 1.0]);
    uvs.push([0.0, 1.0]);

    // 添加两个三角形的索引
    indices.push(start_idx);
    indices.push(start_idx + 2);
    indices.push(start_idx + 1);

    indices.push(start_idx);
    indices.push(start_idx + 3);
    indices.push(start_idx + 2);
}

// 简化的三角形体素化 - 使用体素坐标系统
fn voxelize_triangle(
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    voxel_size: f32,
    filled_voxels: &mut HashSet<VoxelCoord>,
) {
    // 计算三角形的边界框
    let bb_min_x = p1.x.min(p2.x.min(p3.x));
    let bb_min_y = p1.y.min(p2.y.min(p3.y));
    let bb_min_z = p1.z.min(p2.z.min(p3.z));

    let bb_max_x = p1.x.max(p2.x.max(p3.x));
    let bb_max_y = p1.y.max(p2.y.max(p3.y));
    let bb_max_z = p1.z.max(p2.z.max(p3.z));

    // 将边界框转换为体素坐标
    let min_voxel_x = (bb_min_x / voxel_size).floor() as i32;
    let min_voxel_y = (bb_min_y / voxel_size).floor() as i32;
    let min_voxel_z = (bb_min_z / voxel_size).floor() as i32;

    let max_voxel_x = (bb_max_x / voxel_size).ceil() as i32;
    let max_voxel_y = (bb_max_y / voxel_size).ceil() as i32;
    let max_voxel_z = (bb_max_z / voxel_size).ceil() as i32;

    // 限制体素坐标在 -32 ~ 32 范围内
    let min_bound = -(GRID_SIZE / 2);
    let max_bound = GRID_SIZE / 2 - 1;

    let min_voxel_x = min_voxel_x.max(min_bound);
    let min_voxel_y = min_voxel_y.max(min_bound);
    let min_voxel_z = min_voxel_z.max(min_bound);

    let max_voxel_x = max_voxel_x.min(max_bound);
    let max_voxel_y = max_voxel_y.min(max_bound);
    let max_voxel_z = max_voxel_z.min(max_bound);

    // 计算三角形的法线
    let edge1 = p2 - p1;
    let edge2 = p3 - p1;
    let normal = edge1.cross(edge2).normalize();

    // 遍历边界框中的所有体素
    for x in min_voxel_x..=max_voxel_x {
        for y in min_voxel_y..=max_voxel_y {
            for z in min_voxel_z..=max_voxel_z {
                let voxel_center = Vec3::new(
                    (x as f32 + 0.5) * voxel_size,
                    (y as f32 + 0.5) * voxel_size,
                    (z as f32 + 0.5) * voxel_size,
                );

                // 判断体素与三角形是否相交
                // 这里使用简化的方法：检查体素中心到三角形平面的距离是否小于体素半径
                let dist_to_plane = (voxel_center - p1).dot(normal).abs();

                if dist_to_plane <= voxel_size * 0.87 {
                    // sqrt(3)/2 约等于 0.87
                    filled_voxels.insert(VoxelCoord(x, y, z));
                }
            }
        }
    }
}
