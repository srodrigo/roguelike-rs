use rltk::{GameState, Rltk};
use specs::prelude::*;
use specs::World;

use crate::{
    components::{Position, Ranged, Renderable, WantsToDropItem, WantsToUseItem},
    damage_system::{self, DamageSystem},
    gui,
    inventory_system::{ItemColecctionSystem, ItemDropSystem, ItemUseSystem},
    map::{draw_map, Map},
    map_indexing_system::MapIndexingSystem,
    melee_combat_system::MeleeCombatSystem,
    monster_ai_system::MonsterAI,
    player::player_input,
    visibility_system::VisibilitySystem,
};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
}

pub struct State {
    pub world: World,
}

impl State {
    pub fn run_systems(&mut self) {
        let mut visiblity = VisibilitySystem {};
        visiblity.run_now(&self.world);

        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.world);

        let mut map_indexing = MapIndexingSystem {};
        map_indexing.run_now(&self.world);

        let mut melee_combat = MeleeCombatSystem {};
        melee_combat.run_now(&self.world);

        let mut damage = DamageSystem {};
        damage.run_now(&self.world);

        let mut item_collection = ItemColecctionSystem {};
        item_collection.run_now(&self.world);

        let mut item_drop = ItemDropSystem {};
        item_drop.run_now(&self.world);

        let mut item_use = ItemUseSystem {};
        item_use.run_now(&self.world);

        self.world.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        draw_map(&self.world, ctx);

        let mut new_run_state = *self.world.fetch::<RunState>();

        match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                self.world.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                self.run_systems();
                new_run_state = player_input(&mut self.world, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.world.maintain();
                new_run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.world.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let (item_menu_result, entity) = gui::show_inventory(self, ctx);
                match item_menu_result {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = entity.unwrap();
                        let is_ranged = self.world.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            new_run_state = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item: item_entity,
                            }
                        } else {
                            let mut intent = self.world.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.world.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            new_run_state = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::show_drop_item(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.world.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.world.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let (result, point) = gui::ranged_target(self, ctx, range);
                match result {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.world.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.world.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: point,
                                },
                            )
                            .expect("Unable to insert intent");
                    }
                }
            }
        };

        {
            let mut run_writer = self.world.write_resource::<RunState>();
            *run_writer = new_run_state;
        }

        damage_system::delete_the_dead(&mut self.world);

        {
            let positions = self.world.read_storage::<Position>();
            let renderables = self.world.read_storage::<Renderable>();
            let map = self.world.fetch::<Map>();

            let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
            for (pos, render) in data.iter() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
        }

        gui::draw_ui(&self.world, ctx);
    }
}
