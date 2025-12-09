# Brainstorming Oct 9 2025
Underwater theme
Fish / person collecting food / resources / oxygen

Food meter + starve mechanic
=> Exploration covered, but does not require 'voxel engine' 

Place equiment that harvests for you
=> So you need to come back and collect
Limit where these can be placed
=> Could require certain terrain shapes. That would cover the terraforming aspect




# Notes on projectile explosion
Tried a lot of stuff with iterators and queries.
VoxelWorld, Octree & Cunks need some heavy refactoring to fit lazy Iterator concept
For this use case here we need **either** access to both the voxel and the
parent chunk

Problem: We need access to both the chunk and the voxel
Voxel to determine position where to insert
Chunk to mutate

IDEA: We can easily calculate the parent chunk in octree space
- Expose an access method to the chunk at octree position
- Expose an access method to the voxel at relative chunk position
both can be calculated from the contact point of the collision
Next up: Try that idea; not ideal but would get the job done
