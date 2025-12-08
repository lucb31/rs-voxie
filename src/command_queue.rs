use glam::{Mat4, Vec3};
use log::debug;

pub struct CommandQueue {
    queue: Vec<Command>,
}

impl CommandQueue {
    pub fn new() -> CommandQueue {
        Self { queue: vec![] }
    }

    pub fn enqueue(&mut self, cmd: Command) {
        debug!("Enqueuing command {cmd:?}");
        self.queue.push(cmd);
    }

    pub fn iter(&mut self) -> std::vec::Drain<'_, Command> {
        self.queue.drain(..)
    }
}

#[derive(Debug)]
pub enum Command {
    SpawnProjectile { transform: Mat4, velocity: Vec3 },
}
