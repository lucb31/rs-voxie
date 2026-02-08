# rs-voxie Development Guidelines for AI Agents

## 🚀 Project Overview
A Rust-based voxel engine focusing on performance, modularity, and learning. The project uses OpenGL for rendering and supports multiple binary targets.

## 🛠️ Build & Development Commands

### Build Commands
```bash
# Build the entire project
cargo build

# Build with GUI features
cargo build --features "gui"

# Build specific binary
cargo build --bin voxie
cargo build --bin pong-client
cargo build --bin pong-server

# Release build
cargo build --release
```

### Test Commands
```bash
# Run all tests
cargo test

# Run tests for a specific binary
cargo test --bin voxie

# Run a single test (use full test name)
cargo test test_chunk_generation

# Run tests with verbose output
cargo test -- --nocapture

# Run tests for a specific module
cargo test --test collision
```

### Linting & Formatting
```bash
# Run Clippy for linting
cargo clippy

# Run Clippy with all warnings
cargo clippy -- -W clippy::pedantic

# Auto-format code
cargo fmt

# Check formatting without changes
cargo fmt -- --check
```

## 📝 Code Style Guidelines

### 1. Imports & Module Organization
```rust
// Import order matters:
// 1. Standard library imports
// 2. External crate imports
// 3. Local crate imports
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
};

use glam::{Vec3, IVec3};
use serde::{Serialize, Deserialize};

use crate::{
    config::WORLD_SIZE,
    renderer::Renderer,
};
```

### 2. Naming Conventions
- **Types**: `PascalCase` 
  - `VoxelChunk`, `NetworkServer`
- **Functions**: `snake_case`
  - `render_chunk()`, `calculate_distance()`
- **Constants**: `SCREAMING_SNAKE_CASE`
  - `MAX_CHUNK_SIZE`, `RENDER_DISTANCE`
- **Modules**: `snake_case`
  - `renderer`, `network`, `collision`

### 3. Error Handling
- Prefer `Result<T, E>` for fallible operations
- Use `?` operator for error propagation
- Create custom error types when appropriate

```rust
// Error handling pattern
fn load_chunk(id: ChunkId) -> Result<Chunk, ChunkError> {
    let data = read_chunk_data(id)?;
    data.validate().map_err(ChunkError::InvalidData)
}

// Custom error type
#[derive(Debug)]
enum ChunkError {
    IoError(std::io::Error),
    InvalidData(ValidationError),
}
```

### 4. Documentation
- Use `///` for public API documentation
- Include examples when possible
- Document type and function purpose clearly

```rust
/// Generates a new chunk using Perlin noise
///
/// # Arguments
/// * `seed` - Random seed for terrain generation
/// * `position` - Chunk's world position
///
/// # Returns
/// A fully generated chunk with terrain details
///
/// # Examples
/// ```
/// let chunk = generate_chunk(42, IVec3::ZERO);
/// assert!(chunk.is_valid());
/// ```
pub fn generate_chunk(seed: u64, position: IVec3) -> Chunk {
    // Implementation
}
```

### 5. Trait and Generic Usage
- Use traits for abstraction
- Prefer trait bounds over `Any`
- Use generics with constraints

```rust
/// Trait for renderable objects
trait Renderable {
    fn render(&self, renderer: &mut Renderer);
}

// Generic function with trait bound
fn render_objects<T: Renderable>(objects: &[T], renderer: &mut Renderer) {
    for obj in objects {
        obj.render(renderer);
    }
}
```

### 6. Performance Considerations
- Use `#[inline]` for small, frequently called functions
- Prefer `&[T]` over `Vec<T>` in function signatures
- Use `Rc<RefCell<T>>` for shared mutability
- Leverage `bytemuck` for SIMD-friendly structures

## 🏗️ Project Structure
```
src/
├── lib.rs           # Module root
├── bin/             # Binary entry points
├── cameras/         # Camera systems
├── collision/       # Collision detection
├── network/         # Networking layer
├── octree/          # Spatial data structure
├── renderer/        # OpenGL rendering
└── voxels/          # Voxel world generation
```

## 🔍 Testing Guidelines
- Write tests alongside implementation
- Use `#[cfg(test)]` for test modules
- Test both success and failure scenarios
- Use descriptive test names: `test_[feature]_[scenario]`

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_chunk_generation_with_seed() {
        let chunk = generate_chunk(42, IVec3::ZERO);
        assert!(chunk.is_valid());
    }
}
```

## 🚧 Feature Flags
- `gui`: Enables GUI-related modules
- Use `#[cfg(feature = "gui")]` for conditional compilation

## 📦 Dependency Management
- Minimize external dependencies
- Prefer standard library or well-maintained crates
- Always specify version constraints in `Cargo.toml`

## 💡 Advanced Patterns
- Use `#[derive()]` for common traits
- Leverage Rust's type system for safety
- Prefer composition over inheritance

## 🤝 Contribution Notes
- Follow existing code patterns
- Add tests for new functionality
- Run `cargo fmt` and `cargo clippy` before submitting PRs

## 📝 Release Process
```bash
# Update changelog
git cliff -o CHANGELOG.md --tag v0.X.Y

# Commit and tag
git add CHANGELOG.md
git commit -m "chore: update changelog for vX.Y.Z"
git tag vX.Y.Z
git push origin vX.Y.Z
```