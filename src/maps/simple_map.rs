use super::common::{apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel};
use crate::{
    components::Position,
    map::{Map, TileType},
};
use crate::{rect::Rect, spawner};
use rltk::RandomNumberGenerator;

use super::MapBuilder;

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(depth: i32) -> (Map, Position) {
        let mut map = Map::new(depth);
        let initial_pos = SimpleMapBuilder::rooms_and_corridors(&mut map);
        (map, initial_pos)
    }

    fn spawn(map: &Map, world: &mut specs::World, depth: i32) {
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(world, room, depth);
        }
    }
}

impl SimpleMapBuilder {
    fn rooms_and_corridors(map: &mut Map) -> Position {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;

            let new_room = Rect::new(x, y, w, h);

            if !map.rooms.iter().any(|x| new_room.intersect(x)) {
                apply_room_to_map(map, &new_room);

                if !map.rooms.is_empty() {
                    let (center_x, center_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(map, prev_x, center_x, prev_y);
                        apply_vertical_tunnel(map, prev_y, center_y, center_x);
                    } else {
                        apply_vertical_tunnel(map, prev_y, center_y, prev_x);
                        apply_horizontal_tunnel(map, prev_x, center_x, center_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        let (stairs_x, stairs_y) = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.xy_idx(stairs_x, stairs_y);
        map.tiles[stairs_idx] = TileType::DownStairs;

        let (initial_pos_x, initial_pos_y) = map.rooms[0].center();
        Position {
            x: initial_pos_x,
            y: initial_pos_y,
        }
    }
}
