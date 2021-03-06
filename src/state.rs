use rltk::{GameState, Point, Rltk};
use specs::prelude::*;
use specs::World;

use crate::{
    components::{
        CombatStats, Equipped, Hidden, InBackpack, Player, Position, Ranged, Renderable, Viewshed,
        WantsToDropItem, WantsToRemoveItem, WantsToUseItem,
    },
    damage::{self, DamageSystem},
    gamelog::GameLog,
    gui,
    hunger::HungerSystem,
    inventory::{ItemColecctionSystem, ItemDropSystem, ItemRemoveSystem, ItemUseSystem},
    map::{draw_map, Map, MAP_HEIGHT, MAP_WIDTH},
    map_indexing::MapIndexingSystem,
    maps::{self},
    melee_combat::MeleeCombatSystem,
    monster_ai::MonsterAI,
    particles::{self, ParticleSpawnSystem},
    player::player_input,
    saveload, spawner,
    trigger::TriggerSystem,
    visibility::VisibilitySystem,
    SHOW_MAPGEN_VISUALIZER,
};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    NextLevel,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    GameOver,
}

pub struct State {
    pub world: World,
    pub mapgen_next_state: Option<RunState>,
    pub mapgen_history: Vec<Map>,
    pub mapgen_index: usize,
    pub mapgen_timer: f32,
}

impl State {
    pub fn run_systems(&mut self) {
        let mut visiblity = VisibilitySystem {};
        visiblity.run_now(&self.world);

        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.world);

        let mut triggers = TriggerSystem {};
        triggers.run_now(&self.world);

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

        let mut item_remove = ItemRemoveSystem {};
        item_remove.run_now(&self.world);

        let mut hunger = HungerSystem {};
        hunger.run_now(&self.world);

        let mut particles = ParticleSpawnSystem {};
        particles.run_now(&self.world);

        self.world.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let mut to_delete: Vec<Entity> = Vec::new();

        let entities = self.world.entities();
        let player = self.world.read_storage::<Player>();
        let backpack = self.world.read_storage::<InBackpack>();
        let player_entity = self.world.fetch::<Entity>();
        let equipped = self.world.read_storage::<Equipped>();

        for entity in entities.join() {
            let mut should_delete = true;

            if let Some(_) = player.get(entity) {
                should_delete = false;
            }

            if let Some(bp) = backpack.get(entity) {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            if let Some(equip) = equipped.get(entity) {
                if equip.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        for target in self.entities_to_remove_on_level_change() {
            self.world
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        let current_depth;
        {
            let worldmap_resource = self.world.write_resource::<Map>();
            current_depth = worldmap_resource.depth;
        }
        self.generate_world_map(current_depth + 1);

        let player_entity = self.world.fetch::<Entity>();

        let mut gamelog = self.world.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        let mut player_health_store = self.world.write_storage::<CombatStats>();
        if let Some(player_health) = player_health_store.get_mut(*player_entity) {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    pub fn generate_world_map(&mut self, depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let mut map_builder = maps::random_builder(depth);
        map_builder.build_map();
        self.mapgen_history = map_builder.get_snapshot_history();

        let player_start: Position;
        {
            let mut worldmap_resource = self.world.write_resource::<Map>();
            player_start = map_builder.get_starting_position();
            *worldmap_resource = map_builder.get_map();
        }

        map_builder.spawn_entities(&mut self.world);

        let (player_x, player_y) = (player_start.x, player_start.y);
        let mut player_pos = self.world.write_resource::<Point>();
        *player_pos = Point::new(player_pos.x, player_pos.y);

        let mut position_components = self.world.write_storage::<Position>();
        let player_entity = self.world.fetch::<Entity>();
        if let Some(player_pos_comp) = position_components.get_mut(*player_entity) {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        let mut viewshed_components = self.world.write_storage::<Viewshed>();
        if let Some(viewshed) = viewshed_components.get_mut(*player_entity) {
            viewshed.dirty = true;
        }
    }

    fn game_over_cleanup(&mut self) {
        let mut to_delete = Vec::new();
        for entity in self.world.entities().join() {
            to_delete.push(entity);
        }
        for entity in to_delete.iter() {
            self.world.delete_entity(*entity).expect("Deletion failed");
        }

        {
            let player_entity = spawner::player(&mut self.world, 0, 0);
            let mut player_entity_writer = self.world.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        self.generate_world_map(1);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        particles::cull_dead_particles(&mut self.world, ctx);

        let mut new_run_state = *self.world.fetch::<RunState>();

        match new_run_state {
            RunState::MainMenu { .. } => {}
            RunState::GameOver { .. } => {}
            _ => {
                let map = self.world.fetch::<Map>();
                draw_map(&map, ctx);

                {
                    let positions = self.world.read_storage::<Position>();
                    let renderables = self.world.read_storage::<Renderable>();
                    let hidden = self.world.read_storage::<Hidden>();

                    let mut data = (&positions, &renderables, !&hidden)
                        .join()
                        .collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render, _hidden) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                        }
                    }

                    gui::draw_ui(&self.world, ctx);
                }
            }
        }

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
                match *self.world.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        new_run_state = RunState::MagicMapReveal { row: 0 }
                    }
                    _ => new_run_state = RunState::MonsterTurn,
                }
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
            RunState::NextLevel => {
                self.goto_next_level();
                new_run_state = RunState::PreRun;
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        new_run_state = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => new_run_state = RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            saveload::load_game(&mut self.world);
                            new_run_state = RunState::AwaitingInput;
                            saveload::delete_saved_game();
                        }
                        gui::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::ShowRemoveItem => {
                let (result, item_entity) = gui::remove_item_menu(self, ctx);
                match result {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.world.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.world.fetch::<Entity>(),
                                WantsToRemoveItem {
                                    item: item_entity.unwrap(),
                                },
                            )
                            .expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.world.fetch_mut::<Map>();
                for x in 0..MAP_WIDTH {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }

                if row as usize == MAP_HEIGHT - 1 {
                    new_run_state = RunState::MonsterTurn;
                } else {
                    new_run_state = RunState::MagicMapReveal { row: row + 1 };
                }
            }
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    new_run_state = self.mapgen_next_state.unwrap();
                }
                ctx.cls();
                draw_map(&self.mapgen_history[self.mapgen_index], ctx);

                self.mapgen_timer += ctx.frame_time_ms;
                if self.mapgen_timer > 300.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    if self.mapgen_index >= self.mapgen_history.len() {
                        new_run_state = self.mapgen_next_state.unwrap();
                    }
                }
            }
            RunState::SaveGame => {
                saveload::save_game(&mut self.world);

                new_run_state = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::Quit,
                }
            }
            RunState::GameOver => match gui::game_over(ctx) {
                gui::GameOverResult::NoSelection => {}
                gui::GameOverResult::QuitToMenu => {
                    self.game_over_cleanup();
                    new_run_state = RunState::MainMenu {
                        menu_selection: gui::MainMenuSelection::NewGame,
                    }
                }
            },
        };

        {
            let mut run_writer = self.world.write_resource::<RunState>();
            *run_writer = new_run_state;
        }

        damage::delete_the_dead(&mut self.world);
    }
}
