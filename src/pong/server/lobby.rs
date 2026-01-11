use log::info;

use crate::{
    network::{ClientId, NetEntityId},
    pong::network::input::ClientInputBuffer,
};

const LOBBY_SIZE: usize = 2;

pub(super) struct Lobby {
    players: [Option<PlayerInfo>; LOBBY_SIZE],
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
            players: [const { None }; LOBBY_SIZE],
        }
    }

    /// Returns player slot starting at 0
    pub fn join(&mut self, client_id: ClientId) -> Result<usize, String> {
        for (idx, info) in self.players.iter().enumerate() {
            match info {
                Some(info) => {
                    if info.client_id == client_id {
                        return Err("Cannot join. Player already joined this lobby".to_string());
                    }
                }
                None => {
                    self.players[idx] = Some(PlayerInfo::new(client_id));
                    info!("Client {client_id} added to lobby in slot {idx}");
                    return Ok(idx);
                }
            }
        }
        Err("Cannot join. No slot available".to_string())
    }

    pub fn remove(&mut self, client_id: ClientId) -> Result<PlayerInfo, String> {
        let player_slot = self
            .players
            .iter()
            .enumerate()
            .find_map(|(idx, info)| (info.as_ref()?.client_id == client_id).then_some(idx))
            .ok_or("Unable to remove client {client_id}. Not found in lobby")?;
        let info = self.players[player_slot].take().unwrap();
        info!("Client {client_id} removed from lobby");
        Ok(info)
    }

    pub fn is_full(&self) -> bool {
        self.players.iter().all(|f| f.is_some())
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
