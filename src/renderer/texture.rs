use glow::{HasContext, NativeTexture};
use std::{error::Error, path::Path, rc::Rc};

pub struct Texture {
    gl: Rc<glow::Context>,
    tbo: NativeTexture,
}

impl Texture {
    pub fn new(gl: &Rc<glow::Context>, img_path: &Path) -> Result<Texture, Box<dyn Error>> {
        let (image_data, width, height) = load_rgba_image_as_u8_raw(img_path)?;
        let tbo = create_texture_from_rgba_u8(gl, &image_data, width, height);
        Ok(Self {
            gl: Rc::clone(gl),
            tbo,
        })
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.bind_texture(gl::TEXTURE_2D, Some(self.tbo));
        }
    }
    pub fn unbind(&self) {
        unsafe {
            self.gl.bind_texture(gl::TEXTURE_2D, None);
        }
    }
}

fn create_texture_from_rgba_u8(
    gl: &glow::Context,
    data: &[u8],
    width: u32,
    height: u32,
) -> glow::NativeTexture {
    unsafe {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(gl::TEXTURE_2D, Some(texture));

        // Configure texture parameters
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        // Upload texture data
        gl.tex_image_2d(
            gl::TEXTURE_2D,
            0,               // level
            gl::RGBA as i32, // internal format
            width as i32,
            height as i32,
            0,                 // border
            gl::RGBA,          // format
            gl::UNSIGNED_BYTE, // type
            Some(data),        // raw data
        );

        gl.bind_texture(gl::TEXTURE_2D, None);
        texture
    }
}

fn load_rgba_image_as_u8_raw<P: AsRef<Path>>(
    path: P,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_rgba8();
    let (width, height) = img.dimensions();
    Ok((img.into_raw(), width, height))
}
