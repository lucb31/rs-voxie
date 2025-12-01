# 🛠️ Voxie — A Voxel Engine in Rust

### General
- 🎯 **Summer 2025 Side Project**
- 🧱 A simple Minecraft-style voxel engine built using **Rust** and **OpenGL bindings**.
- 🔧 Focused on learning and experimentation — not a full game (yet).

---

## 📚 Table of Contents

1. [🎯 Project Goals](#-project-goals)  
2. [🧰 Tech Stack](#-tech-stack)  
3. [🚧 Project Status](#-project-status)  
4. [🚀 Performance](#-performance)  
   - [🎮 Rendering Optimizations](#-rendering-optimizations)  
   - [🌍 World Generation & Representation](#-world-generation--representation)  
5. [🧪 CI & Benchmarks](#-ci--benchmarks)  
6. [📚 Learning Resources](#-learning-resources)  
7. [📸 Screenshots](#-screenshots)  
8. [📄 License](#-license)

---

## 🎯 Project Goals

- 🚀 Learn **Rust** through a practical, graphics-heavy project.
- 🎨 Deepen understanding of **3D rendering with OpenGL**.
- 🔍 Explore and evaluate **Rust OpenGL bindings** (`gl`, `glium`, etc.).

---

## 🧰 Tech Stack

| Tool/Library | Purpose |
|--------------|---------|
| **Rust**     | Core programming language |
| **OpenGL**   | Rendering graphics |
| **gl**       | OpenGL Rust bindings |
| **glam**     | Math library |
| **glutin**   | Window management & OpenGL context creation |
| **imgui**    | UI framework |

---

## 🚧 Project Status

🛠️ Work in progress. Currently focused on:

### ✅ Completed
- Basic window setup with glutin & imgui  
- Shader loading  
- Rendering cubes  
- Rendering shaded cubes:
  - Primitive directional light source  
  - Fixed light direction along camera axis  
  - Absolute light direction in world space  
- Basic camera movement (WASD + mouse)  
- Advanced camera controls (speed & sensitivity via UI)  
- MIDDLE mouse panning  
- Camera debug info (position & rotation)  
- Simple world chunk generation  
- Viewport culling of world chunks  
- Player collision  

### 🕐 In Progress / Planned
- Rendering textured cubes  
- Add back support for lighting  
- Fix benchmark scene  
- Run benchmarks in CI  
- Growing the world tree on demand  
- Saving & loading world tree  

### 🔧 General Improvements
- [ ] Add more structured logging

---

## 🚀 Performance

### 🎮 Rendering Optimizations

#### ✅ Implemented
- **Batching draw calls**  
  - Reduced draw calls using instanced rendering  
  - Up to **262,144 cubes at 60 FPS**  
  - Current batch size: `1024 x 1024`  
- **OctreeNodes**  
  - Supports infinite, sparse spaces  
  - Efficient region queries for rendering  

#### 💡 Ideas
- **Skip unexposed voxels**  
  - Avoid rendering voxels completely surrounded (27 neighbors)
- Level of Detail rendering
  - Different shader variants for voxels lighting in near, mid and far plane
    - Near: normal map, specular, diffuse, ambient
    - Mid: Diffuse + ambient
    - Far: Ambient only
- Further research: Ambient occlusion baking -> Minecraft seems to do some clever tricks

#### ❌ Discarded
- **Geometry shaders**
  - ✔️ Voxels are uniform, geometry shaders seemed logical
  - ❌ Too costly and poorly optimized vs. instanced draws

---

### 🌍 World Generation & Representation

#### 🏞 Terrain Generation
- Uses **Perlin noise** to generate heightmap
- For each X-Z coordinate, voxels are generated up to a max height

---

## 🧪 CI & Benchmarks

### 📈 CI Log
- [ ] Run benchmarks on `main`/release
- [ ] Run tests in CI

### 🧠 Performance Log & Optimization Strategies
- Do not instantiate mesh for every voxel
  - Track only voxel state (position, orientation, type)
  - Maintain a list of **visible or edge voxels**
- Use caching strategies:
  - Update only when a voxel changes
  - Not every frame
- Reuse `CubeBatches` instead of deleting (pooling strategy)

---
## Release guide

- git cliff -o CHANGELOG.md --tag <next-version>
- git add CHANGELOG.md
- git commit -m "chore: update changelog for vX.Y.Z"
- git tag vX.Y.Z
- git push origin vX.Y.Z
- git push origin main

Create release from github & copy version changelog
---

## 📚 Learning Resources

- [LearnOpenGL.com](https://learnopengl.com/)
- [Rust OpenGL Tutorial](https://github.com/bwasty/learn-opengl-rs)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)

---

## 📸 Screenshots

> Coming Soon — will add screenshots or GIFs of progress here.

---

## 📄 License

MIT License — free to use, modify, and share.

---

