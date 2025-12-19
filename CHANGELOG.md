## [0.0.8] - 2025-12-19

### 🚀 Features

- Ecs based projectile system
- Add projectile explosion
- Add iterator implementation to traverse octree
- Add iterator implementation to traverse voxel chunk
- Add iterator implementation to traverse voxel world

### 🐛 Bug Fixes

- Sphere mesh via ecs_renderer
- World growth
- 1 off error for voxel chunk collision iterator

### 🚜 Refactor

- Reorganize projectile system & cleanup old implementation
- Player logic to ecs
- Gun logic to ecs
- Reorganize modules
- Skybox logic to ecs
- Separate bbs from tree logic
- Replace without init queries with iterator use
- Use iterator for sphere clear
- Split octree logic into separate files
- Use iterator for uninitialized chunk generation & world growth
- Remove deprecated query methods
## [0.0.7] - 2025-12-05

### 🚀 Features

- Add support for different voxel kinds
- Improve default sensitivity & add slider
- Smoothen follow camera
- Add projectiles, shoot with space
- Add projectile lifetime

### 🐛 Bug Fixes

- Camera drift when panning diagonally

### 🚜 Refactor

- Move voxel structs to voxel package
- Move world struct to voxels package
- Move SceneStats to scenes package
- Move scene structs & traits into scenes package
- Stop passing gls
- Organize modules

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.0.7
## [0.0.6] - 2025-11-30

### 🚀 Features

- Simple dirt texture render
- Add lighting scene and simple cube diffuse lighting
- Add normal map support to lighting scene & fix diffuse light in game
- Add world boundary planes
- Initial implementation of one draw per chunk
- Add voxel rendering debug ui
- Add frustum culling
- Add demo for voxel mutation on player collision
- Add debugging info & helpers around world
- Limit player movement to 1e3 in each direction
- Add synchronous world growth
- Async chunk generation

### 🐛 Bug Fixes

- Mouse drag input on windows platform

### 🚜 Refactor

- Introduce ChunkGenerator trait
- Move shader to renderer package
- Use shader helper in cube rendering
- Improve scene management & application event loop
- Add proper logging using 'log' and 'env_logger' crates

### 📚 Documentation

- Structure Readme.md
- Update changelog

### ⚙️ Miscellaneous Tasks

- Add windows build pipeline
- Add release workflow
## [0.0.5] - 2025-10-07

### 🚀 Features

- New approach to octree using Arc<Chunks> instead
- Voxel iteration optimizations
- Parallelize world generation
- Perform generation of new batches asynchronously
- Further performance improvement by correcting chunk intersection
- Add collision test scene and scene selection via CLI
- Add sphere cube collision test example
- Add player collide and slide algorithm

### 🐛 Bug Fixes

- Collision check for sphere cube in collision test scene

### 🚜 Refactor

- Move chunk viewport sampling logic to cube_renderer
- Reorganize code. Move meshes to separate module
- Utilize drop traits to release gpu resources
- Separate player controller, camera controller, camera logic

### 📚 Documentation

- V0.0.4 release notes
- Update CHANGELOG
## [0.0.4] - 2025-09-28

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
