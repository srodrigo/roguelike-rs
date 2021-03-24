use std::cmp::{max, min};

use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::{
    components::{CombatStats, Item, Player, Position, Viewshed, WantsToMelee, WantsToPickUpItem},
    gamelog::GameLog,
    map::Map,
    RunState,
};

pub fn try_move_player(delta_x: i32, delta_y: i32, world: &mut World) {
    let mut positions = world.write_storage::<Position>();
    let mut players = world.write_storage::<Player>();
    let mut viewshed = world.write_storage::<Viewshed>();
    let combat_stats = world.read_storage::<CombatStats>();
    let map = world.fetch::<Map>();
    let entities = world.entities();
    let mut wants_to_melee = world.write_storage::<WantsToMelee>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewshed).join()
    {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            if let Some(_) = combat_stats.get(*potential_target) {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            let mut player_pos = world.write_resource::<Point>();
            player_pos.x = pos.x;
            player_pos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(world: &mut World, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => return RunState::AwaitingInput,
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
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => try_move_player(1, -1, world),
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => try_move_player(-1, -1, world),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, world),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, world),
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::G => get_item(world),
            VirtualKeyCode::I => return RunState::ShowInventory,
            _ => return RunState::AwaitingInput,
        },
    }

    RunState::PlayerTurn
}

fn get_item(world: &mut World) {
    let player_pos = world.fetch::<Point>();
    let player_entity = world.fetch::<Entity>();
    let entities = world.entities();
    let items = world.read_storage::<Item>();
    let positions = world.read_storage::<Position>();
    let mut gamelog = world.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = world.write_storage::<WantsToPickUpItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickUpItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to  insert want to pickup");
        }
    }
}
