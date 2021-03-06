use std::cmp::{max, min};

use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::{
    components::{Player, Position},
    map::{xy_idx, Map, TileType},
};

pub fn try_move_player(delta_x: i32, delta_y: i32, world: &mut World) {
    let mut positions = world.write_storage::<Position>();
    let mut players = world.write_storage::<Player>();
    let map = world.fetch::<Map>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

pub fn player_input(world: &mut World, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, world),
            VirtualKeyCode::Right => try_move_player(1, 0, world),
            VirtualKeyCode::Up => try_move_player(0, -1, world),
            VirtualKeyCode::Down => try_move_player(0, 1, world),
            _ => {}
        },
    }
}
