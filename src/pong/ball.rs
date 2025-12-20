use std::{error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use hecs::World;
use log::info;

use crate::{
    meshes::objmesh::ObjMesh,
    pong::player::PongPaddle,
    renderer::{Mesh, RenderMeshHandle, ecs_renderer::MESH_PROJECTILE_2D, shader::Shader},
    systems::physics::{Transform, Velocity},
};

use crate::collision::{ColliderBody, get_collision_info};

const MIN_SPEED: f32 = 1.0;
const MAX_SPEED: f32 = 10.0;
// Number of paddle bounces until max_speed will be reached
const MAX_BOUNCES: usize = 20;

struct PongBall {
    speed: f32,
    bounces: usize,
}

pub fn spawn_ball(world: &mut World) {
    let scale = Vec3::splat(0.25);
    let direction = Vec3::new(1.0, 0.5, 0.0).normalize();
    let speed = 1.0;
    world.spawn((
        PongBall { speed, bounces: 0 },
        Transform(Mat4::from_scale_rotation_translation(
            scale,
            Quat::IDENTITY,
            Vec3::ZERO,
        )),
        Velocity(direction * speed),
        RenderMeshHandle(MESH_PROJECTILE_2D),
        ColliderBody::SphereCollider { radius: 0.125 },
    ));
}

pub fn bounce_ball(world: &mut World) {
    if let Some((_, (ball_transform, ball_collider, velocity, ball))) = world
        .query::<(&mut Transform, &ColliderBody, &mut Velocity, &mut PongBall)>()
        .iter()
        .next()
    {
        for (collider_entity, (transform, collider)) in world
            .query::<(&Transform, &ColliderBody)>()
            .without::<&PongBall>()
            .iter()
        {
            let collision_info =
                get_collision_info(ball_collider, &ball_transform.0, collider, &transform.0);
            if let Some(info) = collision_info {
                debug_assert!(info.normal.is_finite(), "Received infinite normal");
                // Resolve penetration
                let d_penetration = info.normal * info.penetration_depth;
                ball_transform.0.w_axis.x += d_penetration.x;
                ball_transform.0.w_axis.y += d_penetration.y;
                ball_transform.0.w_axis.z += d_penetration.z;

                // Reflect velocity
                let reflected_velocity =
                    velocity.0 - 2.0 * velocity.0.dot(info.normal) * info.normal;
                // Alternative A: Scale to speed
                //velocity.0 = reflected_velocity.normalize() * ball.speed;
                // Alternative B: Fixed x-speed
                let x_multiplier = (ball.speed / reflected_velocity.x).abs();
                velocity.0 = reflected_velocity * x_multiplier;

                // Increase speed if we've hit a paddle
                if world.get::<&PongPaddle>(collider_entity).is_ok() {
                    ball.bounces += 1;
                    ball.speed = exp_lerp(
                        MIN_SPEED,
                        MAX_SPEED,
                        ball.bounces as f32 / MAX_BOUNCES as f32,
                    );
                    info!("Bounce #{}: New speed = {}", ball.bounces, ball.speed);
                }
            }
        }
    }
}

fn exp_lerp(min_val: f32, max_val: f32, t: f32) -> f32 {
    min_val * (max_val / min_val).powf(t)
}

pub fn projectile2d_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/projectile_2d.vert",
        "assets/shaders/projectile_2d.frag",
    )?;
    // Load vertex data from mesh
    let mut mesh = ObjMesh::new();
    mesh.load("assets/cube.obj").expect("Could not load mesh");
    let vertex_positions = mesh.get_vertex_buffers().position_buffer;
    let vertex_bytes: &[u8] = bytemuck::cast_slice(&vertex_positions);
    unsafe {
        // Setup vertex & index array and buffer
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));
        // Bind vertex data
        let vbo = gl.create_buffer()?;
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_bytes, gl::STATIC_DRAW);
        // Setup position attribute
        gl.vertex_attrib_pointer_f32(
            0,
            3,
            gl::FLOAT,
            false,
            3 * std::mem::size_of::<f32>() as i32,
            0,
        );
        gl.enable_vertex_array_attrib(vao, 0);
        gl.bind_buffer(gl::ARRAY_BUFFER, None);
        // 3 because vertex pos has 3 coordinates for each vertex
        Ok(Mesh::new(shader, vao, (vertex_positions.len() / 3) as i32))
    }
}
