use std::rc::Rc;

use glow::HasContext;

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {}
    }

    pub fn render(&self, ctx: &Rc<glow::Context>) {
        unsafe {
            // The renderer assumes you'll be clearing the buffer yourself
            ctx.clear_color(1.0, 0.0, 0.0, 1.0);
            ctx.clear(glow::COLOR_BUFFER_BIT);
        };
    }
}
