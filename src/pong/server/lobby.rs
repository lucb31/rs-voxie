use glam::Vec3;

use crate::network::ClientId;

pub(super) struct Lobby {
    players: [Option<ClientId>; 2],
    player_spawn_positions: [Vec3; 2],
    next_player_idx: usize,
}

impl Lobby {
    pub fn new() -> Lobby {
        Self {
            players: [None, None],
            player_spawn_positions: [Vec3::new(-2.3, 0.0, 0.0), Vec3::new(2.3, 0.0, 0.0)],
            next_player_idx: 0,
        }
    }

    pub fn join(&mut self, client: ClientId) -> Result<Vec3, String> {
        if self.next_player_idx > 1 {
            return Err("Cannot join. Lobby already full".to_string());
        }
        if self
            .players
            .iter()
            .any(|p| p.is_some() && p.unwrap() == client)
        {
            return Err("Cannot join. Player already joined this lobby".to_string());
        }
        self.players[self.next_player_idx] = Some(client);
        let pos = self.player_spawn_positions[self.next_player_idx];
        self.next_player_idx += 1;
        Ok(pos)
    }

    pub fn is_ready(&self) -> bool {
        self.next_player_idx == 2
    }

    pub fn others(&self, client: ClientId) -> Vec<ClientId> {
        self.players
            .iter()
            .filter_map(|p| match p {
                Some(c) => match *c == client {
                    true => None,
                    false => Some(*c),
                },
                None => None,
            })
            .collect()
    }
}
