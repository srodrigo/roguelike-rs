use rltk::RGB;
use rltk::{GameState, Rltk};
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

pub struct State {
    pub world: World,
}

impl State {
    pub fn run_systems(&mut self) {
        let mut visiblity = VisibilitySystem {};
        visiblity.run_now(&self.world);

        self.world.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(&mut self.world, ctx);
        self.run_systems();

        draw_map(&self.world, ctx);

        let positions = self.world.read_storage::<Position>();
        let renderables = self.world.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
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
    };
    game_state.world.register::<Position>();
    game_state.world.register::<Renderable>();
    game_state.world.register::<Player>();
    game_state.world.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    game_state.world.insert(map);

    game_state
        .world
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Player {})
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

    for i in 0..10 {
        game_state
            .world
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .build();
    }

    rltk::main_loop(context, game_state)
}
