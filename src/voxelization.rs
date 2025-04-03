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
    pub octree_depth: usize,
}

impl Default for VoxelizationSettings {
    fn default() -> Self {
        Self {
            octree_depth: 6, // Default octree depth
        }
    }
}

impl VoxelizationSettings {
    pub fn voxel_size(&self) -> f32 {
        // Calculate voxel size from octree depth (range -1~1, width=2)
        2.0 / (1 << self.octree_depth) as f32
    }
}

// Voxel coordinate for HashSet
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct VoxelCoord(i32, i32, i32);

// Coordinate range from -1 to 1
pub const COORDINATE_RANGE: f32 = 1.0;

pub fn create_voxelized_mesh(model: &tobj::Model, octree_depth: usize) -> Mesh {
    // Calculate voxel size
    let voxel_size = 2.0 / (1 << octree_depth) as f32;

    let positions = &model.mesh.positions;
    let indices = &model.mesh.indices;

    // Create voxel grid using HashSet for efficient lookups
    let mut filled_voxels = HashSet::new();

    // Process all triangles for voxelization
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

        voxelize_triangle(p1, p2, p3, voxel_size, &mut filled_voxels);
    }

    println!(
        "Voxelization complete: depth {}, voxel size {:.6}, generated {} voxels",
        octree_depth,
        voxel_size,
        filled_voxels.len()
    );

    // Create voxelized mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let cube_size = voxel_size;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut mesh_indices = Vec::new();

    // Render visible voxel faces
    for voxel_coord in &filled_voxels {
        // Calculate voxel center position
        let voxel_center = Vec3::new(
            voxel_coord.0 as f32 * voxel_size,
            voxel_coord.1 as f32 * voxel_size,
            voxel_coord.2 as f32 * voxel_size,
        );

        // Check which faces are visible (no adjacent voxels)
        let neighbors = [
            VoxelCoord(voxel_coord.0 + 1, voxel_coord.1, voxel_coord.2), // right
            VoxelCoord(voxel_coord.0 - 1, voxel_coord.1, voxel_coord.2), // left
            VoxelCoord(voxel_coord.0, voxel_coord.1 + 1, voxel_coord.2), // top
            VoxelCoord(voxel_coord.0, voxel_coord.1 - 1, voxel_coord.2), // bottom
            VoxelCoord(voxel_coord.0, voxel_coord.1, voxel_coord.2 + 1), // front
            VoxelCoord(voxel_coord.0, voxel_coord.1, voxel_coord.2 - 1), // back
        ];

        let half = cube_size / 2.0;

        // Define the 8 corners of the cube
        let corners = [
            voxel_center + Vec3::new(-half, -half, -half), // 0: back bottom left
            voxel_center + Vec3::new(half, -half, -half),  // 1: back bottom right
            voxel_center + Vec3::new(half, -half, half),   // 2: front bottom right
            voxel_center + Vec3::new(-half, -half, half),  // 3: front bottom left
            voxel_center + Vec3::new(-half, half, -half),  // 4: back top left
            voxel_center + Vec3::new(half, half, -half),   // 5: back top right
            voxel_center + Vec3::new(half, half, half),    // 6: front top right
            voxel_center + Vec3::new(-half, half, half),   // 7: front top left
        ];

        // Only add geometry for visible faces
        for (i, neighbor) in neighbors.iter().enumerate() {
            if !filled_voxels.contains(neighbor) {
                match i {
                    0 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [1, 2, 6, 5],
                        [1.0, 0.0, 0.0],
                    ), // right face
                    1 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [0, 4, 7, 3],
                        [-1.0, 0.0, 0.0],
                    ), // left face
                    2 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [4, 5, 6, 7],
                        [0.0, 1.0, 0.0],
                    ), // top face
                    3 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [0, 3, 2, 1],
                        [0.0, -1.0, 0.0],
                    ), // bottom face
                    4 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [3, 7, 6, 2],
                        [0.0, 0.0, 1.0],
                    ), // front face
                    5 => add_cube_face(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut mesh_indices,
                        &corners,
                        [0, 1, 5, 4],
                        [0.0, 0.0, -1.0],
                    ), // back face
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

// Add a face to the cube
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

    // Add four vertices
    for &idx in &face_indices {
        vertices.push([corners[idx].x, corners[idx].y, corners[idx].z]);
        normals.push(normal);
    }

    // Add UV coordinates
    uvs.push([0.0, 0.0]);
    uvs.push([1.0, 0.0]);
    uvs.push([1.0, 1.0]);
    uvs.push([0.0, 1.0]);

    // Add two triangles
    indices.push(start_idx);
    indices.push(start_idx + 2);
    indices.push(start_idx + 1);

    indices.push(start_idx);
    indices.push(start_idx + 3);
    indices.push(start_idx + 2);
}

// Triangle voxelization using voxel grid
fn voxelize_triangle(
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    voxel_size: f32,
    filled_voxels: &mut HashSet<VoxelCoord>,
) {
    // Calculate triangle bounding box
    let bb_min_x = p1.x.min(p2.x.min(p3.x));
    let bb_min_y = p1.y.min(p2.y.min(p3.y));
    let bb_min_z = p1.z.min(p2.z.min(p3.z));

    let bb_max_x = p1.x.max(p2.x.max(p3.x));
    let bb_max_y = p1.y.max(p2.y.max(p3.y));
    let bb_max_z = p1.z.max(p2.z.max(p3.z));

    // Convert to voxel coordinates
    let min_voxel_x = (bb_min_x / voxel_size).floor() as i32;
    let min_voxel_y = (bb_min_y / voxel_size).floor() as i32;
    let min_voxel_z = (bb_min_z / voxel_size).floor() as i32;

    let max_voxel_x = (bb_max_x / voxel_size).ceil() as i32;
    let max_voxel_y = (bb_max_y / voxel_size).ceil() as i32;
    let max_voxel_z = (bb_max_z / voxel_size).ceil() as i32;

    // Clamp coordinates to valid range
    let max_idx = (COORDINATE_RANGE / voxel_size) as i32;
    let min_idx = -max_idx;

    let min_voxel_x = min_voxel_x.max(min_idx);
    let min_voxel_y = min_voxel_y.max(min_idx);
    let min_voxel_z = min_voxel_z.max(min_idx);

    let max_voxel_x = max_voxel_x.min(max_idx);
    let max_voxel_y = max_voxel_y.min(max_idx);
    let max_voxel_z = max_voxel_z.min(max_idx);

    // Calculate triangle normal
    let edge1 = p2 - p1;
    let edge2 = p3 - p1;
    let normal = edge1.cross(edge2).normalize();

    // Iterate through all voxels in the bounding box
    for x in min_voxel_x..=max_voxel_x {
        for y in min_voxel_y..=max_voxel_y {
            for z in min_voxel_z..=max_voxel_z {
                let voxel_center = Vec3::new(
                    (x as f32 + 0.5) * voxel_size,
                    (y as f32 + 0.5) * voxel_size,
                    (z as f32 + 0.5) * voxel_size,
                );

                // Check if voxel intersects with triangle
                // Simplified method: check if distance from voxel center to triangle plane is less than voxel radius
                let dist_to_plane = (voxel_center - p1).dot(normal).abs();

                if dist_to_plane <= voxel_size * 0.87 {
                    // sqrt(3)/2 ≈ 0.87
                    filled_voxels.insert(VoxelCoord(x, y, z));
                }
            }
        }
    }
}
