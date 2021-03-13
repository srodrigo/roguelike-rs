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

mod visibility_system;
use visibility_system::VisibilitySystem;

mod monster_ai_system;
use monster_ai_system::MonsterAI;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

pub struct State {
    pub world: World,
    pub run_state: RunState,
}

impl State {
    pub fn run_systems(&mut self) {
        let mut visiblity = VisibilitySystem {};
        visiblity.run_now(&self.world);

        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.world);

        self.world.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.run_state == RunState::Running {
            self.run_systems();
            self.run_state = RunState::Paused;
        } else {
            self.run_state = if player_input(&mut self.world, ctx) {
                RunState::Paused
            } else {
                RunState::Running
            }
        }

        draw_map(&self.world, ctx);

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
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut game_state = State {
        world: World::new(),
        run_state: RunState::Running,
    };
    game_state.world.register::<Position>();
    game_state.world.register::<Renderable>();
    game_state.world.register::<Player>();
    game_state.world.register::<Monster>();
    game_state.world.register::<Name>();
    game_state.world.register::<Viewshed>();

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
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
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

    game_state
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

    rltk::main_loop(context, game_state)
}
