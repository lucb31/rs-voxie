use glam::Mat4;
use log::{debug, error, trace, warn};
use serde::{Deserialize, Serialize};

use crate::systems::physics::Transform;

use super::{NetEntityId, NetworkReplicated, NetworkWorld};

#[derive(Debug, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub net_entity_id: NetEntityId,
    pub transform: Transform,
}

const SNAP_BUFFER_SIZE: usize = 20;
const CLIENT_FRAME_DELAY: u32 = 10;

pub struct SnapshotManager {
    snapshot_buffer: [Option<Snapshot>; SNAP_BUFFER_SIZE],
    head: usize,
    current_client_frame: u32,
}

impl SnapshotManager {
    pub fn new(server_at: u32) -> SnapshotManager {
        Self {
            snapshot_buffer: std::array::from_fn(|_| None),
            head: 0,
            current_client_frame: server_at - CLIENT_FRAME_DELAY,
        }
    }

    pub fn store_snapshot(&mut self, frame: u32, data: Vec<EntitySnapshot>) {
        self.snapshot_buffer[self.head] = Some(Snapshot::new(frame, data));
        self.head = (self.head + 1) % SNAP_BUFFER_SIZE;
    }

    fn find_surrounding_snapshots(&self, frame: u32) -> (Option<&Snapshot>, Option<&Snapshot>) {
        let (prev, next) = find_surrounding_snapshot_indices(&self.snapshot_buffer, frame);
        (
            prev.map(|idx| self.snapshot_buffer[idx].as_ref().unwrap()),
            next.map(|idx| self.snapshot_buffer[idx].as_ref().unwrap()),
        )
    }

    /// Update interpolated entities (marked with NetworkReplicated) with snapshot data available
    pub fn tick(&mut self, world: &mut NetworkWorld) {
        // Figure out available frames and interpolation factor
        let frame = self.current_client_frame;
        let (prev, next) = self.find_surrounding_snapshots(frame);
        if prev.is_none() && next.is_none() {
            warn!("Update interpolated called before any snapshots have arrived");
            return;
        }
        let prev_frame = prev.map(|s| s.frame);
        let next_frame = next.map(|s| s.frame);
        let t = frame_lerp_t(frame, prev_frame, next_frame);
        debug!(
            "Interpolating at client frame {frame}: Prev. frame data found for frame {prev_frame:?}, next frame data found for {next_frame:?} => t = {t}"
        );
        trace!(
            "Available buffer: {:?}",
            self.snapshot_buffer
                .as_ref()
                .iter()
                .map(|s| s.as_ref().map(|inner| inner.frame))
                .collect::<Vec<Option<u32>>>()
        );

        // Apply linear interpolation to all tagged entities
        for (entity, transform) in world
            .query::<&mut Transform>()
            .with::<&NetworkReplicated>()
            .iter()
        {
            let net_entity_id = world
                .get_net_entity_id(&entity)
                .expect("Entity {entity} not tracked as net entity ");

            // Search for transform snapshot in buffer
            let prev_transform = extract_transform(prev, *net_entity_id);
            let next_transform = extract_transform(next, *net_entity_id);

            // Lerp & apply
            let lerp_transform = lerp_optional(prev_transform, next_transform, t);
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
        self.current_client_frame += 1;
    }
}

fn extract_transform(snapshot: Option<&Snapshot>, net_entity_id: NetEntityId) -> Option<Mat4> {
    let entity_snapshots = &snapshot?.snapshots;
    let idx = entity_snapshots
        .binary_search_by_key(&net_entity_id, |e| e.net_entity_id)
        .ok()?;
    Some(entity_snapshots[idx].transform.0)
}

fn frame_lerp_t(current_frame: u32, previous_frame: Option<u32>, next_frame: Option<u32>) -> f32 {
    if previous_frame.is_none() {
        return 1.0;
    } else if next_frame.is_none() {
        return 0.0;
    }
    let next_frame_val = next_frame.unwrap();
    let previous_frame_val = previous_frame.unwrap();
    debug_assert!(next_frame_val > previous_frame_val);
    let numerator = current_frame.saturating_sub(previous_frame_val) as f32;
    let denominator = (next_frame_val - previous_frame_val) as f32;
    (numerator / denominator).clamp(0.0, 1.0)
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

fn find_surrounding_snapshot_indices(
    buffer: &[Option<Snapshot>],
    target_frame: u32,
) -> (Option<usize>, Option<usize>) {
    let mut prev_index: Option<usize> = None;
    let mut next_index: Option<usize> = None;

    let mut prev_frame: Option<u32> = None;
    let mut next_frame: Option<u32> = None;

    for (i, entry) in buffer.iter().enumerate() {
        let snapshot = match entry {
            Some(s) => s,
            None => continue,
        };

        let frame = snapshot.frame;

        if frame < target_frame {
            if prev_frame.is_none_or(|f| frame > f) {
                prev_frame = Some(frame);
                prev_index = Some(i);
            }
        } else if next_frame.is_none_or(|f| frame < f) {
            next_frame = Some(frame);
            next_index = Some(i);
        }
    }

    (prev_index, next_index)
}

#[derive(Debug)]
struct Snapshot {
    frame: u32,
    snapshots: Vec<EntitySnapshot>,
}

impl Snapshot {
    fn new(frame: u32, snapshots: Vec<EntitySnapshot>) -> Self {
        Self { frame, snapshots }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(frame: u32) -> Snapshot {
        Snapshot {
            frame,
            snapshots: Vec::new(),
        }
    }

    const EPS: f32 = 1e-6;

    fn assert_approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < EPS, "expected {}, got {}", b, a);
    }

    #[test]
    fn returns_one_when_previous_is_none() {
        let t = frame_lerp_t(10, None, Some(20));
        assert_approx_eq(t, 1.0);
    }

    #[test]
    fn returns_zero_when_next_is_none() {
        let t = frame_lerp_t(10, Some(0), None);
        assert_approx_eq(t, 0.0);
    }

    #[test]
    fn returns_zero_at_previous_frame() {
        let t = frame_lerp_t(10, Some(10), Some(20));
        assert_approx_eq(t, 0.0);
    }

    #[test]
    fn returns_one_at_next_frame() {
        let t = frame_lerp_t(20, Some(10), Some(20));
        assert_approx_eq(t, 1.0);
    }

    #[test]
    fn returns_halfway_value_between_frames() {
        let t = frame_lerp_t(15, Some(10), Some(20));
        assert_approx_eq(t, 0.5);
    }

    #[test]
    fn clamps_to_zero_when_current_is_before_previous() {
        let t = frame_lerp_t(5, Some(10), Some(20));
        assert_approx_eq(t, 0.0);
    }

    #[test]
    fn clamps_to_one_when_current_is_after_next() {
        let t = frame_lerp_t(30, Some(10), Some(20));
        assert_approx_eq(t, 1.0);
    }

    #[test]
    fn works_with_adjacent_frames() {
        let t = frame_lerp_t(11, Some(10), Some(11));
        assert_approx_eq(t, 1.0);
    }

    #[test]
    fn saturating_sub_prevents_underflow() {
        let t = frame_lerp_t(0, Some(5), Some(10));
        assert_approx_eq(t, 0.0);
    }

    #[test]
    fn handles_large_frame_values() {
        let t = frame_lerp_t(1_000_000, Some(500_000), Some(1_500_000));
        assert_approx_eq(t, 0.5);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn debug_assert_triggers_when_next_not_greater_than_previous() {
        // This test only panics in debug builds
        let _ = frame_lerp_t(10, Some(20), Some(10));
    }

    #[test]
    fn finds_previous_and_next_in_middle() {
        let buffer = [
            Some(snap(10)),
            Some(snap(20)),
            Some(snap(40)),
            Some(snap(70)),
        ];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 35);

        assert_eq!(prev, Some(1)); // frame 20
        assert_eq!(next, Some(2)); // frame 40
    }

    #[test]
    fn finds_exact_match_as_next() {
        let buffer = [Some(snap(10)), Some(snap(20)), Some(snap(40))];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 20);

        assert_eq!(prev, Some(0)); // frame 10
        assert_eq!(next, Some(1)); // frame 20 (>= target)
    }

