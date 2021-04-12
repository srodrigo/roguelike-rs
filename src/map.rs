use std::{collections::HashSet, usize};

use rltk::{Algorithm2D, BaseMap, Point, Rltk, RGB};
use serde::{Deserialize, Serialize};
use specs::{Entity, World};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

pub type MapTiles = Vec<TileType>;
pub type TilesVisibility = Vec<bool>;
pub type TilesBlocking = Vec<bool>;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: MapTiles,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: TilesVisibility,
    pub visible_tiles: TilesVisibility,
    pub blocked: TilesBlocking,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 43;
pub const MAP_SIZE: usize = MAP_WIDTH * MAP_HEIGHT;

impl Map {
    pub fn new(depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; MAP_SIZE],
            width: MAP_WIDTH as i32,
            height: MAP_HEIGHT as i32,
            revealed_tiles: vec![false; MAP_SIZE],
            visible_tiles: vec![false; MAP_SIZE],
            blocked: vec![false; MAP_SIZE],
            depth: depth,
            bloodstains: HashSet::new(),
            tile_content: vec![Vec::new(); MAP_SIZE],
        }
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
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
                        glyph = wall_glyph(&*map, x, y);
                        fg = RGB::from_f32(0.0, 0.5, 0.0);
                    }
                    TileType::DownStairs => {
                        glyph = rltk::to_cp437('>');
                        fg = RGB::from_f32(0., 1.0, 1.0);
                    }
                }

                let mut bg = RGB::from_f32(0., 0., 0.);
                if map.bloodstains.contains(&idx) {
                    bg = RGB::from_f32(0.75, 0., 0.);
                }
                if !map.visible_tiles[idx] {
                    fg = fg.to_greyscale();
                    bg = RGB::from_f32(0., 0., 0.);
                }
                ctx.set(x, y, fg, bg, glyph);
            }
        }
    }
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 as i32 {
        return 35;
    }

    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8;
    }

    match mask {
        0 => 9,    // Pillar because we can't see neighbors
        1 => 186,  // Wall only to the north
        2 => 186,  // Wall only to the south
        3 => 186,  // Wall to the north and south
        4 => 205,  // Wall only to the west
        5 => 188,  // Wall to the north and west
        6 => 187,  // Wall to the south and west
        7 => 185,  // Wall to the north, south and west
        8 => 205,  // Wall only to the east
        9 => 200,  // Wall to the north and east
        10 => 201, // Wall to the south and east
        11 => 204, // Wall to the north, south and east
        12 => 205, // Wall to the east and west
        13 => 202, // Wall to the east, west, and south
        14 => 203, // Wall to the east, west, and north
        15 => 206, // ╬ Wall on all sides
        _ => 35,   // We missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
