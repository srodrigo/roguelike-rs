use std::cmp::{max, min};

use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::{
    components::{Player, Position, Viewshed},
    map::{Map, TileType},
};

pub fn try_move_player(delta_x: i32, delta_y: i32, world: &mut World) {
    let mut positions = world.write_storage::<Position>();
    let mut players = world.write_storage::<Player>();
    let mut viewshed = world.write_storage::<Viewshed>();
    let map = world.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewshed).join() {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            let mut player_pos = world.write_resource::<Point>();
            player_pos.x = pos.x;
            player_pos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(world: &mut World, ctx: &mut Rltk) -> bool {
    match ctx.key {
        None => return true,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, world)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, world)
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, world)
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, world)
            }
            _ => return true,
        },
    }

    false
}
