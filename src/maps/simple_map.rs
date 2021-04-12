use super::common::{apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel};
use crate::{
    components::Position,
    map::{Map, TileType},
};
use crate::{rect::Rect, spawner};
use rltk::RandomNumberGenerator;

use super::MapBuilder;

pub type Rooms = Vec<Rect>;

pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Rooms,
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, world: &mut specs::World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(world, room, self.depth);
        }
    }

    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }
}

impl SimpleMapBuilder {
    pub fn new(depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            rooms: Vec::new(),
        }
    }

    fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.map.height - h - 1) - 1;

            let new_room = Rect::new(x, y, w, h);

            if !self.rooms.iter().any(|x| new_room.intersect(x)) {
                apply_room_to_map(&mut self.map, &new_room);

                if !self.rooms.is_empty() {
                    let (center_x, center_y) = new_room.center();
                    let (prev_x, prev_y) = self.rooms[self.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(&mut self.map, prev_x, center_x, prev_y);
                        apply_vertical_tunnel(&mut self.map, prev_y, center_y, center_x);
                    } else {
                        apply_vertical_tunnel(&mut self.map, prev_y, center_y, prev_x);
                        apply_horizontal_tunnel(&mut self.map, prev_x, center_x, center_y);
                    }
                }

                self.rooms.push(new_room);
            }
        }

        let (stairs_x, stairs_y) = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_idx(stairs_x, stairs_y);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        let (start_x, start_y) = self.rooms[0].center();
        self.starting_position = Position {
            x: start_x,
            y: start_y,
        };
    }
}
