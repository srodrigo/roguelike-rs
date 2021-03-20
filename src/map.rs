use std::{
    cmp::{max, min},
    usize,
};

use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rltk, RGB};
use specs::{Entity, World};

use crate::rect::Rect;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub type MapTiles = Vec<TileType>;
pub type Rooms = Vec<Rect>;
pub type TilesVisibility = Vec<bool>;
pub type TilesBlocking = Vec<bool>;

#[derive(Default)]
pub struct Map {
    pub tiles: MapTiles,
    pub rooms: Rooms,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: TilesVisibility,
    pub visible_tiles: TilesVisibility,
    pub blocked: TilesBlocking,
    pub tile_content: Vec<Vec<Entity>>,
}

const WIDTH: usize = 80;
const HEIGHT: usize = 43;
const SIZE: usize = WIDTH * HEIGHT;

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; SIZE],
            rooms: Vec::<Rect>::new(),
            width: WIDTH as i32,
            height: HEIGHT as i32,
            revealed_tiles: vec![false; SIZE],
            visible_tiles: vec![false; SIZE],
            blocked: vec![false; SIZE],
            tile_content: vec![Vec::new(); SIZE],
        };

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
            if idx > 0 && idx < (self.width * self.height) as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_veritcal_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < (self.width * self.height) as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x == 0 || x == self.width || y == 0 || y == self.height {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();

        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

pub fn draw_map(world: &World, ctx: &mut Rltk) {
    let map = world.fetch::<Map>();

    for x in 0..map.width {
        for y in 0..map.height {
            let idx = (x + y * map.width) as usize;
            let tile = map.tiles[idx];
            if map.revealed_tiles[idx] {
                let glyph;
                let mut fg;
                match tile {
                    TileType::Floor => {
                        glyph = rltk::to_cp437('.');
                        fg = RGB::from_f32(0.5, 0.5, 0.5);
                    }
                    TileType::Wall => {
                        glyph = rltk::to_cp437('#');
                        fg = RGB::from_f32(0.0, 0.5, 0.0);
                    }
                }
                if !map.visible_tiles[idx] {
                    fg = fg.to_greyscale();
                }
                ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
            }
        }
    }
}