use specs::World;

use crate::{components::Position, map::Map, rect::Rect};

mod bsp_dungeon;
mod bsp_interior;
mod common;
mod simple_map;

use self::simple_map::SimpleMapBuilder;
use self::{bsp_dungeon::BspDungeonBuilder, bsp_interior::BspInteriorBuilder};

pub type Rooms = Vec<Rect>;
pub type SnapshotHistory = Vec<Map>;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, world: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
    fn get_snapshot_history(&self) -> SnapshotHistory;
    fn take_snapshot(&mut self);
}

pub fn random_builder(depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    match rng.roll_dice(1, 3) {
        1 => Box::new(SimpleMapBuilder::new(depth)),
        2 => Box::new(BspInteriorBuilder::new(depth)),
        _ => Box::new(BspDungeonBuilder::new(depth)),
    }
}
