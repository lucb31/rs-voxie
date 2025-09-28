## [unreleased]

### 🚀 Features

- *(performance)* Utilize instanced vertex arrays
- Add timing & rendering metrics
- Add option to disable VSync
- Perform visibility check on world generation

### 🐛 Bug Fixes

- One off error when allocating cube render batches
- Rounding errors in BB check by using f32 offset vectors

### 🚜 Refactor

- Only pass voxel positions to vert shader
- Simplify & move voxel struc
- Move camera behavioral code to tick fn
## [0.0.3] - 2025-09-21

### 🚀 Features

- Generate multiple cube clusters
- Add octree node support
- Octree region queries
- Use Octree to represent world in game scene
- Render optimization using camera viewport region & octree region queries

### 🚜 Refactor

- Use glam::IVec3 instead of own impl
## [0.0.2] - 2025-09-15

### 🚀 Features

- Add multi mesh support to scene
- Add colored quad mesh
- Color cube, fix unit cube dimensions
- Checkerboard ground plane
- Improve free fly camera
- Add CLI to set number of cubes
- Benchmarking infrastructure
- Save benchmark results to csv
- One initial batch
- Draw cubes in batches
- Add perlin noise generated cubes
- Generate single cube slice with perlin noise heightmap

### 🚜 Refactor

- Load quad shaders as external assets
- Separate application from main to allow benchmarking
- Move cube shaders to assets
- Renderer trait to replace Mesh

### 📚 Documentation

- Update changelog & readme
## [0.0.1] - 2025-09-13

### 🚀 Features

- Initial setup: Red background + imgui window
- Render a triangle
- Rotating triangle
- Add simple camera zooming out
- Improve DevXP by closign program on ESC press
- Add rotating cube
- Add global camera & fix camera zooming out behavior
- Camera zoom movement
- Add camera WASD + mouse movement
- Control camera speed & sensitivity via UI slider
- Only pan camera while middle mouse clicked
- Let there be light
- Add light direction in world space

### 🐛 Bug Fixes

- Simplify triangle example
- Smoothen camera movement
- Camera axis alignment and movement
- Imgui element interactions

### 🚜 Refactor

- Move game rendering logic outside
- Move triangle code
- Specify rendering surface dimensions in const

### 📚 Documentation

- Add readme.md
- Rebranding
