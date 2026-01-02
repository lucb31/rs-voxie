use crate::network::ClientId;

pub(super) struct Lobby {
    players: [Option<ClientId>; 2],
    next_player_idx: usize,
}

impl Lobby {
    pub fn new() -> Lobby {
        Self {
            players: [None, None],
            next_player_idx: 0,
        }
    }

    /// Returns player slot starting at 0
    pub fn join(&mut self, client: ClientId) -> Result<usize, String> {
        let player_slot = self.next_player_idx;
        if self.next_player_idx > self.players.len() - 1 {
            return Err("Cannot join. Lobby already full".to_string());
        }
        if self
            .players
            .iter()
            .any(|p| p.is_some() && p.unwrap() == client)
        {
            return Err("Cannot join. Player already joined this lobby".to_string());
        }
        self.players[player_slot] = Some(client);
        self.next_player_idx += 1;
        Ok(player_slot)
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
