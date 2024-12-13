use crate::protocol::{GameInfo, PlayerAction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub(crate) nickname: String,
    pub(crate) own_system_uuid: Uuid,
    pub(crate) current_system_uuid: Uuid,
    #[serde(skip_serializing)]
    pub(crate) actions: Vec<PlayerAction>,
    pub(crate) game_infos: Vec<GameInfo>,
}

impl Player {
    pub fn new(nickname: String, own_system_uuid: Uuid, current_system_uuid: Uuid) -> Self {
        Self {
            nickname,
            own_system_uuid,
            current_system_uuid,
            actions: Vec::default(),
            game_infos: Vec::default(),
        }
    }

    pub fn get_nickname(&self) -> &str {
        &self.nickname
    }

    pub fn own_system_uuid(&self) -> Uuid {
        self.own_system_uuid
    }

    pub fn current_system_uuid(&self) -> Uuid {
        self.current_system_uuid
    }
}
