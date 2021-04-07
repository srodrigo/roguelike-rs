use rltk::RGB;
use specs::prelude::*;

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confusion, Consumable, Equippable, Equipped, HungerClock,
        HungerState, InBackpack, InflictsDamage, MagicMapper, Name, Position, ProvidesFood,
        ProvidesHealing, SuffersDamage, WantsToDropItem, WantsToPickUpItem, WantsToRemoveItem,
        WantsToUseItem,
    },
    gamelog::GameLog,
    map::Map,
    particles::ParticlesBuilder,
    state::RunState,
};

pub struct ItemColecctionSystem {}

impl<'a> System<'a> for ItemColecctionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickUpItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ))
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RunState>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SuffersDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
        ReadStorage<'a, MagicMapper>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticlesBuilder>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut map,
            mut runstate,
            mut gamelog,
            entities,
            mut wants_use,
            names,
            consumables,
            healing,
            inflicts_damage,
            mut suffers_damage,
            area_of_effect,
            mut confused,
            mut combat_stats,
            equippable,
            mut equipped,
            provides_food,
            mut hunger_clocks,
            magic_mappers,
            mut backpack,
            mut particles_builder,
            positions,
        ) = data;

        for (entity, use_item, stats) in (&entities, &wants_use, &mut combat_stats).join() {
            let consumable = consumables.get(use_item.item);
            match consumable {
                None => {}
                Some(_) => {
                    entities.delete(use_item.item).expect("Delete failed");
                }
            }

            let item_heals = healing.get(use_item.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You drink the {}, healing {} hp.",
                            names.get(use_item.item).unwrap().name,
                            healer.heal_amount
                        ));
                    }

                    if let Some(pos) = positions.get(entity) {
                        particles_builder.request(
                            pos.x,
                            pos.y,
                            RGB::named(rltk::GREEN),
                            RGB::named(rltk::BLACK),
                            rltk::to_cp437('♥'),
                            200.0,
                        )
                    }
                }
            }

            let item_damage = inflicts_damage.get(use_item.item);
            match item_damage {
                None => {}
                Some(damage) => {
                    let target_point = use_item.target.unwrap();
                    let idx = map.xy_idx(target_point.x, target_point.y);
                    for mob in map.tile_content[idx].iter() {
                        SuffersDamage::new_damage(&mut suffers_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(use_item.item).unwrap();
                            gamelog.entries.push(format!(
                                "you use {} on {}, inflicting {} hp.",
                                item_name.name, mob_name.name, damage.damage
                            ));

                            if let Some(pos) = positions.get(*mob) {
                                particles_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::RED),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                );
                            }
                        }
                    }
                }
            }

            let mut targets: Vec<Entity> = Vec::new();
            match use_item.target {
                None => {
                    targets.push(*player_entity);
                }
                Some(target) => {
                    let area_effect = area_of_effect.get(use_item.item);
                    match area_effect {
                        None => {
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            let mut blast_tiles =
                                rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }

                                particles_builder.request(
                                    tile_idx.x,
                                    tile_idx.y,
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('░'),
                                    200.0,
                                );
                            }
                        }
                    }
                }
            }

            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(use_item.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(use_item.item).unwrap();
                                gamelog.entries.push(format!(
                                    "You use {} on {}, confusing them.",
                                    item_name.name, mob_name.name
                                ));

                                if let Some(pos) = positions.get(*mob) {
                                    particles_builder.request(
                                        pos.x,
                                        pos.y,
                                        rltk::RGB::named(rltk::MAGENTA),
                                        rltk::RGB::named(rltk::BLACK),
                                        rltk::to_cp437('?'),
                                        200.0,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused
                    .insert(mob.0, Confusion { turns: mob.1 })
                    .expect("Unable to insert status");
            }

            let item_equippable = equippable.get(use_item.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in
                        (&entities, &equipped, &names).join()
                    {
                        if already_equipped.owner == target && already_equipped.slot == target_slot
                        {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("you unequip {}.", name.name));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack
                            .insert(*item, InBackpack { owner: target })
                            .expect("Unable to insert backpack entry");
                    }

                    equipped
                        .insert(
                            use_item.item,
                            Equipped {
                                owner: target,
                                slot: target_slot,
                            },
                        )
                        .expect("Unable to insert equipped component");
                    backpack.remove(use_item.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!(
                            "You equip {}.",
                            names.get(use_item.item).unwrap().name
                        ));
                    }
                }
            }

            let item_edible = provides_food.get(use_item.item);
            match item_edible {
                None => {}
                Some(_) => {
                    let target = targets[0];
                    if let Some(hunger_clock) = hunger_clocks.get_mut(target) {
                        hunger_clock.state = HungerState::WellFed;
                        hunger_clock.duration = 200;
                        gamelog.entries.push(format!(
                            "You eat the {}.",
                            names.get(use_item.item).unwrap().name
                        ));
                    }
                }
            }

            let magic_mapper = magic_mappers.get(use_item.item);
            match magic_mapper {
                None => {}
                Some(_) => {
                    for tile in map.revealed_tiles.iter_mut() {
                        *tile = true;
                    }
                    gamelog
                        .entries
                        .push("The map is revealed to you!".to_string());
                    *runstate = RunState::MagicMapReveal { row: 0 };
                }
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }

            positions
                .insert(
                    to_drop.item,
                    Position {
                        x: dropper_pos.x,
                        y: dropper_pos.y,
                    },
                )
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
        }

        wants_remove.clear();
    }
}
