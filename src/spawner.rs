use std::usize;

use rltk::{to_cp437, FontCharType, RandomNumberGenerator, RGB};
use specs::prelude::*;

use crate::{
    components::{
        BlocksTile, CombatStats, Item, Monster, Name, Player, Position, Potion, Renderable,
        Viewshed,
    },
    map::MAP_WIDTH,
    rect::Rect,
};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

pub fn player(world: &mut World, player_x: i32, player_y: i32) -> Entity {
    world
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Player {})
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .build()
}

pub fn random_monster(world: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = world.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => orc(world, x, y),
        _ => goblin(world, x, y),
    }
}

fn orc(world: &mut World, x: i32, y: i32) {
    monster(world, x, y, to_cp437('o'), "Orc");
}

fn goblin(world: &mut World, x: i32, y: i32) {
    monster(world, x, y, to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(world: &mut World, x: i32, y: i32, glyph: FontCharType, name: S) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(BlocksTile {})
        .with(Renderable {
            glyph: glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .build();
}

pub fn spawn_room(world: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    {
        let mut rng = world.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    for idx in monster_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        random_monster(world, x as i32, y as i32);
    }

    for idx in item_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        health_potion(world, x as i32, y as i32);
    }
}

fn health_potion(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(Potion { heal_amount: 8 })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .build();
}
