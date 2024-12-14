use crate::{
    game::{
        galaxy::Galactic,
        repr::{GlobalCoords, LocalCoords, Speed},
    },
    protocol::{ElementInfo, GameInfo, PlayerAction, PlayerInfo},
};
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
    pub fn update(
        &mut self,
        delta: f64,
        mut coords: GlobalCoords,
        direction: LocalCoords,
        speed: Speed,
        current_system: Galactic,
        others: Vec<&Galactic>,
    ) -> GlobalCoords {
        for action in &self.actions {
            match action {
                crate::protocol::PlayerAction::ShipState(ship_state) => {
                    if ship_state.throttle_up {
                        coords.translate_from_local_delta(&(&direction * speed * delta));
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }

        self.actions.clear();

        self.game_infos.push(GameInfo::Player(PlayerInfo {
            coords: coords.get_local_from_element(&current_system),
        }));

        let mut elements_infos = Vec::<ElementInfo>::default();
        for other in others {
            elements_infos.push(ElementInfo {
                coords: other.coords.get_local_from_element(&current_system),
            });
        }
        if !elements_infos.is_empty() {
            self.game_infos
                .push(GameInfo::ElementsInSystem(elements_infos));
        }
        coords
    }
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
