# 🛠️ Voxie — A Voxel Engine in Rust

### General
- 🎯 **Summer 2025 Side Project**
- 🧱 A simple Minecraft-style voxel engine built using **Rust** and **OpenGL bindings**.
- 🔧 Focused on learning and experimentation — not a full game (yet).

### Goals
- 🚀 Learn **Rust** through a practical, graphics-heavy project.
- 🎨 Deepen understanding of **3D rendering with OpenGL**.
- 🔍 Explore and evaluate **Rust OpenGL bindings** (like `gl`, `glium`, etc.).

---

## 🔧 Tech Stack

| Tool/Library | Purpose |
|--------------|---------|
| **Rust** | Core programming language |
| **OpenGL** | Rendering graphics |
| **gl** | OpenGL rust bindings |
| **glam** | Math library |
| **glutin** | Window management & OpenGL context creation |
| **imgui** | UI framework |

---

## 📦 Project Status

🛠️ Work in progress. Currently focused on:
- [x] Basic window setup with glutin & imgui
- [x] Loading shaders
- [x] Rendering cubes
- [x] Rendering shaded cubes
  - [x] Primitive directional light source 
  - [x] Fixed light direction along camera axis
  - [x] Absolute light direction in world space
- [ ] Rendering textured cubes
- [x] Basic camera movement (WASD + mouse)
- [x] Advanced camera movement (Control speed & sensitivity via UI)
- [x] Hold MIDDLE mouse to pan camera
- [x] Camera debug info(position & rot)
- [x] Simple world chunk generation
- [ ] Add back support for lighting
- [x] Viewport culling of world chunks
- [ ] Fix benchmark scene
- [ ] Run benchmarks in ci

- [ ] Camera Collision

- [ ] Growing the world tree on demand
- [ ] Saving & loading world tree

# Performance 
## Rendering 

### Already implemented

### Further ideas
- Utilize geometry shaders
  + Geometry of voxels is uniform
  + Only need pass position & orientation (NESW) to geometry shader
  - Since we're only passing in 12 vertices per draw call (also uniform for all voxels), not sure if it makes 
    a significant difference. Might be nice exercise though


## World generation & representation
Next big step: OctreeNodes:
- Will allow for infinitely growing sparsely populated spaces

### Terrain generation
- Use perlin noise to determine heightmap
- Per x-z position generate voxels until max height is reached

## CI Log 
- [ ] Run benchmarks on main / release to log results
- [ ] Run tests in ci

## Performance log 
- Running into issues as soon as we render 1024 - 2048 cubes
- After optimization: Can render up to 262.144 cubes with 60 fps
  => Batch draw calls instead of single draws

Optimization strategies
- Batch draw cubes
  - Share the mesh / vertex data buffers 
  - Currently each cube is creating the same buffered data
- Do not instantiate a cube mesh / object for every voxel
  - Track only the relevant state for **each** voxel in a matrix
    - Position, orientation, type of voxel (i.e. water, dirt, stone)
  - Track a list of 'visible' or 'edge-voxels'
    - Figure out which voxels are visible / edge-voxels 
    - Caching: Needs to be updated whenever any voxel changes, but not every frame
- Instead of deleting CubeBatches, we could reuse them (pooling strategy)


## 📚 Learning Resources

- [LearnOpenGL.com](https://learnopengl.com/)
- [Rust OpenGL Tutorial](https://github.com/bwasty/learn-opengl-rs)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)

---

## 🙋‍♂️ Notes to Self

- This project is primarily for learning — code quality will improve over time.
- Try to write clean, idiomatic Rust, but don’t stress perfection on the first pass.
- Commit often. Break things. Have fun.

---

## 📸 Screenshots (Coming Soon)

> Will add screenshots or GIFs of progress here.

---

## 📄 License

MIT License — free to use, modify, and share.

---