    #[test]
    fn no_previous_snapshot() {
        let buffer = [Some(snap(30)), Some(snap(40)), Some(snap(50))];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 10);

        assert_eq!(prev, None);
        assert_eq!(next, Some(0)); // frame 30
    }

    #[test]
    fn no_next_snapshot() {
        let buffer = [Some(snap(10)), Some(snap(20)), Some(snap(30))];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 100);

        assert_eq!(prev, Some(2)); // frame 30
        assert_eq!(next, None);
    }

    #[test]
    fn sparse_buffer_with_nones() {
        let buffer = [None, Some(snap(15)), None, Some(snap(45)), None];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 30);

        assert_eq!(prev, Some(1)); // frame 15
        assert_eq!(next, Some(3)); // frame 45
    }

    #[test]
    fn single_snapshot_less_than_target() {
        let buffer = [None, Some(snap(25)), None];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 50);

        assert_eq!(prev, Some(1));
        assert_eq!(next, None);
    }

    #[test]
    fn single_snapshot_greater_than_target() {
        let buffer = [None, Some(snap(25)), None];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 10);

        assert_eq!(prev, None);
        assert_eq!(next, Some(1));
    }

    #[test]
    fn empty_buffer() {
        let buffer: [Option<Snapshot>; 0] = [];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 42);

        assert_eq!(prev, None);
        assert_eq!(next, None);
    }

    #[test]
    fn unordered_buffer_still_works() {
        let buffer = [
            Some(snap(50)),
            Some(snap(10)),
            Some(snap(40)),
            Some(snap(20)),
        ];

        let (prev, next) = find_surrounding_snapshot_indices(&buffer, 35);

        assert_eq!(prev, Some(3)); // frame 20
        assert_eq!(next, Some(2)); // frame 40
    }
}
