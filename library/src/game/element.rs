extern crate downcast;

use downcast::{downcast, Any};
use sqlx::sqlite::SqliteRow;
use uuid::Uuid;

pub use crate::game::elements::body::Body;
pub use crate::game::elements::player::Player;
pub use crate::game::elements::system::System;

use super::repr::Coords;

pub trait Element: Any {
    fn update(&mut self, delta: f32) -> bool;
    fn get_sql_insert_line(&self) -> String;
    fn from_sqlite_row(row: &SqliteRow) -> crate::Result<impl Element>
    where
        Self: Sized;
    fn get_uuid(&self) -> Uuid;
    fn is_synced(&self) -> bool;
    fn set_synced(&mut self, is_synced: bool);
    fn get_coords(&self) -> &Coords;
}

downcast!(dyn Element);
