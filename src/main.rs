use rltk::{GameState, Point, Rltk};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

mod components;
use components::*;

mod map;
use map::*;

mod player;
use player::*;

mod rect;

mod gui;

mod gamelog;
use gamelog::GameLog;

mod visibility_system;
use visibility_system::VisibilitySystem;

mod monster_ai_system;
use monster_ai_system::MonsterAI;

mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;

mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;

mod damage_system;
use damage_system::DamageSystem;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
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

        self.world.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut new_run_state = *self.world.fetch::<RunState>();

        match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                self.run_systems();
                new_run_state = if player_input(&mut self.world, ctx) {
                    RunState::AwaitingInput
                } else {
                    RunState::PreRun
                }
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }
        };
        {
            let mut run_writer = self.world.write_resource::<RunState>();
            *run_writer = new_run_state;
        }

        damage_system::delete_the_dead(&mut self.world);

        draw_map(&self.world, ctx);

        {
            let positions = self.world.read_storage::<Position>();
            let renderables = self.world.read_storage::<Renderable>();
            let map = self.world.fetch::<Map>();

            for (pos, render) in (&positions, &renderables).join() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
        }

        gui::draw_ui(&self.world, ctx);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut game_state = State {
        world: World::new(),
    };
    game_state.world.register::<Position>();
    game_state.world.register::<Renderable>();
    game_state.world.register::<Player>();
    game_state.world.register::<Monster>();
    game_state.world.register::<Name>();
    game_state.world.register::<Viewshed>();
    game_state.world.register::<BlocksTile>();
    game_state.world.register::<CombatStats>();
    game_state.world.register::<WantsToMelee>();
    game_state.world.register::<SufferDamage>();

    game_state.world.insert(RunState::PreRun);

    let map = Map::new_map_rooms_and_corridors();

    let mut rng = RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let name;
        let glyph;
        match rng.roll_dice(1, 2) {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
        };

        game_state
            .world
            .create_entity()
            .with(Position { x, y })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", &name, i),
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
            })
            .build();
    }

    let (player_x, player_y) = map.rooms[0].center();
    game_state.world.insert(Point::new(player_x, player_y));

    game_state.world.insert(map);

    let player_entity = game_state
        .world
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
        })
        .build();
    game_state.world.insert(player_entity);

    game_state.world.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    rltk::main_loop(context, game_state)
}
