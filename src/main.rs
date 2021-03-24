use rltk::Point;
use rltk::RandomNumberGenerator;
use specs::prelude::*;

mod components;
use components::*;

mod map;
use map::*;

mod player;

mod rect;

mod gui;

mod state;
use state::{RunState, State};

mod gamelog;
use gamelog::GameLog;

mod spawner;

mod damage_system;
mod inventory_system;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod visibility_system;

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
    game_state.world.register::<Item>();
    game_state.world.register::<Potion>();
    game_state.world.register::<InBackpack>();
    game_state.world.register::<WantsToPickUpItem>();
    game_state.world.register::<WantsToDropItem>();
    game_state.world.register::<WantsToDrinkPotion>();

    game_state.world.insert(RunState::PreRun);

    let map = Map::new_map_rooms_and_corridors();

    game_state.world.insert(RandomNumberGenerator::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut game_state.world, room);
    }

    let (player_x, player_y) = map.rooms[0].center();
    game_state.world.insert(Point::new(player_x, player_y));

    game_state.world.insert(map);

    let player = spawner::player(&mut game_state.world, player_x, player_y);
    game_state.world.insert(player);

    game_state.world.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    rltk::main_loop(context, game_state)
}
