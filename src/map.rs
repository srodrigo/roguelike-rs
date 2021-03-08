use std::cmp::{max, min};

use rltk::{RandomNumberGenerator, Rltk, RGB};

use crate::rect::Rect;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub type MapTiles = Vec<TileType>;
pub type Rooms = Vec<Rect>;

pub struct Map {
    pub tiles: MapTiles,
    pub rooms: Rooms,
    pub width: usize,
    pub height: usize,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width) + x as usize
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::<Rect>::new(),
            width: 80,
            height: 50,
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, map.height as i32 - h - 1) - 1;

            let new_room = Rect::new(x, y, w, h);

            if !map.rooms.iter().any(|x| new_room.intersect(x)) {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (center_x, center_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, center_x, prev_y);
                        map.apply_veritcal_tunnel(prev_y, center_y, center_x);
                    } else {
                        map.apply_veritcal_tunnel(prev_y, center_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, center_x, center_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width * self.height {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_veritcal_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width * self.height {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
}

pub fn draw_map(map: &Map, ctx: &mut Rltk) {
    for x in 0..map.width {
        for y in 0..map.height {
            match map.tiles[(x + y * map.width) as usize] {
                TileType::Floor => {
                    ctx.set(
                        x,
                        y,
                        RGB::from_f32(0.5, 0.5, 0.5),
                        RGB::from_f32(0., 0., 0.),
                        rltk::to_cp437('.'),
                    );
                }
                TileType::Wall => {
                    ctx.set(
                        x,
                        y,
                        RGB::from_f32(0.0, 0.5, 0.0),
                        RGB::from_f32(0., 0., 0.),
                        rltk::to_cp437('#'),
                    );
                }
            }
        }
    }
}
