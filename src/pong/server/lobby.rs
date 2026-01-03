use crate::{
    network::{ClientId, NetEntityId},
    pong::network::input::ClientInputBuffer,
};

pub(super) struct Lobby {
    players: [Option<PlayerInfo>; 2],
    next_player_idx: usize,
}

pub(super) struct PlayerInfo {
    pub(super) input_buffer: ClientInputBuffer,
    pub(super) client_id: ClientId,
    pub(super) player_net_id: Option<NetEntityId>,
}

impl PlayerInfo {
    pub fn new(client_id: ClientId) -> PlayerInfo {
        Self {
            client_id,
            input_buffer: ClientInputBuffer::new(),
            player_net_id: None,
        }
    }
}

impl Lobby {
    pub fn new() -> Lobby {
        Self {
            players: [None, None],
            next_player_idx: 0,
        }
    }

    /// Returns player slot starting at 0
    pub fn join(&mut self, client_id: ClientId) -> Result<usize, String> {
        let player_slot = self.next_player_idx;
        if self.next_player_idx > self.players.len() - 1 {
            return Err("Cannot join. Lobby already full".to_string());
        }
        if self
            .players
            .iter()
            .any(|p| p.is_some() && p.as_ref().unwrap().client_id == client_id)
        {
            return Err("Cannot join. Player already joined this lobby".to_string());
        }
        self.players[player_slot] = Some(PlayerInfo::new(client_id));
        self.next_player_idx += 1;
        Ok(player_slot)
    }

    pub fn is_ready(&self) -> bool {
        self.next_player_idx == 2
    }

    pub(super) fn get_player_info_mut(&mut self, client_id: ClientId) -> Option<&mut PlayerInfo> {
        self.players
            .iter_mut()
            .find_map(|p| p.as_mut().filter(|info| info.client_id == client_id))
    }

    pub(super) fn iter_players_mut(&mut self) -> impl Iterator<Item = &mut PlayerInfo> {
        self.players.iter_mut().filter_map(|o| o.as_mut())
    }

    pub(super) fn iter_players(&self) -> impl Iterator<Item = &PlayerInfo> {
        self.players.iter().filter_map(|o| o.as_ref())
    }

    pub fn others(&self, client_id: ClientId) -> Vec<ClientId> {
        self.players
            .iter()
            .filter_map(|p| match p {
                Some(c) => match c.client_id == client_id {
                    true => None,
                    false => Some(c.client_id),
                },
                None => None,
            })
            .collect()
    }
}
