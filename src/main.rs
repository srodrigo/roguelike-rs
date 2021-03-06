use rltk::{FontCharType, GameState, Rltk, VirtualKeyCode, RGB};
use specs::{prelude::*, RunNow};
use specs_derive::Component;
use std::cmp::{max, min};

struct State {
    world: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut left_walker = LeftWalker {};
        left_walker.run_now(&self.world);
        self.world.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        let map = self.world.fetch::<Map>();
        draw_map(&map, ctx);

        let positions = self.world.read_storage::<Position>();
        let renderables = self.world.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component, Debug)]
struct Player {}

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

type Map = Vec<TileType>;

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

fn new_map() -> Map {
    let mut map = vec![TileType::Floor; 80 * 50];

    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    let mut rng = rltk::RandomNumberGenerator::new();
    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

fn try_move_player(delta_x: i32, delta_y: i32, world: &mut World) {
    let mut positions = world.write_storage::<Position>();
    let mut players = world.write_storage::<Player>();
    let map = world.fetch::<Map>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

fn player_input(game_state: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut game_state.world),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut game_state.world),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut game_state.world),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut game_state.world),
            _ => {}
        },
    }
}

#[derive(Component)]
struct LeftMover {}

struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = 79;
            }
        }
    }
}

fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    for x in 0..80 {
        for y in 0..50 {
            match map[x + y * 80] {
                TileType::Floor => {
                    ctx.set(
                        x,
                        y,
                        RGB::from_f32(0.5, 0.5, 0.5),
                        RGB::from_f32(0., 0., 0.),
                        rltk::to_cp437('.'),
                    );
                }
                TileType::Wall => {
                    ctx.set(
                        x,
                        y,
                        RGB::from_f32(0.0, 0.5, 0.0),
                        RGB::from_f32(0., 0., 0.),
                        rltk::to_cp437('#'),
                    );
                }
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
    };
    game_state.world.register::<Position>();
    game_state.world.register::<Renderable>();
    game_state.world.register::<Player>();
    game_state.world.register::<LeftMover>();

    game_state.world.insert(new_map());

    game_state
        .world
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Player {})
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
            .with(LeftMover {})
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .build();
    }

    rltk::main_loop(context, game_state)
}
