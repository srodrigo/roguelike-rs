use std::cmp::{max, min};

use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::{
    components::{
        CombatStats, Item, Monster, Player, Position, Viewshed, WantsToMelee, WantsToPickUpItem,
    },
    gamelog::GameLog,
    map::{Map, TileType},
    RunState,
};

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
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => return skip_turn(world),
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::G => get_item(world),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::R => return RunState::ShowRemoveItem,
            VirtualKeyCode::Period => {
                if try_next_level(world) {
                    return RunState::NextLevel;
                }
            }
            VirtualKeyCode::Escape => return RunState::SaveGame,
            _ => return RunState::AwaitingInput,
        },
    }

    RunState::PlayerTurn
}

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

pub fn try_next_level(world: &mut World) -> bool {
    let player_pos = world.fetch::<Point>();
    let map = world.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = world.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("There is no way down from here.".to_string());
        false
    }
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

fn skip_turn(world: &mut World) -> RunState {
    let player_entity = world.fetch::<Entity>();

    if can_heal(&world, &player_entity) {
        let mut health_components = world.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}

fn can_heal(world: &&mut World, player_entity: &specs::shred::Fetch<Entity>) -> bool {
    let map_resource = world.fetch::<Map>();
    let viewshed_components = world.read_storage::<Viewshed>();
    let viewshed = viewshed_components.get(**player_entity).unwrap();
    let monsters = world.read_storage::<Monster>();

    for tile in viewshed.visible_tiles.iter() {
        let idx = map_resource.xy_idx(tile.x, tile.y);
        for entity_id in map_resource.tile_content[idx].iter() {
            match monsters.get(*entity_id) {
                None => {}
                Some(_) => {
                    return false;
                }
            }
        }
    }

    true
}
