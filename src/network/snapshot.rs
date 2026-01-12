use std::time::{Duration, Instant};

use glam::Mat4;
use log::{debug, error, trace, warn};
use serde::{Deserialize, Serialize};

use crate::{
    config::{BROADCAST_DT, SIMULATION_DT},
    systems::physics::Transform,
};

use super::{
    Authority, ClientId, NetEntityId, NetworkReplicated, NetworkWorld, time_sync::TimeSync,
};

#[derive(Debug)]
struct Snapshot {
    server_time: Duration,
    snapshots: Vec<EntitySnapshot>,
}
impl Snapshot {
    fn new(server_time: Duration, snapshots: Vec<EntitySnapshot>) -> Self {
        Self {
            server_time,
            snapshots,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub net_entity_id: NetEntityId,
    pub transform: Transform,
}

const SNAP_BUFFER_SIZE: usize = 20;
const INTERPOLATION_DELAY: Duration = Duration::from_millis(16 * 6); // Simulate client at roughly 6
// frames behind

/// Manages client-side interpolation buffer
pub struct SnapshotManager {
    snapshot_buffer: [Option<Snapshot>; SNAP_BUFFER_SIZE],
    head: usize,
    render_server_time: Duration,
}

impl SnapshotManager {
    pub fn new() -> SnapshotManager {
        Self {
            snapshot_buffer: std::array::from_fn(|_| None),
            head: 0,
            render_server_time: Duration::ZERO,
        }
    }

    pub fn store_snapshot(&mut self, frame: u32, data: Vec<EntitySnapshot>) {
        debug!("Storing snapshot at {frame}");
        let server_ingame_time = frame * SIMULATION_DT;
        self.snapshot_buffer[self.head] = Some(Snapshot::new(server_ingame_time, data));
        self.head = (self.head + 1) % SNAP_BUFFER_SIZE;
    }

    /// Find two snapshots surrounding target server time
    fn sample(&self, target_time: Duration) -> Option<(&Snapshot, &Snapshot, f32)> {
        let mut older: Option<&Snapshot> = None;
        let mut newer: Option<&Snapshot> = None;

        for snap in self.snapshot_buffer.iter().flatten() {
            if snap.server_time <= target_time {
                if older.is_none_or(|o| snap.server_time > o.server_time) {
                    older = Some(snap);
                }
            } else if newer.is_none_or(|n| snap.server_time < n.server_time) {
                newer = Some(snap);
            }
        }

        let (a, b) = match (older, newer) {
            (Some(a), Some(b)) => (a, b),
            _ => return None,
        };

        let alpha = (target_time - a.server_time).as_secs_f32()
            / (b.server_time - a.server_time).as_secs_f32();

        Some((a, b, alpha.clamp(0.0, 1.0)))
    }

    /// Update interpolated entities (marked with NetworkReplicated) with snapshot data available
    pub fn tick(
        &mut self,
        world: &mut NetworkWorld,
        client_id: ClientId,
        time_sync: &TimeSync,
        dt: Duration,
    ) {
        // Increase render server time linear with dt timestep size & snap back if goes out of sync
        // too far with estimated server time
        let target_server_time = time_sync
            .server_time_at(Instant::now())
            .saturating_sub(INTERPOLATION_DELAY);
        if self.render_server_time.abs_diff(target_server_time) >= BROADCAST_DT * 2 {
            warn!(
                "Render time ({:?}+) too far off from estimated server time ({:?}) -> Snapping back",
                self.render_server_time, target_server_time
            );
            self.render_server_time = target_server_time;
        }
        self.render_server_time += dt;
        debug!(
            "Rendering at {:?}, Target would be {:?}",
            self.render_server_time, target_server_time
        );

        // Interpolate values at render time
        if let Some((a, b, alpha)) = self.sample(self.render_server_time) {
            // Apply linear interpolation to all tagged entities
            for (entity, (transform, replication)) in
                world.query::<(&mut Transform, &NetworkReplicated)>().iter()
            {
                if auth_match(&replication.authority, client_id) {
                    // Skip entities that the current client has authority over.
                    // These will be predicted, not interpolated
                    continue;
                }
                let net_entity_id = world
                    .get_net_entity_id(&entity)
                    .expect("Entity {entity} not tracked as net entity ");

                // Search for transform snapshot in buffer
                let prev_transform = extract_transform(a, *net_entity_id);
                let next_transform = extract_transform(b, *net_entity_id);

                // Lerp & apply
                let lerp_transform = lerp_optional(prev_transform, next_transform, alpha);
                match lerp_transform {
                    Some(snap) => {
                        trace!("Updating transform for net {net_entity_id} to {snap}");
                        transform.0 = snap;
                    }
                    None => {
                        error!(
                            "Could not interpolate transform. Probably missing snapshot information for entity {entity:?}, net_entity_id {net_entity_id}"
                        );
                    }
                };
            }
        }
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

fn auth_match(authority: &Authority, client_id: ClientId) -> bool {
    if let Authority::Client(auth_client_id) = authority {
        *auth_client_id == client_id
    } else {
        false
    }
}
fn extract_transform(snapshot: &Snapshot, net_entity_id: NetEntityId) -> Option<Mat4> {
    let entity_snapshots = &snapshot.snapshots;
    let idx = entity_snapshots
        .binary_search_by_key(&net_entity_id, |e| e.net_entity_id)
        .ok()?;
    Some(entity_snapshots[idx].transform.0)
}

fn lerp_optional(a: Option<Mat4>, b: Option<Mat4>, t: f32) -> Option<Mat4> {
    match a {
        Some(val_a) => match b {
            Some(val_b) => Some(lerp_mat4(val_a, val_b, t)),
            None => Some(val_a),
        },
        None => b,
    }
}

fn lerp_mat4(a: Mat4, b: Mat4, t: f32) -> Mat4 {
    a + (b - a) * t
}
