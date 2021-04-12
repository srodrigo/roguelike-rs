use specs::World;

use crate::{components::Position, map::Map};

mod common;
mod simple_map;

use self::simple_map::SimpleMapBuilder;

trait MapBuilder {
    fn build(depth: i32) -> (Map, Position);
    fn spawn(map: &Map, world: &mut World, depth: i32);
}

pub fn build_random_map(depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(depth)
}

pub fn spawn(map: &Map, world: &mut World, depth: i32) {
    SimpleMapBuilder::spawn(map, world, depth);
}
