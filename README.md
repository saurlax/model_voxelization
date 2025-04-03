# Model Voxelization

A 3D model voxelization tool implemented using the Bevy engine.

## Features

- Load and voxelize common 3D models (.obj, .stl, .fbx)
- Adjust voxelization precision through octree depth
- Intuitive user interface
- Interactive 3D navigation and viewing

## Installation

### Prerequisites

- Rust and Cargo (recommended to use [rustup](https://rustup.rs/))
- Graphics driver supporting Vulkan, Metal, or DX12

### Installation Steps

```bash
# Clone the repository
git clone https://github.com/saurlax/model_voxelization.git
cd model_voxelization

# Compile and run the project
cargo run --release
```

## Usage Guide

### Loading Models

1. Click on `File > Open` in the top menu bar
2. Select a 3D model file in .obj, .stl, or .fbx format from the file selector
3. The model will be loaded and displayed in the window

### Adjusting Voxelization Settings

1. Click on `Settings` in the top menu bar
2. Use the `Octree Depth` slider to adjust voxel precision (1-10)
   - Higher values produce finer voxels but require more processing resources
   - The model will automatically reload after modification

### Model Information

After loading a model, the `Model Info` window will display:

- Current loaded model path
- Set octree depth
- Calculated voxel size

Click the `Reload` button to reload the current model.

### 3D Navigation Controls

- **Left click and drag**: Rotate camera
- **Right click and drag**: Pan camera
- **Mouse wheel**: Zoom
- **WASD keys**: Move horizontally
- **QE keys**: Move vertically
