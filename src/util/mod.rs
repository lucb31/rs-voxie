use glam::Vec3;

mod ecs;

pub use ecs::despawn_all;

#[macro_export]
macro_rules! log_err {
    ($expr:expr, $($arg:tt)+) => {
        if let Err(err) = $expr {
            log::error!($($arg)+, err = err);
        }
    };
}

pub fn smooth_damp(
    current: Vec3,
    target: Vec3,
    velocity: &mut Vec3,
    smooth_time: f32,
    dt: f32,
) -> Vec3 {
    let omega = 2.0 / smooth_time;
    let x = omega * dt;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    let change = current - target;
    let temp = (*velocity + omega * change) * dt;
    *velocity = (*velocity - omega * temp) * exp;

    target + (change + temp) * exp
}
