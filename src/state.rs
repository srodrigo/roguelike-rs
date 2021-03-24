use rltk::{GameState, Rltk};
use specs::prelude::*;
use specs::World;

use crate::{
    components::{Position, Renderable, WantsToDrinkPotion, WantsToDropItem},
    damage_system::{self, DamageSystem},
    gui,
    inventory_system::{ItemColecctionSystem, ItemDropSystem, PotionUseSystem},
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

        let mut potion_use = PotionUseSystem {};
        potion_use.run_now(&self.world);

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
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.world.write_storage::<WantsToDrinkPotion>();
                        intent
                            .insert(
                                *self.world.fetch::<Entity>(),
                                WantsToDrinkPotion {
                                    potion: item_entity,
                                },
                            )
                            .expect("Unable to insert intent");
                        new_run_state = RunState::AwaitingInput;
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