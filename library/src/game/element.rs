pub use crate::game::elements::body::Body;
pub use crate::game::elements::player::Player;
pub use crate::game::elements::system::System;

pub enum Element {
    System(System),
    Body(Body),
    Player(Player),
}
