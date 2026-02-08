use glow::HasContext;
use std::mem::size_of;

pub struct FrameUniforms {
    ubo: glow::NativeBuffer,
}

impl FrameUniforms {
    pub fn new(gl: &glow::Context) -> Self {
        unsafe {
            let ubo = gl.create_buffer().expect("Failed to create UBO");
            gl.bind_buffer(glow::UNIFORM_BUFFER, Some(ubo));
            // Allocate storage (std140: float = 4 bytes)
            gl.buffer_data_size(
                glow::UNIFORM_BUFFER,
                size_of::<f32>() as i32,
                glow::DYNAMIC_DRAW,
            );
            // Bind to binding point 0
            gl.bind_buffer_base(glow::UNIFORM_BUFFER, 0, Some(ubo));

            gl.bind_buffer(glow::UNIFORM_BUFFER, None);
            Self { ubo }
        }
    }

    pub fn update_time(&self, gl: &glow::Context, time_seconds: f32) {
        unsafe {
            gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.ubo));
            let bytes = std::slice::from_raw_parts(
                &time_seconds as *const f32 as *const u8,
                size_of::<f32>(),
            );
            gl.buffer_sub_data_u8_slice(glow::UNIFORM_BUFFER, 0, bytes);
            gl.bind_buffer(glow::UNIFORM_BUFFER, None);
        }
    }
}
