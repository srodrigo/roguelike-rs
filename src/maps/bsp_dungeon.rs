use rltk::RandomNumberGenerator;

use crate::{
    components::Position,
    map::{Map, TileType},
    rect::Rect,
    spawner, SHOW_MAPGEN_VISUALIZER,
};

use super::{common::apply_room_to_map, MapBuilder, Rooms, SnapshotHistory};

pub struct BspDungeonBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Rooms,
    history: SnapshotHistory,
    rects: Rooms,
}

impl MapBuilder for BspDungeonBuilder {
    fn build_map(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.rects.clear();
        self.rects
            .push(Rect::new(2, 2, self.map.width - 5, self.map.height - 5));
        let first_room = self.rects[0];
        self.add_subrects(first_room);

        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect(&mut rng);
            let candidate = self.get_random_sub_rect(rect, &mut rng);

            if self.is_possible(candidate) {
                apply_room_to_map(&mut self.map, &candidate);
                self.rooms.push(candidate);
                self.add_subrects(rect);
                self.take_snapshot();
            }

            n_rooms += 1;
        }

        self.rooms.sort_by(|a, b| a.x1.cmp(&b.x1));

        for i in 0..self.rooms.len() - 1 {
            let room = self.rooms[i];
            let next_room = self.rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x =
                next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1);
            let end_y =
                next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1);
            self.draw_corridor(start_x, start_y, end_x, end_y);
            self.take_snapshot();
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

    fn get_snapshot_history(&self) -> SnapshotHistory {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl BspDungeonBuilder {
    pub fn new(depth: i32) -> BspDungeonBuilder {
        BspDungeonBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth: depth,
            rooms: Vec::new(),
            history: Vec::new(),
            rects: Vec::new(),
        }
    }

    fn add_subrects(&mut self, rect: Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects
            .push(Rect::new(rect.x1, rect.y1, half_width, half_height));
        self.rects.push(Rect::new(
            rect.x1,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
    }

    fn get_random_rect(&mut self, rng: &mut RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 {
            return self.rects[0];
        }
        let idx = (rng.roll_dice(1, self.rects.len() as i32) - 1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect: Rect, rng: &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;

        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10)) - 1) + 1;
        let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10)) - 1) + 1;

        result.x1 += rng.roll_dice(1, 6) - 1;
        result.y1 += rng.roll_dice(1, 6) - 1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect: Rect) -> bool {
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        let mut can_build = true;
        for y in expanded.y1..=expanded.y2 {
            for x in expanded.x1..=expanded.x2 {
                if x > self.map.width - 2 {
                    can_build = false;
                }
                if y > self.map.height - 2 {
                    can_build = false;
                }
                if x < 1 {
                    can_build = false;
                }
                if y < 1 {
                    can_build = false;
                }
                if can_build {
                    let idx = self.map.xy_idx(x, y);
                    if self.map.tiles[idx] != TileType::Wall {
                        can_build = false;
                    }
                }
            }
        }

        can_build
    }

    fn draw_corridor(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
            if x < x2 {
                x += 1;
            } else if x > x2 {
                x -= 1;
            } else if y < y2 {
                y += 1;
            } else if y > y2 {
                y -= 1;
            }

            let idx = self.map.xy_idx(x, y);
            self.map.tiles[idx] = TileType::Floor;
        }
    }
}
