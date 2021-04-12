use specs::World;

use crate::{components::Position, map::Map};

mod common;
mod simple_map;

use self::simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, world: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
}

pub fn random_builder(depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(depth))
}
